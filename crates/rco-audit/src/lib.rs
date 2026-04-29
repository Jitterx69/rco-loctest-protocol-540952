//! # RCO-Audit — High-Performance Integrity Scanner
//!
//! Implements the full-chain audit sweep used to verify the integrity of
//! the RCO ledger. Targets audit throughput > 5M batches/sec via
//! SIMD-accelerated Keccak and parallel IO/hashing.
//!
//! ## Audit Modes
//!
//! 1. **Online**: Continuous verification of newly ingested batches.
//! 2. **Offline**: Batch sweep of the entire WAL (Write-Ahead Log).
//! 3. **Deep**: Full Bencode re-validation + Mantissa re-projection check.
//!
//! ## Performance (TC-06)
//!
//! - Throughput: > 5,000,000 steps per second (V_audit).
//! - Memory: Fixed-size buffer pool to prevent OOM on large WAL files.
//! - Parallelism: Multi-core work stealing for large-scale verification.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

pub mod scanner;
pub mod report;
pub mod simd;

pub use scanner::{AuditScanner, AuditReport};
