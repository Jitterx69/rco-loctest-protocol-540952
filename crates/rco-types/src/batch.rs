//! # Telemetry Batch Structures
//!
//! Defines the canonical `TelemetryBatch` — the atomic unit of data flowing
//! through the RCO pipeline. Every batch is Bencode-serialized, P14-projected,
//! and hashed into the Recursive Merkle-Lineage (RML) chain.
//!
//! ## Structural Invariants
//!
//! A valid batch satisfies:
//! 1. All reward values are `i128` (post-P14 projection).
//! 2. The `batch_index` is strictly monotonically increasing.
//! 3. The `run_uuid` matches the active session.

use crate::HashDigest;
use alloc::string::String;
use alloc::vec::Vec;

/// A single telemetry batch — the atomic record in the RML chain.
///
/// This struct represents the *logical* batch before Bencode serialization.
/// The encoder converts this into the canonical binary form `Bencode(B_n)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryBatch {
    /// Globally unique run identifier (UUID v4).
    pub run_uuid: String,

    /// Monotonically increasing batch index within the run.
    /// Invariant: `batch_index(B_n) = batch_index(B_{n-1}) + 1`.
    pub batch_index: u64,

    /// TAI64N timestamp of batch creation (nanosecond precision).
    /// Invariant: `timestamp(B_n) > timestamp(B_{n-1})`.
    pub timestamp_tai64n: [u8; 12],

    /// The agent's action taken at this step (encoded as bytes).
    pub action: Vec<u8>,

    /// Post-P14 projected reward: `r̂ = ⌊r × 10^14 + sgn(r) × 0.5⌋`.
    /// Stored as `i128` to guarantee cross-architecture bit-identity.
    pub reward_p14: i128,

    /// Serialized observation/state vector (opaque bytes).
    /// The RCO protocol does not interpret this; it only hashes it.
    pub observation: Vec<u8>,

    /// Additional key-value metadata (must be Bencode-serializable).
    pub metadata: Vec<(String, Vec<u8>)>,
}

/// A verified batch with its computed lineage anchor.
///
/// This is the output of the ingestion pipeline — a batch that has been
/// serialized, hashed, and chained into the RML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchoredBatch {
    /// The original telemetry batch.
    pub batch: TelemetryBatch,

    /// The Keccak-256 hash of the Bencoded batch: `H(Bencode(B_n))`.
    pub content_hash: HashDigest,

    /// The RML lineage anchor: `L_n = Keccak-256(Bencode(B_n) || L_{n-1})`.
    pub lineage_anchor: HashDigest,

    /// Size of the Bencoded representation in bytes.
    pub encoded_size: usize,
}
