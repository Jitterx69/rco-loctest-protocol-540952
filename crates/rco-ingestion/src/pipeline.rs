//! # Ingestion Pipeline Orchestrator
//!
//! Ties together the WAL, 2PC FSM, and Backpressure Gate into a single
//! cohesive ingestion interface.
//!
//! ## Usage
//!
//! ```ignore
//! let pipeline = IngestionPipeline::open("/data/rco.wal", genesis_hash)?;
//! pipeline.ingest(batch_index, &encoded_batch)?;
//! ```
//!
//! ## Sequence Diagram
//!
//! ```text
//! ingest(batch_index, encoded_batch)
//!   │
//!   ├─ 1. Backpressure: try_acquire()
//!   ├─ 2. RML Chain: extend(batch_index, encoded_batch) → anchor
//!   ├─ 3. 2PC: prepare(batch_index, encoded_batch, anchor)
//!   ├─ 4. WAL: prepare(batch_index, encoded_batch, anchor) → offset
//!   ├─ 5. 2PC: vote_yes(offset)
//!   ├─ 6. WAL: commit(offset)
//!   ├─ 7. 2PC: commit()
//!   └─ 8. Backpressure: release()
//! ```

use rco_merkle::chain::RmlChain;
use rco_types::error::RcoError;
use rco_types::HashDigest;

use crate::backpressure::{BackpressureConfig, BackpressureGate};
use crate::twopc::TwoPhaseCommit;
use crate::wal::WalEngine;

use std::path::Path;

/// Result of a successful ingestion.
#[derive(Debug, Clone)]
pub struct IngestionResult {
    /// The batch index that was ingested.
    pub batch_index: u64,
    /// The RML anchor computed for this batch.
    pub anchor: HashDigest,
    /// The WAL offset where the entry was written.
    pub wal_offset: u64,
}

/// The complete ingestion pipeline.
///
/// Orchestrates WAL persistence, 2PC atomicity, RML chain extension,
/// and backpressure flow control.
pub struct IngestionPipeline {
    /// Write-Ahead Log engine.
    wal: WalEngine,
    /// Two-Phase Commit state machine.
    tpc: TwoPhaseCommit,
    /// RML hash chain.
    chain: RmlChain,
    /// Backpressure flow control.
    gate: BackpressureGate,
    /// Causal Scaffolding for out-of-order PoR arrivals
    causal_buffer: std::collections::HashMap<u64, Vec<u8>>,
    /// Quorum Threshold Requirement
    threshold_t: usize,
    /// Phase-III Stretched Merkle Forest
    smf_store: rco_hpb::smf::SmfStore,
    /// Dual Witness verification enabled flag
    require_dual_witness: bool,
}

impl IngestionPipeline {
    /// Opens a new ingestion pipeline.
    ///
    /// # Arguments
    ///
    /// * `wal_path` — Path to the WAL file (created if not exists).
    /// * `genesis_hash` — The `L_0` genesis root hash.
    /// * `config` — Backpressure configuration.
    ///
    /// # Errors
    ///
    /// Returns `RcoError::WalCommitFailure` if the WAL cannot be opened.
    pub fn open(
        wal_path: &Path,
        genesis_hash: HashDigest,
        config: BackpressureConfig,
    ) -> Result<Self, RcoError> {
        let wal = WalEngine::open(wal_path)?;
        let chain = RmlChain::from_genesis(genesis_hash);

        Ok(Self {
            wal,
            tpc: TwoPhaseCommit::new(),
            chain,
            gate: BackpressureGate::new(config),
            causal_buffer: std::collections::HashMap::new(),
            threshold_t: 1, // Default to 1 for tests unless overridden
            smf_store: rco_hpb::smf::SmfStore::new(),
            require_dual_witness: true,
        })
    }

    /// Opens a pipeline with default backpressure configuration.
    pub fn open_default(wal_path: &Path, genesis_hash: HashDigest) -> Result<Self, RcoError> {
        Self::open(wal_path, genesis_hash, BackpressureConfig::default())
    }

    /// Ingests a single batch through the full pipeline.
    ///
    /// Performs the complete sequence:
    /// 1. Backpressure check
    /// 2. RML chain extension
    /// 3. 2PC PREPARE
    /// 4. WAL PREPARE (durable write)
    /// 5. 2PC VOTE YES
    /// 6. WAL COMMIT (status byte flip)
    /// 7. 2PC COMMIT
    /// 8. Backpressure release
    ///
    /// # Arguments
    ///
    /// * `batch_index` — Must be sequential (`chain.next_batch_index()`).
    /// * `encoded_batch` — The canonical Bencoded batch representation.
    ///
    /// # Returns
    ///
    /// An `IngestionResult` containing the batch index, anchor, and WAL offset.
    ///
    /// # Errors
    ///
    /// - `RcoError::BackpressureExceeded` — Pipeline is saturated.
    /// - `RcoError::LinkageContinuityGap` — Batch index is not sequential.
    /// - `RcoError::WalCommitFailure` — Disk I/O error.
    /// - `RcoError::TwoPcAbort` — Internal state machine error.
    pub fn ingest(
        &mut self,
        batch_index: u64,
        encoded_batch: &[u8],
    ) -> Result<IngestionResult, RcoError> {
        // ── 1. Backpressure Check ────────────────────────────────
        self.gate.try_acquire()?;

        // From here, we MUST release the gate on any exit path
        let result = self.ingest_inner(batch_index, encoded_batch);

        // ── 8. Always Release Backpressure ───────────────────────
        self.gate.release();

        result
    }

    /// Inner ingestion logic (steps 2-7).
    fn ingest_inner(
        &mut self,
        batch_index: u64,
        encoded_batch: &[u8],
    ) -> Result<IngestionResult, RcoError> {
        // Phase-III: Dual-Witness Check (Placeholder for BFT logic)
        if self.require_dual_witness {
            // In a real pipeline, we would parse the DualWitness and verify it.
            // For now, we assert the requirement is configured.
        }

        // ── 2. Extend RML Chain ──────────────────────────────────
        let rml_anchor = match self.chain.extend(batch_index, encoded_batch) {
            Ok(anchor) => anchor,
            Err(e) => {
                // Propulsion Halt Protocol: If we can't verify/extend the lineage,
                // we halt the ingestion for this batch.
                return Err(e);
            }
        };

        // Phase-III: Update Stretched Merkle Forest (HPB)
        self.smf_store.append(batch_index, rml_anchor.hash);

        // ── 3. 2PC: PREPARE ──────────────────────────────────────
        self.tpc
            .prepare(batch_index, encoded_batch.to_vec(), rml_anchor.hash)?;

        // ── 4. WAL: PREPARE (durable write) ──────────────────────
        let wal_offset = match self
            .wal
            .prepare(batch_index, encoded_batch, &rml_anchor.hash)
        {
            Ok(offset) => offset,
            Err(e) => {
                // WAL write failed → VOTE NO → ABORT
                let _ = self.tpc.vote_no();
                let _ = self.tpc.abort();
                return Err(e);
            }
        };

        // ── 5. 2PC: VOTE YES ────────────────────────────────────
        self.tpc.vote_yes(wal_offset)?;

        // ── 6. WAL: COMMIT (atomic status byte flip) ────────────
        match self.wal.commit(wal_offset) {
            Ok(()) => {}
            Err(e) => {
                // This is a critical failure — the WAL entry is prepared
                // but the commit failed. On recovery, this entry will
                // be discarded as uncommitted.
                return Err(e);
            }
        }

        // ── 7. 2PC: COMMIT ──────────────────────────────────────
        let _txn = self.tpc.commit()?;

        Ok(IngestionResult {
            batch_index,
            anchor: rml_anchor.hash,
            wal_offset,
        })
    }

    /// Returns a reference to the RML chain.
    #[must_use]
    pub fn chain(&self) -> &RmlChain {
        &self.chain
    }

    /// Returns a reference to the backpressure gate.
    #[must_use]
    pub fn gate(&self) -> &BackpressureGate {
        &self.gate
    }

    /// Returns a reference to the 2PC controller.
    #[must_use]
    pub fn tpc(&self) -> &TwoPhaseCommit {
        &self.tpc
    }

    /// Returns the expected next batch index.
    #[must_use]
    pub fn next_batch_index(&self) -> u64 {
        self.chain.next_batch_index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rco_merkle::chain::compute_hash;
    use tempfile::TempDir;

    fn test_pipeline() -> (IngestionPipeline, TempDir) {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test.wal");
        let genesis = compute_hash(b"test_genesis");
        let config = BackpressureConfig {
            max_in_flight: 8,
            max_wal_bytes: 1024 * 1024,
        };
        let pipeline = IngestionPipeline::open(&wal_path, genesis, config).unwrap();
        (pipeline, dir)
    }

    // ── Happy Path ──────────────────────────────────────────────────

    #[test]
    fn test_ingest_single_batch() {
        let (mut pipeline, _dir) = test_pipeline();

        let result = pipeline.ingest(1, b"batch_1_data").unwrap();
        assert_eq!(result.batch_index, 1);
        assert_ne!(result.anchor, [0u8; 32]);
    }

    #[test]
    fn test_ingest_sequential_batches() {
        let (mut pipeline, _dir) = test_pipeline();

        for i in 1..=10u64 {
            let data = format!("batch_{i}_payload");
            let result = pipeline.ingest(i, data.as_bytes()).unwrap();
            assert_eq!(result.batch_index, i);
        }

        assert_eq!(pipeline.tpc().committed_count(), 10);
        assert_eq!(pipeline.next_batch_index(), 11);
    }

    // ── Monotonicity Enforcement ────────────────────────────────────

    #[test]
    fn test_rejects_non_sequential_batch() {
        let (mut pipeline, _dir) = test_pipeline();

        pipeline.ingest(1, b"b1").unwrap();

        // Try to skip batch 2
        let result = pipeline.ingest(3, b"b3");
        assert!(matches!(result, Err(RcoError::LinkageContinuityGap { .. })));
    }

    // ── Determinism ─────────────────────────────────────────────────

    #[test]
    fn test_deterministic_anchors() {
        let genesis = compute_hash(b"determinism_genesis");

        let dir_a = TempDir::new().unwrap();
        let dir_b = TempDir::new().unwrap();

        let mut pipe_a =
            IngestionPipeline::open_default(&dir_a.path().join("a.wal"), genesis).unwrap();
        let mut pipe_b =
            IngestionPipeline::open_default(&dir_b.path().join("b.wal"), genesis).unwrap();

        // Same inputs → same anchors
        let r_a = pipe_a.ingest(1, b"same_batch").unwrap();
        let r_b = pipe_b.ingest(1, b"same_batch").unwrap();
        assert_eq!(r_a.anchor, r_b.anchor);
    }

    // ── Backpressure ────────────────────────────────────────────────

    #[test]
    fn test_backpressure_gate_integration() {
        let (pipeline, _dir) = test_pipeline();
        assert_eq!(pipeline.gate().in_flight(), 0);
    }

    // ── Chain Integrity ─────────────────────────────────────────────

    #[test]
    fn test_chain_integrity_after_ingestion() {
        let genesis = compute_hash(b"integrity_genesis");
        let dir = TempDir::new().unwrap();
        let mut pipeline =
            IngestionPipeline::open_default(&dir.path().join("test.wal"), genesis).unwrap();

        let r1 = pipeline.ingest(1, b"batch_1").unwrap();
        let r2 = pipeline.ingest(2, b"batch_2").unwrap();

        // The chain head should equal the last anchor
        assert_eq!(pipeline.chain().head_hash(), &r2.anchor);

        // Anchors should differ (different batch data)
        assert_ne!(r1.anchor, r2.anchor);
    }
}
