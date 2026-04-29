//! # Lexicographic Key Sort
//!
//! Utility functions for enforcing the Lexicographic Sorting Invariant on
//! Bencode dictionary keys. This module provides both a validation function
//! (for checking existing order) and a sort function (for constructing
//! correctly ordered dictionaries from unordered input).
//!
//! ## Mathematical Invariant
//!
//! For an RCO-Dictionary D with keys k_1, k_2, …, k_n:
//!
//! ```text
//! k_1 <_lex k_2 <_lex … <_lex k_n
//! ```
//!
//! where `<_lex` denotes byte-by-byte comparison of raw string representations.

use rco_types::error::RcoError;

/// Validates that a slice of keys is in strict lexicographic order.
///
/// Returns `Ok(())` if the ordering is valid, or `Err(RcoError::KeyOrderViolation)`
/// if any adjacent pair violates the invariant.
///
/// This function also checks for duplicate keys (F-11).
///
/// # Complexity
///
/// `O(n × m)` where `n` is the number of keys and `m` is the average key length.
pub fn validate_key_order(keys: &[&[u8]]) -> Result<(), RcoError> {
    for window in keys.windows(2) {
        match window[0].cmp(window[1]) {
            core::cmp::Ordering::Less => continue,
            core::cmp::Ordering::Equal => return Err(RcoError::DuplicateKey),
            core::cmp::Ordering::Greater => return Err(RcoError::KeyOrderViolation),
        }
    }
    Ok(())
}

/// Sorts key-value pairs in-place by lexicographic key order.
///
/// This is used during dictionary construction to ensure the output conforms
/// to `G_RCO`. The sort is **stable** — equal keys (which would indicate a
/// duplicate) preserve their relative order for detection.
///
/// # Panics
///
/// Does not panic. Duplicate key detection is deferred to `validate_key_order`.
pub fn sort_keys_lexicographic<V>(entries: &mut [(alloc::vec::Vec<u8>, V)]) {
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));
}

/// Checks if a byte slice `a` is strictly less than `b` in lexicographic order.
///
/// This is the raw comparator used throughout the Bencode engine.
#[inline(always)]
#[must_use]
pub fn is_lex_less(a: &[u8], b: &[u8]) -> bool {
    a < b
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_valid_order() {
        let keys: &[&[u8]] = &[b"a", b"b", b"c", b"z"];
        assert!(validate_key_order(keys).is_ok());
    }

    #[test]
    fn test_invalid_order() {
        let keys: &[&[u8]] = &[b"b", b"a"];
        assert_eq!(validate_key_order(keys), Err(RcoError::KeyOrderViolation));
    }

    #[test]
    fn test_duplicate_keys() {
        let keys: &[&[u8]] = &[b"a", b"a"];
        assert_eq!(validate_key_order(keys), Err(RcoError::DuplicateKey));
    }

    #[test]
    fn test_empty_keys() {
        let keys: &[&[u8]] = &[];
        assert!(validate_key_order(keys).is_ok());
    }

    #[test]
    fn test_single_key() {
        let keys: &[&[u8]] = &[b"only"];
        assert!(validate_key_order(keys).is_ok());
    }

    #[test]
    fn test_byte_level_comparison() {
        // "a" (0x61) < "b" (0x62) — correct
        let keys: &[&[u8]] = &[b"a", b"b"];
        assert!(validate_key_order(keys).is_ok());

        // Prefix comparison: "ab" < "abc"
        let keys: &[&[u8]] = &[b"ab", b"abc"];
        assert!(validate_key_order(keys).is_ok());
    }

    #[test]
    fn test_sort_produces_valid_order() {
        let mut entries: Vec<(Vec<u8>, i32)> = alloc::vec![
            (b"zebra".to_vec(), 3),
            (b"alpha".to_vec(), 1),
            (b"beta".to_vec(), 2),
        ];
        sort_keys_lexicographic(&mut entries);
        assert_eq!(entries[0].0, b"alpha");
        assert_eq!(entries[1].0, b"beta");
        assert_eq!(entries[2].0, b"zebra");
    }
}
