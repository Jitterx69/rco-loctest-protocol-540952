//! # RML Chain Engine
//!
//! Core recursive hash-chaining logic for the Recursive Merkle-Lineage.
//!
//! ## Recurrence
//!
//! ```text
//! L_n = Keccak-256(Bencode(B_n) ‖ L_{n-1})
//! ```
//!
//! The concatenation operator `‖` is a simple byte-append: the serialized
//! batch followed immediately by the 32-byte previous anchor. This is
//! unambiguous because the anchor is always exactly 32 bytes.

use rco_types::error::RcoError;
use rco_types::{HashDigest, HASH_SIZE};
use sha3::{Digest, Keccak256};

/// A single anchor in the Recursive Merkle-Lineage.
///
/// Each anchor records the hash, batch index, and a reference to the
/// previous anchor's hash, forming a causal chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RmlAnchor {
    /// The batch index this anchor corresponds to.
    pub batch_index: u64,

    /// `L_n = Keccak-256(Bencode(B_n) ‖ L_{n-1})`
    pub hash: HashDigest,

    /// `L_{n-1}` — the previous anchor's hash.
    /// For genesis (n=0), this is all zeros.
    pub prev_hash: HashDigest,
}

/// Stateful RML chain that tracks the head anchor and supports extension.
///
/// This is the primary interface for building lineage chains during
/// simulation ingestion.
#[derive(Debug, Clone)]
pub struct RmlChain {
    /// The current head anchor (most recently committed).
    head: RmlAnchor,

    /// Total number of anchors in the chain (including genesis).
    len: u64,
}

impl RmlChain {
    /// Creates a new chain from a genesis root.
    ///
    /// The genesis root must be computed from the genesis block using
    /// `compute_genesis_root()`. The chain starts with `len = 1`.
    #[must_use]
    pub fn from_genesis(genesis_hash: HashDigest) -> Self {
        let anchor = RmlAnchor {
            batch_index: 0,
            hash: genesis_hash,
            prev_hash: [0u8; HASH_SIZE], // Genesis has no predecessor
        };
        Self {
            head: anchor,
            len: 1,
        }
    }

    /// Returns the current head anchor.
    #[must_use]
    pub fn head(&self) -> &RmlAnchor {
        &self.head
    }

    /// Returns the current head hash (`L_n`).
    #[must_use]
    pub fn head_hash(&self) -> &HashDigest {
        &self.head.hash
    }

    /// Returns the total number of anchors in the chain.
    #[must_use]
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Returns `true` if the chain is empty (should never happen after genesis).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the expected next batch index.
    #[must_use]
    pub fn next_batch_index(&self) -> u64 {
        self.head.batch_index + 1
    }

    /// Extends the chain with a new batch.
    ///
    /// Computes: `L_{n+1} = Keccak-256(encoded_batch ‖ L_n)`
    ///
    /// # Arguments
    ///
    /// * `batch_index` — Must equal `self.next_batch_index()`.
    /// * `encoded_batch` — The canonical Bencoded representation of the batch.
    ///
    /// # Returns
    ///
    /// The newly computed `RmlAnchor` on success.
    ///
    /// # Errors
    ///
    /// - `RcoError::LinkageContinuityGap` if `batch_index` is not sequential.
    pub fn extend(
        &mut self,
        batch_index: u64,
        encoded_batch: &[u8],
    ) -> Result<RmlAnchor, RcoError> {
        // ── Monotonicity Guard ────────────────────────────────────
        let expected = self.next_batch_index();
        if batch_index != expected {
            return Err(RcoError::LinkageContinuityGap {
                expected_index: expected,
                received_index: batch_index,
            });
        }

        // ── Compute L_{n+1} = Keccak-256(Bencode(B_{n+1}) ‖ L_n) ──
        let new_hash = compute_chained_hash(encoded_batch, &self.head.hash);

        let anchor = RmlAnchor {
            batch_index,
            hash: new_hash,
            prev_hash: self.head.hash,
        };

        self.head = anchor.clone();
        self.len += 1;

        Ok(anchor)
    }
}

/// Computes the chained hash: `Keccak-256(data ‖ prev_hash)`.
///
/// This is the atomic operation at the core of the RML recurrence.
/// Exposed publicly for use in audit verification paths.
#[must_use]
pub fn compute_chained_hash(data: &[u8], prev_hash: &HashDigest) -> HashDigest {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.update(prev_hash);
    let result = hasher.finalize();

    let mut hash = [0u8; HASH_SIZE];
    hash.copy_from_slice(&result);
    hash
}

/// Computes a standalone hash: `Keccak-256(data)`.
///
/// Used for genesis root computation and content hashing.
#[must_use]
pub fn compute_hash(data: &[u8]) -> HashDigest {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();

    let mut hash = [0u8; HASH_SIZE];
    hash.copy_from_slice(&result);
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Genesis Construction ────────────────────────────────────────

    #[test]
    fn test_chain_from_genesis() {
        let genesis_hash = compute_hash(b"genesis_block_data");
        let chain = RmlChain::from_genesis(genesis_hash);

        assert_eq!(chain.len(), 1);
        assert_eq!(chain.head().batch_index, 0);
        assert_eq!(chain.head_hash(), &genesis_hash);
        assert_eq!(chain.head().prev_hash, [0u8; HASH_SIZE]);
        assert_eq!(chain.next_batch_index(), 1);
    }

    // ── Chain Extension ─────────────────────────────────────────────

    #[test]
    fn test_extend_sequential() {
        let genesis_hash = compute_hash(b"genesis");
        let mut chain = RmlChain::from_genesis(genesis_hash);

        let anchor1 = chain.extend(1, b"batch_1_data").unwrap();
        assert_eq!(anchor1.batch_index, 1);
        assert_eq!(anchor1.prev_hash, genesis_hash);
        assert_eq!(chain.len(), 2);

        let anchor2 = chain.extend(2, b"batch_2_data").unwrap();
        assert_eq!(anchor2.batch_index, 2);
        assert_eq!(anchor2.prev_hash, anchor1.hash);
        assert_eq!(chain.len(), 3);
    }

    #[test]
    fn test_extend_rejects_non_sequential() {
        let genesis_hash = compute_hash(b"genesis");
        let mut chain = RmlChain::from_genesis(genesis_hash);

        // Try to extend with batch_index 5 instead of 1
        let result = chain.extend(5, b"batch_data");
        assert!(matches!(
            result,
            Err(RcoError::LinkageContinuityGap {
                expected_index: 1,
                received_index: 5,
            })
        ));
    }

    #[test]
    fn test_extend_rejects_duplicate_index() {
        let genesis_hash = compute_hash(b"genesis");
        let mut chain = RmlChain::from_genesis(genesis_hash);

        chain.extend(1, b"batch_1").unwrap();

        // Try to re-submit batch 1
        let result = chain.extend(1, b"batch_1_again");
        assert!(matches!(result, Err(RcoError::LinkageContinuityGap { .. })));
    }

    // ── Determinism ─────────────────────────────────────────────────

    #[test]
    fn test_chained_hash_deterministic() {
        let prev = [0xABu8; HASH_SIZE];
        let a = compute_chained_hash(b"data", &prev);
        let b = compute_chained_hash(b"data", &prev);
        assert_eq!(a, b, "Chained hash must be deterministic");
    }

    #[test]
    fn test_chain_deterministic_full_replay() {
        // Build chain A
        let genesis = compute_hash(b"genesis_entropy");
        let mut chain_a = RmlChain::from_genesis(genesis);
        chain_a.extend(1, b"batch_1").unwrap();
        chain_a.extend(2, b"batch_2").unwrap();
        chain_a.extend(3, b"batch_3").unwrap();

        // Build chain B with identical inputs
        let mut chain_b = RmlChain::from_genesis(genesis);
        chain_b.extend(1, b"batch_1").unwrap();
        chain_b.extend(2, b"batch_2").unwrap();
        chain_b.extend(3, b"batch_3").unwrap();

        assert_eq!(
            chain_a.head_hash(),
            chain_b.head_hash(),
            "Identical inputs must produce identical lineage"
        );
    }

    // ── Forward Integrity (Tamper Detection) ────────────────────────

    #[test]
    fn test_forward_integrity_tamper_detection() {
        // Build original chain
        let genesis = compute_hash(b"genesis");
        let mut original = RmlChain::from_genesis(genesis);
        original.extend(1, b"batch_1_original").unwrap();
        original.extend(2, b"batch_2").unwrap();
        original.extend(3, b"batch_3").unwrap();

        // Build tampered chain (batch 1 modified)
        let mut tampered = RmlChain::from_genesis(genesis);
        tampered.extend(1, b"batch_1_TAMPERED").unwrap();
        tampered.extend(2, b"batch_2").unwrap();
        tampered.extend(3, b"batch_3").unwrap();

        // The head hashes MUST differ — this is the forward integrity guarantee
        assert_ne!(
            original.head_hash(),
            tampered.head_hash(),
            "Tampered batch must produce different lineage"
        );
    }

    #[test]
    fn test_different_genesis_different_chain() {
        let mut chain_a = RmlChain::from_genesis(compute_hash(b"genesis_A"));
        let mut chain_b = RmlChain::from_genesis(compute_hash(b"genesis_B"));

        chain_a.extend(1, b"same_batch").unwrap();
        chain_b.extend(1, b"same_batch").unwrap();

        assert_ne!(
            chain_a.head_hash(),
            chain_b.head_hash(),
            "Different genesis must produce different lineage"
        );
    }

    // ── Hash Quality ────────────────────────────────────────────────

    #[test]
    fn test_hash_is_32_bytes() {
        let hash = compute_hash(b"test");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_hash_changes_with_input() {
        let a = compute_hash(b"input_a");
        let b = compute_hash(b"input_b");
        assert_ne!(a, b);
    }

    #[test]
    fn test_empty_input_hash() {
        let hash = compute_hash(b"");
        // Keccak-256("") is a well-known value
        assert_ne!(hash, [0u8; HASH_SIZE], "Empty hash must not be all zeros");
    }

    // ── Causal Chain Property ───────────────────────────────────────

    #[test]
    fn test_anchor_records_prev_hash() {
        let genesis = compute_hash(b"genesis");
        let mut chain = RmlChain::from_genesis(genesis);

        let a1 = chain.extend(1, b"b1").unwrap();
        let a2 = chain.extend(2, b"b2").unwrap();
        let a3 = chain.extend(3, b"b3").unwrap();

        // Each anchor's prev_hash must point to the previous anchor
        assert_eq!(a1.prev_hash, genesis);
        assert_eq!(a2.prev_hash, a1.hash);
        assert_eq!(a3.prev_hash, a2.hash);
    }
}
