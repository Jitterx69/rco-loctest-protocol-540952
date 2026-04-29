//! # Chain Verification
//!
//! Audit-grade verification of RML chains and individual anchors.
//!
//! ## Single Anchor Verification
//!
//! Given `(B_n, L_{n-1})`, verify that `L_n = Keccak-256(Bencode(B_n) ‖ L_{n-1})`.
//!
//! ## Segment Verification
//!
//! Re-compute an entire chain segment from raw batches and compare
//! against stored anchors. This is the foundation of the Audit SDK.

use crate::chain::compute_chained_hash;
use rco_types::HashDigest;
use rco_types::error::RcoError;

/// Verifies a single anchor against its batch data and predecessor.
///
/// Recomputes `L_n = Keccak-256(encoded_batch ‖ prev_hash)` and checks
/// it matches the claimed `anchor_hash`.
///
/// # Returns
///
/// - `Ok(true)` if the anchor is valid.
/// - `Ok(false)` if the computed hash doesn't match (tampering detected).
#[must_use]
pub fn verify_anchor(
    encoded_batch: &[u8],
    prev_hash: &HashDigest,
    claimed_hash: &HashDigest,
) -> bool {
    let computed = compute_chained_hash(encoded_batch, prev_hash);
    constant_time_eq(&computed, claimed_hash)
}

/// Verifies a contiguous segment of the RML chain.
///
/// Takes a series of `(encoded_batch, claimed_anchor_hash)` pairs and
/// verifies each one sequentially, threading the previous anchor through.
///
/// # Arguments
///
/// * `genesis_hash` — The `L_0` root to start verification from.
/// * `batches` — Sequence of `(encoded_batch_bytes, claimed_L_n)` pairs.
///
/// # Returns
///
/// - `Ok(())` if all anchors verify correctly.
/// - `Err(RcoError::AuditHashMismatch { batch_index })` at the first mismatch.
pub fn verify_chain_segment(
    genesis_hash: &HashDigest,
    batches: &[(&[u8], &HashDigest)],
) -> Result<(), RcoError> {
    let mut prev_hash = *genesis_hash;

    for (i, (encoded_batch, claimed_hash)) in batches.iter().enumerate() {
        if !verify_anchor(encoded_batch, &prev_hash, claimed_hash) {
            return Err(RcoError::AuditHashMismatch {
                batch_index: (i + 1) as u64,
            });
        }
        prev_hash = **claimed_hash;
    }

    Ok(())
}

/// Constant-time byte comparison to prevent timing side-channels.
///
/// An auditor should not be able to determine *which* byte of a hash
/// mismatched by observing comparison latency.
#[inline]
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut acc: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        acc |= x ^ y;
    }
    acc == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::{RmlChain, compute_chained_hash, compute_hash};
    use alloc::vec::Vec;

    // ── Single Anchor Verification ──────────────────────────────────

    #[test]
    fn test_verify_valid_anchor() {
        let prev = compute_hash(b"genesis");
        let batch = b"batch_1_data";
        let expected = compute_chained_hash(batch, &prev);

        assert!(verify_anchor(batch, &prev, &expected));
    }

    #[test]
    fn test_verify_invalid_anchor() {
        let prev = compute_hash(b"genesis");
        let batch = b"batch_1_data";
        let wrong_hash = [0xFFu8; 32];

        assert!(!verify_anchor(batch, &prev, &wrong_hash));
    }

    #[test]
    fn test_verify_tampered_batch() {
        let prev = compute_hash(b"genesis");
        let original = b"original_batch";
        let original_hash = compute_chained_hash(original, &prev);

        // Try to verify with tampered data but original hash
        assert!(!verify_anchor(b"tampered_batch", &prev, &original_hash));
    }

    #[test]
    fn test_verify_wrong_predecessor() {
        let correct_prev = compute_hash(b"correct_genesis");
        let wrong_prev = compute_hash(b"wrong_genesis");
        let batch = b"batch_data";
        let hash = compute_chained_hash(batch, &correct_prev);

        // Correct predecessor works
        assert!(verify_anchor(batch, &correct_prev, &hash));
        // Wrong predecessor fails
        assert!(!verify_anchor(batch, &wrong_prev, &hash));
    }

    // ── Chain Segment Verification ──────────────────────────────────

    #[test]
    fn test_verify_valid_segment() {
        let genesis = compute_hash(b"genesis");
        let mut chain = RmlChain::from_genesis(genesis);

        let a1 = chain.extend(1, b"batch_1").unwrap();
        let a2 = chain.extend(2, b"batch_2").unwrap();
        let a3 = chain.extend(3, b"batch_3").unwrap();

        let batches: Vec<(&[u8], &HashDigest)> = alloc::vec![
            (b"batch_1" as &[u8], &a1.hash),
            (b"batch_2" as &[u8], &a2.hash),
            (b"batch_3" as &[u8], &a3.hash),
        ];

        assert!(verify_chain_segment(&genesis, &batches).is_ok());
    }

    #[test]
    fn test_verify_segment_detects_tamper() {
        let genesis = compute_hash(b"genesis");
        let mut chain = RmlChain::from_genesis(genesis);

        let a1 = chain.extend(1, b"batch_1").unwrap();
        let a2 = chain.extend(2, b"batch_2").unwrap();
        let a3 = chain.extend(3, b"batch_3").unwrap();

        // Tamper batch 2 data but keep original hash
        let batches: Vec<(&[u8], &HashDigest)> = alloc::vec![
            (b"batch_1" as &[u8], &a1.hash),
            (b"TAMPERED" as &[u8], &a2.hash), // ← tampered
            (b"batch_3" as &[u8], &a3.hash),
        ];

        let result = verify_chain_segment(&genesis, &batches);
        assert!(matches!(
            result,
            Err(RcoError::AuditHashMismatch { batch_index: 2 })
        ));
    }

    #[test]
    fn test_verify_empty_segment() {
        let genesis = compute_hash(b"genesis");
        assert!(verify_chain_segment(&genesis, &[]).is_ok());
    }

    // ── Constant-Time Comparison ────────────────────────────────────

    #[test]
    fn test_constant_time_eq_equal() {
        let a = [0x42u8; 32];
        assert!(constant_time_eq(&a, &a));
    }

    #[test]
    fn test_constant_time_eq_different() {
        let a = [0x42u8; 32];
        let b = [0x43u8; 32];
        assert!(!constant_time_eq(&a, &b));
    }

    #[test]
    fn test_constant_time_eq_different_lengths() {
        let a = [0x42u8; 32];
        let b = [0x42u8; 16];
        assert!(!constant_time_eq(&a, &b));
    }
}
