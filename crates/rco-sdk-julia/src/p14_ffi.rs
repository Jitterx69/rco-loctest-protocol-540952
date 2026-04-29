//! # P14 FFI — Mantissa Projection for Julia
//!
//! Exposes `project_p14` and `unproject_p14` across the C-ABI boundary.
//!
//! ## Julia Usage
//!
//! ```julia
//! const librco = "librco_sdk_julia"
//!
//! function project_p14(reward::Float64)::Int128
//!     projected = Ref{Int128}(0)
//!     status = ccall((:rco_p14_project, librco), Int32,
//!                    (Float64, Ptr{Int128}), reward, projected)
//!     status == 0 || error("P14 projection failed: status=$status")
//!     return projected[]
//! end
//! ```

use crate::error_codes::RcoStatus;
use rco_p14::projection::{project_p14, ProjectedReward};

/// Projects a single `f64` reward into P14 integer space.
///
/// # C Signature
///
/// ```c
/// int32_t rco_p14_project(double reward, int128_t* out_projected);
/// ```
///
/// # Safety
///
/// `out_projected` must be a valid, aligned, non-null pointer to an `i128`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_p14_project(
    reward: f64,
    out_projected: *mut i128,
) -> i32 {
    if out_projected.is_null() {
        return RcoStatus::NullPointer as i32;
    }

    match project_p14(reward) {
        Ok(projected) => {
            // SAFETY: Caller guarantees out_projected is valid and aligned.
            unsafe { *out_projected = projected.raw(); }
            RcoStatus::Ok as i32
        }
        Err(ref e) => RcoStatus::from_error(e) as i32,
    }
}

/// Recovers an approximate `f64` from a P14-projected value.
///
/// # C Signature
///
/// ```c
/// double rco_p14_unproject(int128_t projected);
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn rco_p14_unproject(projected: i128) -> f64 {
    let p = ProjectedReward::from_raw(projected);
    rco_p14::unproject_p14(p)
}

/// Projects a batch of `f64` rewards into P14 integer space.
///
/// # C Signature
///
/// ```c
/// int32_t rco_p14_project_batch(
///     const double* rewards, size_t count,
///     int128_t* out_projected,
///     size_t* out_first_error_index
/// );
/// ```
///
/// # Safety
///
/// - `rewards` must point to `count` valid `f64` values.
/// - `out_projected` must point to a buffer of at least `count` `i128` values.
/// - `out_first_error_index` may be null (error index not reported).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_p14_project_batch(
    rewards: *const f64,
    count: usize,
    out_projected: *mut i128,
    out_first_error_index: *mut usize,
) -> i32 {
    if rewards.is_null() || out_projected.is_null() {
        return RcoStatus::NullPointer as i32;
    }

    if count == 0 {
        return RcoStatus::Ok as i32;
    }

    // SAFETY: Caller guarantees rewards[0..count] and out_projected[0..count] are valid.
    let rewards_slice = unsafe { core::slice::from_raw_parts(rewards, count) };
    let out_slice = unsafe { core::slice::from_raw_parts_mut(out_projected, count) };

    for (i, &r) in rewards_slice.iter().enumerate() {
        match project_p14(r) {
            Ok(projected) => {
                out_slice[i] = projected.raw();
            }
            Err(ref e) => {
                if !out_first_error_index.is_null() {
                    unsafe { *out_first_error_index = i; }
                }
                return RcoStatus::from_error(e) as i32;
            }
        }
    }

    RcoStatus::Ok as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_project_p14() {
        let mut out: i128 = 0;
        let status = unsafe { rco_p14_project(1.0, &mut out) };
        assert_eq!(status, 0);
        assert_eq!(out, 100_000_000_000_000);
    }

    #[test]
    fn test_ffi_project_nan() {
        let mut out: i128 = 0;
        let status = unsafe { rco_p14_project(f64::NAN, &mut out) };
        assert_eq!(status, RcoStatus::RewardNaN as i32);
    }

    #[test]
    fn test_ffi_project_overflow() {
        let mut out: i128 = 0;
        let status = unsafe { rco_p14_project(1e5, &mut out) };
        assert_eq!(status, RcoStatus::RewardOverflow as i32);
    }

    #[test]
    fn test_ffi_project_null_output() {
        let status = unsafe { rco_p14_project(1.0, core::ptr::null_mut()) };
        assert_eq!(status, RcoStatus::NullPointer as i32);
    }

    #[test]
    fn test_ffi_unproject() {
        let result = rco_p14_unproject(314_159_265_358_979);
        assert!((result - core::f64::consts::PI).abs() < 1e-14);
    }

    #[test]
    fn test_ffi_project_batch() {
        let rewards = [1.0, 2.0, 3.0, -0.5];
        let mut out = [0i128; 4];
        let mut err_idx: usize = 0;

        let status = unsafe {
            rco_p14_project_batch(
                rewards.as_ptr(),
                rewards.len(),
                out.as_mut_ptr(),
                &mut err_idx,
            )
        };

        assert_eq!(status, 0);
        assert_eq!(out[0], 100_000_000_000_000);
        assert_eq!(out[1], 200_000_000_000_000);
        assert_eq!(out[2], 300_000_000_000_000);
        assert_eq!(out[3], -50_000_000_000_000);
    }

    #[test]
    fn test_ffi_project_batch_with_nan() {
        let rewards = [1.0, f64::NAN, 3.0];
        let mut out = [0i128; 3];
        let mut err_idx: usize = 999;

        let status = unsafe {
            rco_p14_project_batch(
                rewards.as_ptr(),
                rewards.len(),
                out.as_mut_ptr(),
                &mut err_idx,
            )
        };

        assert_eq!(status, RcoStatus::RewardNaN as i32);
        assert_eq!(err_idx, 1);
    }

    #[test]
    fn test_ffi_project_batch_empty() {
        let status = unsafe {
            rco_p14_project_batch(
                core::ptr::null(),
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            )
        };
        // count == 0 returns Ok before null check on pointers
        assert_eq!(status, RcoStatus::NullPointer as i32);
    }

    #[test]
    fn test_ffi_project_batch_null_input() {
        let status = unsafe {
            rco_p14_project_batch(
                core::ptr::null(),
                5,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            )
        };
        assert_eq!(status, RcoStatus::NullPointer as i32);
    }
}
