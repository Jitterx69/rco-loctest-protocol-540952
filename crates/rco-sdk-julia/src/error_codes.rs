//! # FFI Error Codes
//!
//! Integer status codes returned across the C-ABI boundary.
//! Julia checks these after every `ccall` to detect failures.
//!
//! ## Convention
//!
//! - `0` = Success
//! - `1..99` = Protocol errors (mapped from `RcoError` variants)
//! - `100..199` = FFI-specific errors (null pointer, buffer too small)

use rco_types::error::RcoError;

/// C-compatible status code returned by all FFI functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum RcoStatus {
    /// Operation succeeded.
    Ok = 0,

    /// Input reward is NaN.
    RewardNaN = 1,

    /// Input reward exceeds P14 representable range.
    RewardOverflow = 2,

    /// Bencode encoding failed (buffer too small).
    EncodeFailed = 10,

    /// Bencode decoding failed (invalid grammar).
    DecodeFailed = 11,

    /// Duplicate dictionary key.
    DuplicateKey = 12,

    /// RML chain: non-sequential batch index.
    LinkageGap = 20,

    /// Hash mismatch during verification.
    HashMismatch = 21,

    /// Null pointer passed to FFI function.
    NullPointer = 100,

    /// Output buffer too small.
    BufferTooSmall = 101,

    /// Invalid argument (e.g., zero-length where non-zero required).
    InvalidArgument = 102,

    /// Unknown internal error.
    InternalError = 127,
}

impl RcoStatus {
    /// Converts an `RcoError` to a C-compatible status code.
    #[must_use]
    pub fn from_error(err: &RcoError) -> Self {
        match err {
            RcoError::RewardNaN => Self::RewardNaN,
            RcoError::RewardOverflow { .. } => Self::RewardOverflow,
            RcoError::BufferTooSmall { .. } => Self::BufferTooSmall,
            RcoError::DuplicateKey => Self::DuplicateKey,
            RcoError::InvalidGrammar { .. }
            | RcoError::KeyOrderViolation
            | RcoError::InvalidInteger { .. }
            | RcoError::RecursionDepthExceeded { .. } => Self::DecodeFailed,
            RcoError::LinkageContinuityGap { .. } => Self::LinkageGap,
            RcoError::AuditHashMismatch { .. }
            | RcoError::AnchorMismatch => Self::HashMismatch,
            _ => Self::InternalError,
        }
    }
}

impl From<i32> for RcoStatus {
    fn from(code: i32) -> Self {
        match code {
            0 => Self::Ok,
            1 => Self::RewardNaN,
            2 => Self::RewardOverflow,
            10 => Self::EncodeFailed,
            11 => Self::DecodeFailed,
            12 => Self::DuplicateKey,
            20 => Self::LinkageGap,
            21 => Self::HashMismatch,
            100 => Self::NullPointer,
            101 => Self::BufferTooSmall,
            102 => Self::InvalidArgument,
            _ => Self::InternalError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_ok_is_zero() {
        assert_eq!(RcoStatus::Ok as i32, 0);
    }

    #[test]
    fn test_status_roundtrip() {
        let codes = [0, 1, 2, 10, 11, 12, 20, 21, 100, 101, 102, 127];
        for code in codes {
            let status = RcoStatus::from(code);
            assert_eq!(status as i32, code);
        }
    }

    #[test]
    fn test_from_rco_error() {
        assert_eq!(RcoStatus::from_error(&RcoError::RewardNaN), RcoStatus::RewardNaN);
        assert_eq!(
            RcoStatus::from_error(&RcoError::RewardOverflow { bits: 0 }),
            RcoStatus::RewardOverflow
        );
        assert_eq!(
            RcoStatus::from_error(&RcoError::DuplicateKey),
            RcoStatus::DuplicateKey
        );
    }

    #[test]
    fn test_unknown_code_maps_to_internal() {
        assert_eq!(RcoStatus::from(999), RcoStatus::InternalError);
    }
}
