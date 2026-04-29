//! # RCO-Ingestion — Atomic Two-Phase Commit Gateway
//!
//! Implements the crash-consistent ingestion pipeline that bridges the
//! simulation SDK and the persistent storage layer.
//!
//! ## Architecture
//!
//! ```text
//! Simulation → [Backpressure Gate] → [2PC FSM] → [WAL Engine] → Persisted
//! ```
//!
//! ## Atomicity Invariant
//!
//! ```text
//! (B_n ∈ Storage) ⟺ (L_n ∈ Storage)
//! ```
//!
//! A batch and its lineage anchor are ALWAYS committed together.
//! There are never orphan anchors or unanchored batches.
//!
//! ## State Machine: PREPARE → VOTE → COMMIT
//!
//! The Two-Phase Commit (2PC) protocol ensures atomicity:
//!
//! 1. **PREPARE**: Serialize the batch, compute anchor, validate invariants.
//! 2. **VOTE**: Write to WAL (durable). If WAL write succeeds → vote YES.
//! 3. **COMMIT**: Mark the WAL entry as committed. Update in-memory state.
//!
//! On crash during any phase:
//! - PREPARE incomplete → nothing written, safe.
//! - VOTE incomplete → WAL entry is uncommitted, discarded on recovery.
//! - COMMIT incomplete → WAL entry is committed, replayed on recovery.
//!
//! ## Acceptance Criterion (TC-05)
//!
//! 10-node cluster, random SIGKILL during COMMIT → zero orphan anchors.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

pub mod backpressure;
pub mod pipeline;
pub mod twopc;
pub mod wal;

pub use backpressure::BackpressureGate;
pub use pipeline::IngestionPipeline;
pub use twopc::TwoPhaseCommit;
