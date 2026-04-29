//! # RCO-P14 — Fixed-Precision Mantissa Projection
//!
//! Implements the $\mathcal{P}_{14}$ operator that eliminates IEEE-754 floating-point
//! non-determinism by projecting real-valued telemetry into a 128-bit integer lattice.
//!
//! ## The Problem: Gradient Hallucination
//!
//! IEEE-754 arithmetic is non-associative: `(a + b) + c ≠ a + (b + c)`.
//! In distributed reinforcement learning, this manifests as *Gradient Hallucination* —
//! where identical logical rewards produce divergent neural parameter updates across
//! heterogeneous hardware (x86 AVX-512 vs. ARM NEON). The divergence, though small
//! per step (~10⁻¹⁶), accumulates over 10⁹ steps to total policy instability.
//!
//! ## The Solution: Mantissa Projection
//!
//! ```text
//! P14: ℝ → ℤ₁₂₈
//! r̂ = ⌊r × 10¹⁴ + sgn(r) × 0.5⌋
//! ```
//!
//! This projects all rewards into integer space with 14 significant digits,
//! guaranteeing **bit-identical** representation across all architectures.
//!
//! ## Error Bound
//!
//! ```text
//! ε = |r - r̂ × 10⁻¹⁴| < 0.5 × 10⁻¹⁴
//! ```
//!
//! ## Innovation: Branchless Implementation
//!
//! The `sgn(r)` rounding bias is implemented without branches using
//! `f64::copysign(0.5, r)`, which compiles to a single instruction
//! on both x86 (`VFIXUPIMMPD`) and ARM (`FCSEL`), eliminating branch
//! misprediction overhead in high-frequency pipelines.
//!
//! ## Acceptance Criterion (TC-02)
//!
//! ```text
//! ∀t, r̂_t^ARM = r̂_t^x86
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

pub mod boundary;
pub mod projection;

pub use boundary::{validate_reward, RewardClass};
pub use projection::{project_p14, unproject_p14, ProjectedReward};
