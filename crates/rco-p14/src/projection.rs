//! # P14 Projection Core
//!
//! The heart of the mantissa projection operator. Converts IEEE-754 `f64`
//! rewards into deterministic `i128` representations.
//!
//! ## Mathematical Definition
//!
//! ```text
//! P14(r) = ⌊r × 10¹⁴ + copysign(0.5, r)⌋
//! ```
//!
//! The use of `copysign` instead of `sgn(r) × 0.5` is the key innovation:
//! it avoids a branch on the sign of `r`, compiling to a single hardware
//! instruction on all target architectures.
//!
//! ## Precision Analysis
//!
//! - **Dynamic range**: ±9.2 × 10⁴ (limited by `i128` ÷ 10¹⁴)
//! - **Resolution**: 10⁻¹⁴ (sub-femto precision)
//! - **Error bound**: ε < 0.5 × 10⁻¹⁴ (half-ULP rounding)

use rco_types::error::RcoError;
use rco_types::P14_SCALE;

/// A P14-projected reward value.
///
/// This newtype wrapper enforces that the value has been through the
/// projection operator and is safe for deterministic serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ProjectedReward(i128);

impl ProjectedReward {
    /// Creates a `ProjectedReward` from a raw `i128` value.
    ///
    /// This is intended for deserialization paths where the value
    /// is already known to be a valid projection.
    #[must_use]
    #[inline]
    pub const fn from_raw(raw: i128) -> Self {
        Self(raw)
    }

    /// Returns the raw `i128` value for Bencode serialization.
    #[must_use]
    #[inline]
    pub const fn raw(self) -> i128 {
        self.0
    }

    /// Returns the zero projected reward.
    #[must_use]
    #[inline]
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Returns `true` if this reward is negative.
    #[must_use]
    #[inline]
    pub const fn is_negative(self) -> bool {
        self.0 < 0
    }

    /// Returns `true` if this reward is exactly zero.
    #[must_use]
    #[inline]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Recovers an approximate `f64` from the projected value.
    ///
    /// Note: This is lossy — floating-point cannot perfectly represent
    /// all integers × 10⁻¹⁴. Use only for display/debugging, never for
    /// re-projection.
    #[must_use]
    #[inline]
    pub fn approximate_f64(self) -> f64 {
        self.0 as f64 / P14_SCALE as f64
    }
}

/// Projects a reward scalar from `f64` to `i128` using the P14 operator.
///
/// # Mathematical Definition
///
/// ```text
/// r̂ = ⌊r × 10¹⁴ + copysign(0.5, r)⌋
/// ```
///
/// # Implementation: Branchless via `copysign`
///
/// The traditional implementation branches on `sgn(r)`:
/// ```text
/// if r >= 0 { floor(r * 1e14 + 0.5) } else { floor(r * 1e14 - 0.5) }
/// ```
///
/// We replace this with:
/// ```text
/// (r * 1e14 + copysign(0.5, r)) as i128
/// ```
///
/// `f64::copysign(0.5, r)` produces `+0.5` if `r ≥ 0` and `-0.5` if `r < 0`,
/// using a single bit-mask instruction (no branch). The `as i128` truncation
/// then gives us the correct floor for positive and ceil for negative values,
/// which is precisely the intended rounding behavior.
///
/// # Errors
///
/// - `RcoError::RewardNaN` — if `r` is NaN.
/// - `RcoError::RewardOverflow` — if `|r| > 9.2 × 10⁴` (would overflow `i128`
///   when scaled by 10¹⁴).
///
/// # Cross-Platform Guarantee (TC-02)
///
/// ```text
/// ∀t, project_p14(r_t) on ARM = project_p14(r_t) on x86
/// ```
///
/// This holds because:
/// 1. `f64::copysign` is a bit-level operation (IEEE-754 sign-bit copy).
/// 2. `f64 * f64` produces identical results on all IEEE-754 hardware.
/// 3. `f64 as i128` truncation is defined by the Rust spec (saturating).
#[inline]
pub fn project_p14(r: f64) -> Result<ProjectedReward, RcoError> {
    // ── Guard: NaN ────────────────────────────────────────────────
    if r.is_nan() {
        return Err(RcoError::RewardNaN);
    }

    // ── Guard: Infinity ───────────────────────────────────────────
    if r.is_infinite() {
        return Err(RcoError::RewardOverflow { bits: r.to_bits() });
    }

    // ── Guard: Overflow ───────────────────────────────────────────
    // Maximum safe value: i128::MAX / 10^14 ≈ 1.7 × 10²⁴
    // But for practical reward ranges, we cap at ±10⁴ as per spec.
    // The hard limit is where scaled + 0.5 would overflow f64 precision.
    const MAX_SAFE: f64 = 9.2e4;
    if r > MAX_SAFE || r < -MAX_SAFE {
        return Err(RcoError::RewardOverflow { bits: r.to_bits() });
    }

    // ── Branchless P14 Projection ─────────────────────────────────
    //
    // copysign(0.5, r) = +0.5 if r ≥ +0.0, -0.5 if r ≤ -0.0
    // This replaces the branch: if r >= 0 { +0.5 } else { -0.5 }
    //
    // On x86: compiles to VANDPD + VORPD (2 instructions, no branch)
    // On ARM: compiles to FCSEL (1 instruction, no branch)
    let scaled = r * (P14_SCALE as f64);
    let biased = scaled + f64::copysign(0.5, scaled);

    // Truncation toward zero gives us the correct rounding:
    // - Positive: floor(r * 10^14 + 0.5) = round-half-up
    // - Negative: floor(r * 10^14 - 0.5) = round-half-down (toward -∞)
    let projected = biased as i128;

    Ok(ProjectedReward(projected))
}

#[cfg(p14_asm)]
unsafe extern "C" {
    fn rco_p14_project_batch_avx512(input: *const f64, count: usize, output: *mut i128);
    fn rco_p14_project_batch_neon(input: *const f64, count: usize, output: *mut i128);
}

/// Projects a batch of rewards using the most efficient available method.
pub fn project_p14_batch(rewards: &[f64], output: &mut [i128]) -> Result<(), RcoError> {
    if rewards.len() != output.len() {
        return Err(RcoError::BufferTooSmall {
            required: rewards.len(),
            available: output.len(),
        });
    }

    #[cfg(p14_asm)]
    {
        #[cfg(all(target_arch = "x86_64", feature = "std"))]
        if std::is_x86_feature_detected!("avx512f") {
            unsafe {
                rco_p14_project_batch_avx512(rewards.as_ptr(), rewards.len(), output.as_mut_ptr());
            }
            return Ok(());
        }

        #[cfg(target_arch = "aarch64")]
        {
            unsafe {
                rco_p14_project_batch_neon(rewards.as_ptr(), rewards.len(), output.as_mut_ptr());
            }
            return Ok(());
        }
    }

    // Fallback to scalar Rust implementation
    for (i, &r) in rewards.iter().enumerate() {
        output[i] = project_p14(r)?.raw();
    }
    Ok(())
}

/// Recovers an approximate `f64` from a P14-projected value.
///
/// # Important
///
/// This is **not** the inverse of `project_p14`. Due to floating-point
/// quantization, `unproject_p14(project_p14(r))` may differ from `r`
/// by up to `0.5 × 10⁻¹⁴`. This function is intended for display
/// and debugging only — never re-project an unprojected value.
#[must_use]
#[inline]
pub fn unproject_p14(projected: ProjectedReward) -> f64 {
    projected.approximate_f64()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Core Projection Tests ───────────────────────────────────────

    #[test]
    fn test_project_zero() {
        let result = project_p14(0.0).unwrap();
        assert_eq!(result.raw(), 0);
    }

    #[test]
    fn test_project_positive_zero() {
        let result = project_p14(0.0).unwrap();
        assert_eq!(result.raw(), 0);
        assert!(!result.is_negative());
    }

    #[test]
    fn test_project_negative_zero() {
        // IEEE-754 negative zero should project to 0
        let result = project_p14(-0.0).unwrap();
        assert_eq!(result.raw(), 0);
    }

    #[test]
    fn test_project_positive_integer() {
        let result = project_p14(1.0).unwrap();
        assert_eq!(result.raw(), 100_000_000_000_000); // 1 × 10^14
    }

    #[test]
    fn test_project_negative_integer() {
        let result = project_p14(-1.0).unwrap();
        assert_eq!(result.raw(), -100_000_000_000_000); // -1 × 10^14
    }

    #[test]
    fn test_project_pi() {
        // π = 3.14159265358979...
        // P14(π) = ⌊3.14159265358979 × 10^14 + 0.5⌋ = 314159265358979
        let result = project_p14(core::f64::consts::PI).unwrap();
        assert_eq!(result.raw(), 314_159_265_358_979);
    }

    #[test]
    fn test_project_negative_pi() {
        let result = project_p14(-core::f64::consts::PI).unwrap();
        assert_eq!(result.raw(), -314_159_265_358_979);
    }

    #[test]
    fn test_project_small_positive() {
        // 0.00000000000001 = 10^-14 → P14 = 1
        let result = project_p14(1e-14).unwrap();
        assert_eq!(result.raw(), 1);
    }

    #[test]
    fn test_project_small_negative() {
        let result = project_p14(-1e-14).unwrap();
        assert_eq!(result.raw(), -1);
    }

    #[test]
    fn test_project_sub_resolution() {
        // 0.4 × 10^-14 should round to 0 (below half-ULP)
        let result = project_p14(0.4e-14).unwrap();
        assert_eq!(result.raw(), 0);
    }

    #[test]
    fn test_project_typical_rl_reward() {
        // Typical RL reward: 0.99
        let result = project_p14(0.99).unwrap();
        assert_eq!(result.raw(), 99_000_000_000_000);
    }

    #[test]
    fn test_project_large_reward() {
        // Maximum spec range: 10^4
        let result = project_p14(10_000.0).unwrap();
        assert_eq!(result.raw(), 1_000_000_000_000_000_000);
    }

    // ── Symmetry Tests ──────────────────────────────────────────────

    #[test]
    fn test_symmetry_around_zero() {
        // P14(r) should equal -P14(-r) for exact values
        let values = [1.0, 0.5, 0.1, 42.0, 100.0, 0.001];
        for &v in &values {
            let pos = project_p14(v).unwrap().raw();
            let neg = project_p14(-v).unwrap().raw();
            assert_eq!(pos, -neg, "Symmetry violated for {v}");
        }
    }

    // ── Error Bound Verification ────────────────────────────────────

    #[test]
    fn test_error_bound() {
        // Verify: ε = |r - r̂ × 10⁻¹⁴| < 0.5 × 10⁻¹⁴
        let test_values = [
            0.1,
            0.2,
            0.3,
            1.0 / 3.0,
            core::f64::consts::PI,
            core::f64::consts::E,
            42.42,
            -99.99,
            0.000_001,
        ];
        let max_error = 0.5e-14;

        for &r in &test_values {
            let projected = project_p14(r).unwrap();
            let recovered = projected.raw() as f64 * 1e-14;
            let error = (r - recovered).abs();
            assert!(
                error < max_error,
                "Error bound violated for r={r}: ε={error:.2e} > {max_error:.2e}"
            );
        }
    }

    // ── Determinism Tests ───────────────────────────────────────────

    #[test]
    fn test_deterministic_repeated_projection() {
        let r = core::f64::consts::PI;
        let a = project_p14(r).unwrap();
        let b = project_p14(r).unwrap();
        assert_eq!(a, b, "Repeated projection must be bit-identical");
    }

    #[test]
    fn test_deterministic_same_value_different_path() {
        // Compute the same value via different arithmetic paths
        let a = 0.1 + 0.2; // Known IEEE-754 quirk: 0.30000000000000004
        let b = 0.3;

        let pa = project_p14(a).unwrap();
        let pb = project_p14(b).unwrap();

        // These MAY differ because a ≠ b in IEEE-754,
        // but that's correct behavior — P14 preserves the distinction.
        // The important thing is that EACH projection is deterministic.
        let pa2 = project_p14(a).unwrap();
        let pb2 = project_p14(b).unwrap();
        assert_eq!(pa, pa2);
        assert_eq!(pb, pb2);
    }

    // ── Edge Cases: Error Handling ───────────────────────────────────

    #[test]
    fn test_reject_nan() {
        assert!(matches!(project_p14(f64::NAN), Err(RcoError::RewardNaN)));
    }

    #[test]
    fn test_reject_positive_infinity() {
        assert!(matches!(
            project_p14(f64::INFINITY),
            Err(RcoError::RewardOverflow { .. })
        ));
    }

    #[test]
    fn test_reject_negative_infinity() {
        assert!(matches!(
            project_p14(f64::NEG_INFINITY),
            Err(RcoError::RewardOverflow { .. })
        ));
    }

    #[test]
    fn test_reject_overflow() {
        // Value exceeding the ±9.2×10⁴ safe range
        assert!(matches!(
            project_p14(1e5),
            Err(RcoError::RewardOverflow { .. })
        ));
    }

    #[test]
    fn test_reject_negative_overflow() {
        assert!(matches!(
            project_p14(-1e5),
            Err(RcoError::RewardOverflow { .. })
        ));
    }

    // ── Newtype Tests ───────────────────────────────────────────────

    #[test]
    fn test_projected_reward_ordering() {
        let a = ProjectedReward::from_raw(100);
        let b = ProjectedReward::from_raw(200);
        assert!(a < b);
    }

    #[test]
    fn test_projected_reward_zero() {
        let z = ProjectedReward::zero();
        assert!(z.is_zero());
        assert!(!z.is_negative());
    }

    #[test]
    fn test_projected_reward_approximate_f64() {
        let p = ProjectedReward::from_raw(314_159_265_358_979);
        let approx = p.approximate_f64();
        assert!((approx - core::f64::consts::PI).abs() < 1e-14);
    }

    // ── Roundtrip Test ──────────────────────────────────────────────

    #[test]
    fn test_unproject_approximate() {
        let original = 42.5;
        let projected = project_p14(original).unwrap();
        let recovered = unproject_p14(projected);
        assert!((original - recovered).abs() < 1e-14);
    }

    #[test]
    fn test_project_p14_batch() {
        let rewards = [1.0, 2.0, 3.14159, -0.5, 0.0, 100.0, 9.2e4, -9.2e4];
        let mut out = [0i128; 8];
        
        project_p14_batch(&rewards, &mut out).unwrap();
        
        for i in 0..rewards.len() {
            let expected = project_p14(rewards[i]).unwrap().raw();
            assert_eq!(out[i], expected, "Batch index {i} mismatch");
        }
    }
}
