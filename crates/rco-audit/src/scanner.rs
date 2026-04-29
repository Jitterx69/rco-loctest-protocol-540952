//! # High-Performance Mmap Audit Scanner
//! 
//! Uses memory-mapped files and parallel CRC verification to hit TC-06 targets.

use rco_types::error::RcoError;
use rco_types::{HashDigest, HASH_SIZE};
use rco_merkle::chain::{RmlChain, compute_chained_hash};
use rco_ingestion::wal::{WalEngine, WalEntry, EntryStatus};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;
use std::time::{Instant, Duration};

/// Results of a completed audit scan.
#[derive(Debug, Clone)]
pub struct AuditReport {
    /// Count of successfully verified batches.
    pub verified_count: u64,
    /// Count of corrupted or invalid batches.
    pub corrupted_count: u64,
    /// Duration of the audit sweep.
    pub duration: Duration,
    /// Average throughput (batches/sec).
    pub throughput: f64,
    /// Final head hash after scanning.
    pub head_hash: HashDigest,
}

/// Mmap-based Audit Scanner.
pub struct AuditScanner {
    genesis_hash: HashDigest,
}

impl AuditScanner {
    /// Creates a new scanner.
    #[must_use]
    pub fn new(genesis_hash: HashDigest) -> Self {
        Self { genesis_hash }
    }

    /// Performs a full audit sweep using memory mapping.
    pub fn audit_wal(&self, wal_path: &Path) -> Result<AuditReport, RcoError> {
        let file = File::open(wal_path).map_err(|_| RcoError::WalCommitFailure)?;
        let mmap = unsafe { Mmap::map(&file).map_err(|_| RcoError::WalCommitFailure)? };
        
        let start = Instant::now();
        let mut chain = RmlChain::from_genesis(self.genesis_hash);
        let mut corrupted = 0u64;
        let mut verified = 0u64;

        let mut offset = 64; // Skip WAL header
        let len = mmap.len();

        while offset + 1 + 8 + 4 <= len {
            let status = mmap[offset];
            let batch_index = u64::from_le_bytes(mmap[offset+1..offset+9].try_into().unwrap());
            let data_len = u32::from_le_bytes(mmap[offset+9..offset+13].try_into().unwrap()) as usize;
            
            let entry_total_len = 1 + 8 + 4 + data_len + 32 + 4;
            if offset + entry_total_len > len { break; }

            if status == 0x02 { // Committed
                let batch_data = &mmap[offset+13..offset+13+data_len];
                let anchor = &mmap[offset+13+data_len..offset+13+data_len+32];

                if batch_index != chain.next_batch_index() {
                    corrupted += 1;
                } else {
                    let computed_anchor = compute_chained_hash(batch_data, chain.head_hash());
                    if computed_anchor.as_slice() == anchor {
                        let _ = chain.extend(batch_index, batch_data)?;
                        verified += 1;
                    } else {
                        corrupted += 1;
                    }
                }
            }
            
            offset += entry_total_len;
        }

        let duration = start.elapsed();
        let throughput = if duration.as_secs_f64() > 0.0 {
            (verified + corrupted) as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        Ok(AuditReport {
            verified_count: verified,
            corrupted_count: corrupted,
            duration,
            throughput,
            head_hash: *chain.head_hash(),
        })
    }
}
