//! # Error Taxonomy
//!
//! Comprehensive error types for the RCO Protocol, directly mapping to the
//! Failure Mode and Effects Analysis (FMEA) documented in the patent
//! specification. Each variant corresponds to a specific failure code (F-XX).
//!
//! ## Design Philosophy
//!
//! Errors are divided into two severity classes:
//! - **`Fatal`**: Triggers immediate Propulsion Halt. The simulation MUST stop.
//! - **`Recoverable`**: The protocol can recover without halting the pipeline.

use core::fmt;

/// Top-level error type for the RCO Protocol.
///
/// Every error carries its FMEA failure code for forensic traceability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RcoError {
    // ── Serialization Errors (Phase 1) ──────────────────────────────
    /// F-11: Duplicate dictionary key detected during Bencode encoding.
    /// This is a **fatal** grammar violation; the batch is rejected.
    DuplicateKey,

    /// F-31: Bencode recursion depth exceeds the hard limit of 16.
    /// Prevents stack-overflow attacks from deeply nested payloads.
    RecursionDepthExceeded {
        /// The depth at which the limit was breached.
        depth: usize,
    },

    /// Invalid Bencode grammar: the input does not conform to `G_RCO`.
    InvalidGrammar {
        /// Byte offset where the parse error occurred.
        offset: usize,
        /// Human-readable description of the violation.
        detail: &'static str,
    },

    /// Keys are not in strict lexicographic order.
    KeyOrderViolation,

    /// Integer encoding violation: leading zeros or negative zero.
    InvalidInteger {
        /// Byte offset of the malformed integer token.
        offset: usize,
    },

    // ── Mantissa Projection Errors (Phase 2) ─────────────────────────
    /// F-06: A floating-point value entered the pipeline without P14 projection.
    NumericalIntegrityFault,

    /// The input reward is NaN, which has no valid P14 representation.
    RewardNaN,

    /// The input reward exceeds the representable range of P14 (±10^4).
    RewardOverflow {
        /// The offending value's IEEE-754 bit representation (`f64::to_bits()`).
        bits: u64,
    },

    // ── Merkle-Lineage Errors (Phase 3) ──────────────────────────────
    /// F-02: Linkage continuity gap — L_n does not chain to L_{n-1}.
    LinkageContinuityGap {
        /// The expected batch index.
        expected_index: u64,
        /// The received batch index.
        received_index: u64,
    },

    /// The computed hash does not match the provided anchor.
    AnchorMismatch,

    // ── Genesis Errors (Phase 4) ─────────────────────────────────────
    /// F-04: Genesis entropy does not meet the 7.99 bits/byte threshold.
    InsufficientEntropy {
        /// Measured Shannon entropy in millibits per byte (e.g., 7990 = 7.99 b/B).
        measured_millibits: u32,
    },

    /// F-19: Low-entropy seed detected (e.g., VM boot stall).
    GenesisCollision,

    // ── Ingestion Errors (Phase 5) ───────────────────────────────────
    /// F-01: Byzantine replay attack detected — duplicate (run_uuid, batch_index).
    ByzantineReplay,

    /// F-05: Threshold quorum failure — insufficient nodes for 2PC commit.
    QuorumFailure {
        /// Number of available nodes.
        available: usize,
        /// Minimum required for consensus.
        required: usize,
    },

    /// F-07: Temporal monotonicity violation — timestamp out of sequence.
    TemporalMonotonicViolation,

    /// F-08: Sidecar IPC buffer saturation — backpressure triggered.
    BackpressureSaturation,

    /// Backpressure gate rejected ingestion — too many in-flight batches.
    BackpressureExceeded {
        /// Current queue depth at time of rejection.
        queue_depth: u64,
    },

    /// Write-Ahead Log commit failure (disk I/O error).
    WalCommitFailure,

    /// Two-Phase Commit protocol abort — invalid state transition.
    TwoPcAbort,

    // ── Audit Errors (Phase 7) ───────────────────────────────────────
    /// The full-chain audit sweep detected a hash mismatch.
    AuditHashMismatch {
        /// The batch index where the mismatch occurred.
        batch_index: u64,
    },

    /// F-33: NVMe bit-rot detected during background scrub.
    BitRotDetected {
        /// The batch index of the corrupted record.
        batch_index: u64,
    },

    // ── Protocol Errors ──────────────────────────────────────────────
    /// F-13: SDK version mismatch during handshake.
    VersionMismatch {
        /// The version reported by the connecting node.
        remote_version: &'static str,
    },

    /// Buffer too small to hold the serialized output.
    BufferTooSmall {
        /// Required buffer size in bytes.
        required: usize,
        /// Available buffer size in bytes.
        available: usize,
    },
}

impl RcoError {
    /// Returns the FMEA failure code for this error, if one is assigned.
    #[must_use]
    pub const fn failure_code(&self) -> Option<u16> {
        match self {
            Self::ByzantineReplay => Some(1),
            Self::LinkageContinuityGap { .. } => Some(2),
            Self::InsufficientEntropy { .. } => Some(4),
            Self::QuorumFailure { .. } => Some(5),
            Self::NumericalIntegrityFault => Some(6),
            Self::TemporalMonotonicViolation => Some(7),
            Self::BackpressureSaturation => Some(8),
            Self::DuplicateKey => Some(11),
            Self::VersionMismatch { .. } => Some(13),
            Self::RecursionDepthExceeded { .. } => Some(31),
            Self::BitRotDetected { .. } => Some(33),
            _ => None,
        }
    }

    /// Returns `true` if this error requires an immediate Propulsion Halt.
    #[must_use]
    pub const fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::ByzantineReplay
                | Self::LinkageContinuityGap { .. }
                | Self::AnchorMismatch
                | Self::NumericalIntegrityFault
                | Self::RewardNaN
                | Self::GenesisCollision
                | Self::DuplicateKey
                | Self::KeyOrderViolation
                | Self::AuditHashMismatch { .. }
        )
    }
}

impl fmt::Display for RcoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateKey => write!(f, "[F-11] Duplicate dictionary key in Bencode payload"),
            Self::RecursionDepthExceeded { depth } => {
                write!(
                    f,
                    "[F-31] Bencode recursion depth {depth} exceeds limit of 16"
                )
            }
            Self::InvalidGrammar { offset, detail } => {
                write!(f, "Invalid Bencode grammar at byte {offset}: {detail}")
            }
            Self::KeyOrderViolation => {
                write!(f, "Dictionary keys not in strict lexicographic order")
            }
            Self::InvalidInteger { offset } => {
                write!(f, "Malformed integer token at byte {offset}")
            }
            Self::NumericalIntegrityFault => {
                write!(f, "[F-06] Floating-point value bypassed P14 projection")
            }
            Self::RewardNaN => write!(f, "Reward value is NaN — no valid P14 representation"),
            Self::RewardOverflow { bits } => {
                write!(
                    f,
                    "Reward (bits=0x{bits:016X}) exceeds P14 representable range"
                )
            }
            Self::LinkageContinuityGap {
                expected_index,
                received_index,
            } => {
                write!(
                    f,
                    "[F-02] Linkage gap: expected batch {expected_index}, received {received_index}"
                )
            }
            Self::AnchorMismatch => write!(f, "Computed hash does not match provided anchor"),
            Self::InsufficientEntropy { measured_millibits } => {
                let whole = measured_millibits / 1000;
                let frac = measured_millibits % 1000;
                write!(
                    f,
                    "[F-04] Genesis entropy {whole}.{frac:03} bits/byte < 7.99 threshold"
                )
            }
            Self::GenesisCollision => write!(f, "[F-19] Genesis anchor collision detected"),
            Self::ByzantineReplay => {
                write!(
                    f,
                    "[F-01] Byzantine replay attack — duplicate batch detected"
                )
            }
            Self::QuorumFailure {
                available,
                required,
            } => {
                write!(
                    f,
                    "[F-05] Quorum failure: {available}/{required} nodes available"
                )
            }
            Self::TemporalMonotonicViolation => {
                write!(f, "[F-07] Timestamp monotonicity violation")
            }
            Self::BackpressureSaturation => {
                write!(f, "[F-08] IPC buffer saturated — Propulsion Halt")
            }
            Self::BackpressureExceeded { queue_depth } => {
                write!(
                    f,
                    "Backpressure: {queue_depth} batches in-flight, gate closed"
                )
            }
            Self::WalCommitFailure => write!(f, "WAL commit failure — disk I/O error"),
            Self::TwoPcAbort => write!(f, "2PC abort — invalid state transition"),
            Self::AuditHashMismatch { batch_index } => {
                write!(f, "Audit hash mismatch at batch {batch_index}")
            }
            Self::BitRotDetected { batch_index } => {
                write!(f, "[F-33] Bit-rot detected at batch {batch_index}")
            }
            Self::VersionMismatch { remote_version } => {
                write!(
                    f,
                    "[F-13] SDK version mismatch: remote reports {remote_version}"
                )
            }
            Self::BufferTooSmall {
                required,
                available,
            } => {
                write!(
                    f,
                    "Buffer too small: need {required} bytes, have {available}"
                )
            }
        }
    }
}
