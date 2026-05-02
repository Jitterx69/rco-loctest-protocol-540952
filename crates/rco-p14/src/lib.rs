//! # RCO-P14 — Fixed-Precision Mantissa Projection
//!
//! Implements the $\mathcal{P}_{14}$ operator that eliminates IEEE-754 floating-point
//! non-determinism by projecting real-valued telemetry into a 128-bit integer lattice.

#![no_std]
#![allow(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

#[cfg(feature = "std")]
extern crate std;

pub mod boundary;
pub mod projection;

pub use boundary::{validate_reward, RewardClass};
pub use projection::{project_p14, unproject_p14, ProjectedReward};
