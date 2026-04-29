//! # Genesis Block Definition
//!
//! The genesis block `B_0` is the non-negotiable anchor for every simulation
//! run. Its construction is governed by Phase 4 of Stage-I, requiring:
//!
//! - A UUID v4 for global uniqueness.
//! - A TAI64N timestamp for temporal grounding.
//! - A 256-bit entropy vector from the Multi-Source Whitening Gate.
//! - The protocol version for handshake validation.
//!
//! ## Security Model
//!
//! The genesis block's entropy must satisfy:
//! `H(L_0[i..i+31]) > 7.99 bits/byte` across any 32-byte window.
//! Failure triggers `InsufficientEntropy` (F-04).

use crate::{HashDigest, PROTOCOL_VERSION};
use alloc::string::String;
use zeroize::Zeroize;

/// The genesis block — root anchor for the Recursive Merkle-Lineage.
///
/// This block is created exactly once per simulation run and defines `L_0`.
/// All subsequent lineage anchors `L_n` recursively depend on this root.
#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
#[zeroize(drop)]
pub struct GenesisBlock {
    /// UUID v4 identifying this unique simulation run.
    pub run_uuid: String,

    /// TAI64N timestamp of simulation initialization.
    /// 8 bytes TAI64 + 4 bytes nanosecond fraction = 12 bytes.
    pub timestamp_genesis: [u8; 12],

    /// 256-bit entropy vector from the Hardware-Rooted Whitening Gate.
    /// Source: `RDRAND ⊕ E_jitter`.
    pub entropy_vector: [u8; 32],

    /// Protocol version string (must match `PROTOCOL_VERSION`).
    pub protocol_version: String,
}

impl GenesisBlock {
    /// Creates a new genesis block with the current protocol version.
    ///
    /// # Arguments
    ///
    /// * `run_uuid` — A UUID v4 string for this run.
    /// * `timestamp` — TAI64N timestamp bytes.
    /// * `entropy` — 256-bit entropy from the whitening gate.
    #[must_use]
    pub fn new(run_uuid: String, timestamp: [u8; 12], entropy: [u8; 32]) -> Self {
        Self {
            run_uuid,
            timestamp_genesis: timestamp,
            entropy_vector: entropy,
            protocol_version: String::from(PROTOCOL_VERSION),
        }
    }

    /// Returns a reference to the entropy vector for validation.
    #[must_use]
    pub fn entropy(&self) -> &[u8; 32] {
        &self.entropy_vector
    }
}

/// The computed genesis root — the `L_0` value that seeds the RML chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenesisRoot {
    /// The genesis block that produced this root.
    pub block: GenesisBlock,

    /// `L_0 = Keccak-256(Bencode(B_0))` — the chain's initial anchor.
    pub root_hash: HashDigest,
}
