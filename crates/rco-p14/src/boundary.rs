//! # Reward Boundary Validation
//!
//! Pre-projection validation layer that classifies incoming reward values
//! and applies defensive guards before they enter the P14 pipeline.
//!
//! ## Classification Taxonomy
//!
//! | Class | Range | Action |
//! |---|---|---|
//! | `Normal` | \|r\| ≤ 10⁴ | Project normally |
//! | `Subnormal` | \|r\| < 10⁻³⁰⁸ | Flush to zero |
//! | `NaN` | — | Reject (F-06) |
//! | `Infinite` | — | Reject (F-06) |
//! | `Overflow` | \|r\| > 9.2×10⁴ | Reject (F-06) |
//!
//! ## Design Rationale
//!
//! This module exists separately from `projection.rs` to provide a clean
//! validation boundary. Callers can validate a batch of rewards without
//! paying the cost of projection, useful for pre-flight checks on the
//! simulation SDK side.

use rco_types::error::RcoError;

/// Maximum safe absolute reward value for P14 projection.
///
/// Derived from: `i128::MAX / 10^14 ≈ 1.7 × 10²⁴`, but the patent
/// specifies a dynamic range of ±10⁴. We use 9.2×10⁴ as the hard
/// engineering limit to leave margin for accumulated arithmetic.
pub const MAX_SAFE_REWARD: f64 = 9.2e4;

/// Minimum non-zero absolute value that survives P14 projection.
///
/// Values smaller than `0.5 × 10⁻¹⁴` round to zero.
pub const MIN_NONZERO_REWARD: f64 = 0.5e-14;

/// Classification of a reward value for P14 processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewardClass {
    /// Normal value within the projectable range.
    Normal,

    /// Value is exactly (or effectively) zero.
    Zero,

    /// Subnormal IEEE-754 value — will be flushed to zero.
    Subnormal,

    /// NaN — invalid, must be rejected.
    NaN,

    /// Infinite — invalid, must be rejected.
    Infinite,

    /// Exceeds the P14 representable range — must be rejected.
    Overflow,
}

/// Classifies a reward value for P14 processing.
///
/// This is a pure function with no side effects — it does not perform
/// the projection, only determines whether it would succeed.
#[must_use]
#[inline]
pub fn classify_reward(r: f64) -> RewardClass {
    if r.is_nan() {
        RewardClass::NaN
    } else if r.is_infinite() {
        RewardClass::Infinite
    } else if r == 0.0 || r == -0.0 {
        RewardClass::Zero
    } else if r.is_subnormal() {
        RewardClass::Subnormal
    } else if r.abs() > MAX_SAFE_REWARD {
        RewardClass::Overflow
    } else {
        RewardClass::Normal
    }
}

/// Validates a reward value, returning `Ok(())` if it is projectable.
///
/// This performs the same checks as `project_p14` but without computing
/// the projection. Useful for pre-flight batch validation.
///
/// # Errors
///
/// - `RcoError::RewardNaN` — value is NaN.
/// - `RcoError::RewardOverflow` — value exceeds the P14 range.
/// - `RcoError::NumericalIntegrityFault` — value is infinite.
pub fn validate_reward(r: f64) -> Result<(), RcoError> {
    match classify_reward(r) {
        RewardClass::Normal | RewardClass::Zero | RewardClass::Subnormal => Ok(()),
        RewardClass::NaN => Err(RcoError::RewardNaN),
        RewardClass::Infinite => Err(RcoError::RewardOverflow { bits: r.to_bits() }),
        RewardClass::Overflow => Err(RcoError::RewardOverflow { bits: r.to_bits() }),
    }
}

/// Validates a batch of rewards, returning the index of the first invalid value.
///
/// This enables early rejection of entire batches without projecting all values.
///
/// # Errors
///
/// Returns `Err((index, error))` for the first invalid reward encountered.
pub fn validate_reward_batch(rewards: &[f64]) -> Result<(), (usize, RcoError)> {
    for (i, &r) in rewards.iter().enumerate() {
        validate_reward(r).map_err(|e| (i, e))?;
    }
    Ok(())
}

/// Flushes subnormal values to zero, passing all others unchanged.
///
/// Subnormal IEEE-754 values have reduced precision and can cause
/// performance degradation on some hardware due to microcode assists.
/// This function ensures they are eliminated before entering the
/// projection pipeline.
#[must_use]
#[inline]
pub fn flush_subnormals(r: f64) -> f64 {
    if r.is_subnormal() {
        0.0
    } else {
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_normal() {
        assert_eq!(classify_reward(1.0), RewardClass::Normal);
        assert_eq!(classify_reward(-42.0), RewardClass::Normal);
        assert_eq!(classify_reward(0.001), RewardClass::Normal);
    }

    #[test]
    fn test_classify_zero() {
        assert_eq!(classify_reward(0.0), RewardClass::Zero);
        assert_eq!(classify_reward(-0.0), RewardClass::Zero);
    }

    #[test]
    fn test_classify_nan() {
        assert_eq!(classify_reward(f64::NAN), RewardClass::NaN);
    }

    #[test]
    fn test_classify_infinite() {
        assert_eq!(classify_reward(f64::INFINITY), RewardClass::Infinite);
        assert_eq!(classify_reward(f64::NEG_INFINITY), RewardClass::Infinite);
    }

    #[test]
    fn test_classify_overflow() {
        assert_eq!(classify_reward(1e5), RewardClass::Overflow);
        assert_eq!(classify_reward(-1e5), RewardClass::Overflow);
    }

    #[test]
    fn test_classify_subnormal() {
        // Smallest positive subnormal: 5 × 10⁻³²⁴
        assert_eq!(classify_reward(5e-324), RewardClass::Subnormal);
    }

    #[test]
    fn test_validate_normal() {
        assert!(validate_reward(1.0).is_ok());
        assert!(validate_reward(0.0).is_ok());
        assert!(validate_reward(-42.0).is_ok());
    }

    #[test]
    fn test_validate_rejects_nan() {
        assert!(matches!(
            validate_reward(f64::NAN),
            Err(RcoError::RewardNaN)
        ));
    }

    #[test]
    fn test_validate_rejects_overflow() {
        assert!(matches!(
            validate_reward(1e5),
            Err(RcoError::RewardOverflow { .. })
        ));
    }

    #[test]
    fn test_validate_batch_all_valid() {
        let rewards = [0.1, 0.2, 0.3, -0.5, 1.0];
        assert!(validate_reward_batch(&rewards).is_ok());
    }

    #[test]
    fn test_validate_batch_first_invalid() {
        let rewards = [0.1, f64::NAN, 0.3];
        let err = validate_reward_batch(&rewards).unwrap_err();
        assert_eq!(err.0, 1); // Index of NaN
    }

    #[test]
    fn test_flush_subnormals_normal() {
        assert_eq!(flush_subnormals(1.0), 1.0);
    }

    #[test]
    fn test_flush_subnormals_subnormal() {
        assert_eq!(flush_subnormals(5e-324), 0.0);
    }

    #[test]
    fn test_flush_subnormals_zero() {
        assert_eq!(flush_subnormals(0.0).to_bits(), 0.0_f64.to_bits());
    }
}
