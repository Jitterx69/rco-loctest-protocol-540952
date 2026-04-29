//! # SIMD Acceleration Kernels
//! 
//! Provides optimized hashing kernels for the audit sweep.
//! Current implementation leverages the `sha3` crate's `asm` features
//! which use AVX2/AVX-512 on x86_64.

/// Checks if the current hardware supports the required SIMD features
/// for peak audit performance.
#[must_use]
pub fn check_simd_support() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        is_x86_feature_detected!("avx2")
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}
