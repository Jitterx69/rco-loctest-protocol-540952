//! Stretched Merkle Forest (SMF)
//!
//! Generates backward jumps for sub-linear traversal.

use rco_types::HashDigest;
use sha3::{Digest, Keccak256};
use std::collections::HashMap;

/// A node in the Stretched Merkle Forest.
#[derive(Clone, Debug)]
pub struct SmfNode {
    /// The batch index $n$.
    pub index: u64,
    /// The primary hash of the telemetry batch at this index.
    pub batch_hash: HashDigest,
    /// The composite root hash of this node.
    pub root_hash: HashDigest,
    /// The jump links back to previous root hashes ($n - 2^k$).
    pub jump_links: Vec<HashDigest>,
}

/// An in-memory store for SMF nodes.
pub struct SmfStore {
    nodes: HashMap<u64, SmfNode>,
}

impl SmfStore {
    /// Creates a new, empty store.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Adds a new batch to the SMF and computes its jump links and root hash.
    pub fn append(&mut self, index: u64, batch_hash: HashDigest) -> HashDigest {
        let mut jump_links = Vec::new();
        let mut hasher = Keccak256::new();
        hasher.update(batch_hash);

        // Compute jumps for k=0, 1, 2...
        let mut k = 0;
        while let Some(target_index) = index.checked_sub(1 << k) {
            if let Some(target_node) = self.nodes.get(&target_index) {
                jump_links.push(target_node.root_hash);
                hasher.update(target_node.root_hash);
            } else {
                // If it's missing from the store, we just skip.
                // In a real system, this would fetch from persistent storage.
            }
            k += 1;
        }

        let root_hash: HashDigest = hasher.finalize().into();

        let node = SmfNode {
            index,
            batch_hash,
            root_hash,
            jump_links,
        };

        self.nodes.insert(index, node);
        root_hash
    }

    /// Retrieves an SMF node by index.
    pub fn get(&self, index: u64) -> Option<&SmfNode> {
        self.nodes.get(&index)
    }
}

impl Default for SmfStore {
    fn default() -> Self {
        Self::new()
    }
}
