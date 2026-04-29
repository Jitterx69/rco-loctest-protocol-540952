//! Persistence Witness ($\Omega$)
//!
//! Compact proof of holographic integrity.

use rco_types::HashDigest;
use sha3::{Digest, Keccak256};

/// A compact Persistence Witness ($\Omega$).
/// Must remain under 1024 bytes. We represent it by aggregating the hashes of the samples.
#[derive(Clone, Debug)]
pub struct PersistenceWitness {
    /// The aggregated hash root.
    pub aggregate_root: HashDigest,
    /// The number of samples folded into this witness.
    pub sample_count: usize,
}

impl PersistenceWitness {
    /// Creates a new empty witness.
    pub fn new() -> Self {
        Self {
            aggregate_root: [0u8; 32],
            sample_count: 0,
        }
    }

    /// Folds a sample root into the witness.
    pub fn accumulate(&mut self, sample_root: HashDigest) {
        let mut hasher = Keccak256::new();
        hasher.update(self.aggregate_root);
        hasher.update(sample_root);
        self.aggregate_root = hasher.finalize().into();
        self.sample_count += 1;
    }

    /// Returns the byte representation of the witness.
    /// It's a 32-byte hash plus an 8-byte count, easily fitting in <1024 bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(40);
        bytes.extend_from_slice(&self.aggregate_root);
        bytes.extend_from_slice(&self.sample_count.to_le_bytes());
        bytes
    }
}

impl Default for PersistenceWitness {
    fn default() -> Self {
        Self::new()
    }
}
