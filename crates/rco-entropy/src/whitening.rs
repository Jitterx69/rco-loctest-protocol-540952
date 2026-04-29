//! # Multi-Source Whitening Gate
//!
//! Combines multiple entropy sources through XOR whitening and Keccak-256
//! conditioning to produce the genesis block's `entropy_vector`.
//!
//! ## Algorithm
//!
//! ```text
//! E_genesis = Keccak-256(HRNG ⊕ E_jitter ⊕ H(auxiliary))
//! ```
//!
//! The final Keccak-256 pass ensures:
//! 1. Uniform distribution (even if individual sources have bias).
//! 2. Fixed output size (32 bytes).
//! 3. Forward secrecy (the output cannot be used to recover inputs).

use rco_types::error::RcoError;
use sha3::{Digest, Keccak256};

use crate::hwrng;
use crate::jitter;

/// Generates the 32-byte genesis entropy vector.
///
/// Implements the Multi-Source Whitening Gate:
/// ```text
/// E = Keccak-256(HRNG ⊕ E_jitter ⊕ H(aux))
/// ```
///
/// # Arguments
///
/// * `auxiliary_entropy` — Optional additional entropy (e.g., user-provided salt,
///   TPM-derived nonce, or process PID + hostname hash). Can be empty.
///
/// # Returns
///
/// A 32-byte entropy vector suitable for the genesis block.
///
/// # Errors
///
/// Returns `RcoError::InsufficientEntropy` if the hardware RNG fails.
pub fn generate_genesis_entropy(auxiliary_entropy: &[u8]) -> Result<[u8; 32], RcoError> {
    // ── Source 1: Hardware RNG ────────────────────────────────────
    let hrng = hwrng::generate_hwrng_32()?;

    // ── Source 2: CPU Jitter ─────────────────────────────────────
    let jitter = jitter::collect_jitter_entropy();

    // ── Source 3: Auxiliary (hashed to 32 bytes) ──────────────────
    let aux_hash = if auxiliary_entropy.is_empty() {
        [0u8; 32]
    } else {
        let mut hasher = Keccak256::new();
        hasher.update(auxiliary_entropy);
        let result = hasher.finalize();
        let mut h = [0u8; 32];
        h.copy_from_slice(&result);
        h
    };

    // ── XOR Whitening ────────────────────────────────────────────
    let mut xored = [0u8; 32];
    for i in 0..32 {
        xored[i] = hrng[i] ^ jitter[i] ^ aux_hash[i];
    }

    // ── Final Conditioning Pass ──────────────────────────────────
    let mut hasher = Keccak256::new();
    hasher.update(xored);
    // Also feed raw sources for additional mixing
    hasher.update(&hrng);
    hasher.update(&jitter);
    hasher.update(auxiliary_entropy);
    let result = hasher.finalize();

    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    Ok(output)
}

/// Generates genesis entropy from a deterministic seed (for testing only).
///
/// # Warning
///
/// This function produces **deterministic** output and MUST NOT be used
/// in production. It exists solely for reproducible test cases.
#[cfg(test)]
pub fn generate_deterministic_entropy(seed: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(seed);
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_genesis_entropy() {
        let entropy = generate_genesis_entropy(b"test_salt").unwrap();
        assert_ne!(entropy, [0u8; 32], "Genesis entropy must not be all zeros");
    }

    #[test]
    fn test_generate_genesis_entropy_no_aux() {
        let entropy = generate_genesis_entropy(b"").unwrap();
        assert_ne!(entropy, [0u8; 32]);
    }

    #[test]
    fn test_genesis_entropy_unique() {
        let a = generate_genesis_entropy(b"salt_a").unwrap();
        let b = generate_genesis_entropy(b"salt_b").unwrap();
        assert_ne!(a, b, "Different salts should produce different entropy");
    }

    #[test]
    fn test_genesis_entropy_same_salt_differs() {
        // Even with the same salt, HRNG + jitter should produce different output
        let a = generate_genesis_entropy(b"same_salt").unwrap();
        let b = generate_genesis_entropy(b"same_salt").unwrap();
        assert_ne!(
            a, b,
            "Same salt with different HRNG/jitter should produce different entropy"
        );
    }

    #[test]
    fn test_deterministic_entropy() {
        let a = generate_deterministic_entropy(b"seed");
        let b = generate_deterministic_entropy(b"seed");
        assert_eq!(a, b, "Deterministic entropy must be reproducible");
    }

    #[test]
    fn test_deterministic_entropy_different_seeds() {
        let a = generate_deterministic_entropy(b"seed_a");
        let b = generate_deterministic_entropy(b"seed_b");
        assert_ne!(a, b);
    }
}
