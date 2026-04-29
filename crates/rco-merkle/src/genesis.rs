//! # Genesis Root Computation
//!
//! Computes the initial lineage anchor `L_0` from the genesis block.
//!
//! ```text
//! L_0 = Keccak-256(Bencode(B_0))
//! ```
//!
//! Note: The genesis root has NO predecessor — it is computed from the
//! genesis block alone, without any `prev_hash` concatenation.

use crate::chain::compute_hash;
use rco_types::HashDigest;

/// Computes the genesis root `L_0 = Keccak-256(encoded_genesis)`.
///
/// # Arguments
///
/// * `encoded_genesis` — The canonical Bencoded representation of the genesis block.
///
/// # Returns
///
/// The 32-byte Keccak-256 digest that will serve as `L_0`, the root
/// of the Recursive Merkle-Lineage chain.
///
/// # Example (Conceptual)
///
/// ```ignore
/// let genesis_block = GenesisBlock::new(uuid, timestamp, entropy);
/// let encoded = bencode::encode_to_vec(&genesis_as_bencode)?;
/// let L_0 = compute_genesis_root(&encoded);
/// let chain = RmlChain::from_genesis(L_0);
/// ```
#[must_use]
pub fn compute_genesis_root(encoded_genesis: &[u8]) -> HashDigest {
    compute_hash(encoded_genesis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::RmlChain;

    #[test]
    fn test_genesis_root_deterministic() {
        let data = b"d7:entropyX(some_entropy_bytes)8:run_uuid36:550e8400-e29b-41d4-a716-4466554400007:versione";
        let a = compute_genesis_root(data);
        let b = compute_genesis_root(data);
        assert_eq!(a, b, "Genesis root must be deterministic");
    }

    #[test]
    fn test_genesis_root_differs_by_content() {
        let a = compute_genesis_root(b"genesis_A");
        let b = compute_genesis_root(b"genesis_B");
        assert_ne!(a, b);
    }

    #[test]
    fn test_genesis_root_seeds_chain() {
        let root = compute_genesis_root(b"test_genesis");
        let chain = RmlChain::from_genesis(root);
        assert_eq!(chain.head_hash(), &root);
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_genesis_not_all_zeros() {
        let root = compute_genesis_root(b"any_genesis_data");
        assert_ne!(root, [0u8; 32], "Genesis root must not be all zeros");
    }
}
