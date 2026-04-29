//! # Write-Ahead Log (WAL) Engine
//!
//! Append-only, crash-consistent log for atomic batch + anchor persistence.
//!
//! ## On-Disk Format
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │ WAL Header (64 bytes)                           │
//! ├─────────────────────────────────────────────────┤
//! │ Entry 0: [Status][Len][BatchData][Anchor][CRC]  │
//! │ Entry 1: [Status][Len][BatchData][Anchor][CRC]  │
//! │ ...                                             │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ## Entry Status Byte
//!
//! - `0x00` — Unused/Invalid
//! - `0x01` — Prepared (written but not committed)
//! - `0x02` — Committed (durable and finalized)
//! - `0xFF` — Tombstone (logically deleted after compaction)
//!
//! ## Crash Recovery
//!
//! On startup, the WAL is scanned:
//! - Committed entries (`0x02`) are replayed into the in-memory chain.
//! - Prepared entries (`0x01`) are discarded (incomplete 2PC).
//! - CRC-invalid entries are discarded (torn writes).

use rco_types::error::RcoError;
use rco_types::{HashDigest, HASH_SIZE};

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// WAL entry status byte values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EntryStatus {
    /// Entry slot is unused.
    Invalid = 0x00,
    /// Entry has been written but 2PC has not committed.
    Prepared = 0x01,
    /// Entry is committed and durable.
    Committed = 0x02,
    /// Entry has been compacted away.
    Tombstone = 0xFF,
}

impl EntryStatus {
    /// Converts a raw byte to an `EntryStatus`.
    fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(Self::Invalid),
            0x01 => Some(Self::Prepared),
            0x02 => Some(Self::Committed),
            0xFF => Some(Self::Tombstone),
            _ => None,
        }
    }
}

/// A single WAL entry (in-memory representation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalEntry {
    /// Status of this entry.
    pub status: EntryStatus,
    /// Batch index this entry corresponds to.
    pub batch_index: u64,
    /// Serialized batch data (Bencoded).
    pub batch_data: Vec<u8>,
    /// RML lineage anchor hash.
    pub anchor: HashDigest,
    /// CRC-32 checksum of (batch_data ‖ anchor).
    pub checksum: u32,
}

/// WAL file header (64 bytes).
const WAL_HEADER_SIZE: usize = 64;
/// Magic bytes identifying an RCO WAL file.
const WAL_MAGIC: [u8; 8] = *b"RCOWAL01";

/// The Write-Ahead Log engine.
///
/// Provides crash-consistent append and commit operations.
pub struct WalEngine {
    /// Path to the WAL file.
    path: PathBuf,
    /// The underlying file handle.
    file: File,
    /// Number of entries currently in the WAL.
    entry_count: u64,
    /// Offset of the next write position.
    write_offset: u64,
}

impl WalEngine {
    /// Opens or creates a WAL file at the given path.
    ///
    /// If the file exists, it is opened and the header is validated.
    /// If it doesn't exist, a new WAL is created with a fresh header.
    ///
    /// # Errors
    ///
    /// Returns `RcoError::WalCommitFailure` on I/O errors.
    pub fn open(path: &Path) -> Result<Self, RcoError> {
        let exists = path.exists();

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|_| RcoError::WalCommitFailure)?;

        if exists {
            // Validate header
            Self::validate_header(&mut file)?;
        } else {
            // Write fresh header
            Self::write_header(&mut file)?;
        }

        // Determine entry count and write offset by scanning
        let (entry_count, write_offset) = Self::scan_entries(&mut file)?;

        Ok(Self {
            path: path.to_path_buf(),
            file,
            entry_count,
            write_offset,
        })
    }

    /// Appends a new entry in PREPARED state.
    ///
    /// The entry is written to disk but NOT yet committed.
    /// The caller must call `commit()` to finalize.
    ///
    /// Returns the file offset of the entry for later commit.
    pub fn prepare(
        &mut self,
        batch_index: u64,
        batch_data: &[u8],
        anchor: &HashDigest,
    ) -> Result<u64, RcoError> {
        let entry_offset = self.write_offset;

        // Compute CRC-32 over batch_data ‖ anchor
        let checksum = compute_crc32(batch_data, anchor);

        // Write entry: [status:1][batch_index:8][data_len:4][data:N][anchor:32][crc:4]
        let mut buf = Vec::new();
        buf.push(EntryStatus::Prepared as u8); // 1 byte: status
        buf.extend_from_slice(&batch_index.to_le_bytes()); // 8 bytes: batch_index
        buf.extend_from_slice(&(batch_data.len() as u32).to_le_bytes()); // 4 bytes: data length
        buf.extend_from_slice(batch_data); // N bytes: batch data
        buf.extend_from_slice(anchor); // 32 bytes: anchor
        buf.extend_from_slice(&checksum.to_le_bytes()); // 4 bytes: CRC

        self.file
            .seek(SeekFrom::Start(entry_offset))
            .map_err(|_| RcoError::WalCommitFailure)?;
        self.file
            .write_all(&buf)
            .map_err(|_| RcoError::WalCommitFailure)?;

        // fdatasync: ensure the data hits persistent storage
        self.file
            .sync_data()
            .map_err(|_| RcoError::WalCommitFailure)?;

        self.write_offset += buf.len() as u64;

        Ok(entry_offset)
    }

    /// Commits a previously prepared entry by flipping its status byte.
    ///
    /// This is the atomic operation — a single byte write followed by fdatasync.
    /// If the process crashes after this write, the entry will be seen as
    /// committed on recovery.
    pub fn commit(&mut self, entry_offset: u64) -> Result<(), RcoError> {
        // Overwrite the status byte at the entry offset
        self.file
            .seek(SeekFrom::Start(entry_offset))
            .map_err(|_| RcoError::WalCommitFailure)?;
        self.file
            .write_all(&[EntryStatus::Committed as u8])
            .map_err(|_| RcoError::WalCommitFailure)?;

        // fdatasync: make the status flip durable
        self.file
            .sync_data()
            .map_err(|_| RcoError::WalCommitFailure)?;

        self.entry_count += 1;
        Ok(())
    }

    /// Recovers committed entries from the WAL.
    ///
    /// Scans the entire WAL and returns all entries with status `Committed`.
    /// Prepared (uncommitted) entries are discarded.
    pub fn recover(&mut self) -> Result<Vec<WalEntry>, RcoError> {
        self.file
            .seek(SeekFrom::Start(WAL_HEADER_SIZE as u64))
            .map_err(|_| RcoError::WalCommitFailure)?;

        let mut entries = Vec::new();
        loop {
            match self.read_entry() {
                Ok(Some(entry)) => {
                    if entry.status == EntryStatus::Committed {
                        entries.push(entry);
                    }
                }
                Ok(None) => break,
                Err(RcoError::BitRotDetected { .. }) => {
                    // During standard recovery, we skip bit-rotted entries
                    // but continue reading if possible? No, standard recovery
                    // usually stops to avoid replaying corrupted state.
                    break;
                }
                Err(_) => break,
            }
        }

        Ok(entries)
    }

    /// Returns an iterator over all entries in the WAL, including those with bit-rot.
    ///
    /// This is used by the audit scanner to detect integrity violations.
    pub fn raw_entries(&mut self) -> WalIterator {
        let _ = self.file.seek(SeekFrom::Start(WAL_HEADER_SIZE as u64));
        WalIterator { wal: self }
    }

    /// Returns the number of committed entries.
    #[must_use]
    pub fn entry_count(&self) -> u64 {
        self.entry_count
    }

    /// Returns the path to the WAL file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    // ── Internal Methods ────────────────────────────────────────────

    fn write_header(file: &mut File) -> Result<(), RcoError> {
        let mut header = [0u8; WAL_HEADER_SIZE];
        header[0..8].copy_from_slice(&WAL_MAGIC);
        // Bytes 8..16: version (1)
        header[8..16].copy_from_slice(&1u64.to_le_bytes());
        // Rest is reserved zeros

        file.seek(SeekFrom::Start(0))
            .map_err(|_| RcoError::WalCommitFailure)?;
        file.write_all(&header)
            .map_err(|_| RcoError::WalCommitFailure)?;
        file.sync_data().map_err(|_| RcoError::WalCommitFailure)?;

        Ok(())
    }

    fn validate_header(file: &mut File) -> Result<(), RcoError> {
        let mut header = [0u8; WAL_HEADER_SIZE];
        file.seek(SeekFrom::Start(0))
            .map_err(|_| RcoError::WalCommitFailure)?;

        match file.read_exact(&mut header) {
            Ok(()) => {}
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // File is smaller than header — corrupt or truncated
                return Err(RcoError::WalCommitFailure);
            }
            Err(_) => return Err(RcoError::WalCommitFailure),
        }

        if header[0..8] != WAL_MAGIC {
            return Err(RcoError::WalCommitFailure);
        }

        Ok(())
    }

    fn scan_entries(file: &mut File) -> Result<(u64, u64), RcoError> {
        file.seek(SeekFrom::Start(WAL_HEADER_SIZE as u64))
            .map_err(|_| RcoError::WalCommitFailure)?;

        let mut count = 0u64;
        let mut offset = WAL_HEADER_SIZE as u64;

        loop {
            // Try to read status byte
            let mut status_byte = [0u8; 1];
            match file.read_exact(&mut status_byte) {
                Ok(()) => {}
                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(_) => break,
            }

            let status = match EntryStatus::from_byte(status_byte[0]) {
                Some(s) => s,
                None => break,
            };

            // Read batch_index (8 bytes)
            let mut idx_bytes = [0u8; 8];
            if file.read_exact(&mut idx_bytes).is_err() {
                break;
            }

            // Read data_len (4 bytes)
            let mut len_bytes = [0u8; 4];
            if file.read_exact(&mut len_bytes).is_err() {
                break;
            }
            let data_len = u32::from_le_bytes(len_bytes) as u64;

            // Skip data + anchor (32) + crc (4)
            let skip = data_len + HASH_SIZE as u64 + 4;
            if file.seek(SeekFrom::Current(skip as i64)).is_err() {
                break;
            }

            if status == EntryStatus::Committed {
                count += 1;
            }

            // 1 (status) + 8 (index) + 4 (len) + data_len + 32 (anchor) + 4 (crc)
            offset += 1 + 8 + 4 + data_len + HASH_SIZE as u64 + 4;
        }

        Ok((count, offset))
    }

    fn read_entry(&mut self) -> Result<Option<WalEntry>, RcoError> {
        // Read status byte
        let mut status_byte = [0u8; 1];
        match self.file.read_exact(&mut status_byte) {
            Ok(()) => {}
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(_) => return Err(RcoError::WalCommitFailure),
        }

        let status = EntryStatus::from_byte(status_byte[0]).ok_or(RcoError::WalCommitFailure)?;

        if status == EntryStatus::Invalid {
            return Ok(None);
        }

        // Read batch_index
        let mut idx_bytes = [0u8; 8];
        self.file
            .read_exact(&mut idx_bytes)
            .map_err(|_| RcoError::WalCommitFailure)?;
        let batch_index = u64::from_le_bytes(idx_bytes);

        // Read data length
        let mut len_bytes = [0u8; 4];
        self.file
            .read_exact(&mut len_bytes)
            .map_err(|_| RcoError::WalCommitFailure)?;
        let data_len = u32::from_le_bytes(len_bytes) as usize;

        // Read batch data
        let mut batch_data = vec![0u8; data_len];
        self.file
            .read_exact(&mut batch_data)
            .map_err(|_| RcoError::WalCommitFailure)?;

        // Read anchor
        let mut anchor = [0u8; HASH_SIZE];
        self.file
            .read_exact(&mut anchor)
            .map_err(|_| RcoError::WalCommitFailure)?;

        // Read CRC
        let mut crc_bytes = [0u8; 4];
        self.file
            .read_exact(&mut crc_bytes)
            .map_err(|_| RcoError::WalCommitFailure)?;
        let stored_crc = u32::from_le_bytes(crc_bytes);

        // Validate CRC
        let computed_crc = compute_crc32(&batch_data, &anchor);
        if stored_crc != computed_crc {
            return Err(RcoError::BitRotDetected { batch_index });
        }

        Ok(Some(WalEntry {
            status,
            batch_index,
            batch_data,
            anchor,
            checksum: stored_crc,
        }))
    }
}

/// Iterator over WAL entries for auditing.
pub struct WalIterator<'a> {
    wal: &'a mut WalEngine,
}

impl<'a> Iterator for WalIterator<'a> {
    type Item = Result<WalEntry, RcoError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.wal.read_entry() {
            Ok(Some(entry)) => Some(Ok(entry)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// Computes CRC-32 over `data ‖ anchor` for integrity checking.
///
/// Uses a simple CRC-32/ISO-HDLC (same as zlib) implementation.
/// This is NOT for cryptographic integrity (that's the RML anchor's job) —
/// it detects torn writes and bit-rot.
fn compute_crc32(data: &[u8], anchor: &[u8; HASH_SIZE]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data.iter().chain(anchor.iter()) {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_wal(name: &str) -> (WalEngine, TempDir) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(name);
        let wal = WalEngine::open(&path).unwrap();
        (wal, dir)
    }

    #[test]
    fn test_create_new_wal() {
        let (wal, _dir) = test_wal("test.wal");
        assert_eq!(wal.entry_count(), 0);
    }

    #[test]
    fn test_prepare_and_commit() {
        let (mut wal, _dir) = test_wal("test.wal");
        let anchor = [0xAB; HASH_SIZE];

        let offset = wal.prepare(1, b"batch_data", &anchor).unwrap();
        wal.commit(offset).unwrap();

        assert_eq!(wal.entry_count(), 1);
    }

    #[test]
    fn test_prepare_without_commit_is_discarded() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.wal");

        {
            let mut wal = WalEngine::open(&path).unwrap();
            let anchor = [0xAB; HASH_SIZE];
            // Prepare but don't commit
            wal.prepare(1, b"uncommitted", &anchor).unwrap();
        }

        // Reopen — prepared entry should be discarded
        let mut wal = WalEngine::open(&path).unwrap();
        let entries = wal.recover().unwrap();
        assert!(
            entries.is_empty(),
            "Prepared-only entries must be discarded"
        );
    }

    #[test]
    fn test_committed_entries_survive_reopen() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.wal");
        let anchor = [0xCD; HASH_SIZE];

        {
            let mut wal = WalEngine::open(&path).unwrap();
            let off = wal.prepare(1, b"batch_1", &anchor).unwrap();
            wal.commit(off).unwrap();
        }

        // Reopen and recover
        let mut wal = WalEngine::open(&path).unwrap();
        let entries = wal.recover().unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].batch_index, 1);
        assert_eq!(entries[0].batch_data, b"batch_1");
        assert_eq!(entries[0].anchor, anchor);
        assert_eq!(entries[0].status, EntryStatus::Committed);
    }

    #[test]
    fn test_multiple_entries() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.wal");

        {
            let mut wal = WalEngine::open(&path).unwrap();
            for i in 1..=5u64 {
                let data = format!("batch_{i}");
                let anchor = [i as u8; HASH_SIZE];
                let off = wal.prepare(i, data.as_bytes(), &anchor).unwrap();
                wal.commit(off).unwrap();
            }
        }

        let mut wal = WalEngine::open(&path).unwrap();
        let entries = wal.recover().unwrap();
        assert_eq!(entries.len(), 5);

        for (i, entry) in entries.iter().enumerate() {
            assert_eq!(entry.batch_index, (i + 1) as u64);
        }
    }

    #[test]
    fn test_mixed_prepared_and_committed() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.wal");

        {
            let mut wal = WalEngine::open(&path).unwrap();
            let anchor = [0xAA; HASH_SIZE];

            // Commit batch 1
            let off1 = wal.prepare(1, b"committed", &anchor).unwrap();
            wal.commit(off1).unwrap();

            // Prepare batch 2 but don't commit
            wal.prepare(2, b"uncommitted", &anchor).unwrap();

            // Commit batch 3
            let off3 = wal.prepare(3, b"also_committed", &anchor).unwrap();
            wal.commit(off3).unwrap();
        }

        let mut wal = WalEngine::open(&path).unwrap();
        let entries = wal.recover().unwrap();

        // Only batches 1 and 3 should survive
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].batch_data, b"committed");
        assert_eq!(entries[1].batch_data, b"also_committed");
    }

    #[test]
    fn test_crc_integrity() {
        let anchor = [0xFF; HASH_SIZE];
        let crc1 = compute_crc32(b"data_a", &anchor);
        let crc2 = compute_crc32(b"data_b", &anchor);
        assert_ne!(crc1, crc2, "Different data must produce different CRC");
    }

    #[test]
    fn test_crc_deterministic() {
        let anchor = [0x42; HASH_SIZE];
        let a = compute_crc32(b"same_data", &anchor);
        let b = compute_crc32(b"same_data", &anchor);
        assert_eq!(a, b);
    }
}
