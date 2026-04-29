//! # Hardware Random Number Generator (HRNG) Interface
//!
//! Wraps the OS-provided cryptographic random source (`getrandom()`)
//! which in turn uses RDRAND/RDSEED on x86 or `/dev/urandom` on Linux.
//!
//! This module provides the first entropy source for the whitening gate.

use rco_types::error::RcoError;

/// Fills a buffer with hardware-sourced random bytes.
///
/// On Linux, this calls `getrandom(2)` → `/dev/urandom` → RDRAND/RDSEED.
/// On failure, returns `RcoError::InsufficientEntropy`.
///
/// # Errors
///
/// Returns `RcoError::InsufficientEntropy` if the OS RNG fails or is unavailable.
pub fn fill_hardware_entropy(buf: &mut [u8]) -> Result<(), RcoError> {
    getrandom::fill(buf).map_err(|_| RcoError::InsufficientEntropy {
        measured_millibits: 0,
    })
}

/// Generates a 32-byte hardware entropy vector.
///
/// # Errors
///
/// Returns `RcoError::InsufficientEntropy` if the OS RNG fails.
pub fn generate_hwrng_32() -> Result<[u8; 32], RcoError> {
    let mut buf = [0u8; 32];
    fill_hardware_entropy(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hwrng_produces_bytes() {
        let buf = generate_hwrng_32().unwrap();
        // High probability that all-zeros won't be produced
        assert_ne!(buf, [0u8; 32], "HRNG produced all zeros (entropy failure)");
    }

    #[test]
    fn test_hwrng_produces_unique() {
        let a = generate_hwrng_32().unwrap();
        let b = generate_hwrng_32().unwrap();
        assert_ne!(a, b, "Two HRNG calls should produce different output");
    }

    #[test]
    fn test_fill_partial_buffer() {
        let mut buf = [0u8; 8];
        fill_hardware_entropy(&mut buf).unwrap();
        assert_ne!(buf, [0u8; 8]);
    }
}
