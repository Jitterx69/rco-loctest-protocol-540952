//! # RCO Types — Shared Type Definitions
//!
//! Core type definitions for the Feedback-Coupled Cryptographic Observation (RCO)
//! protocol. This crate provides the canonical data structures used across all
//! protocol layers, from serialization through audit verification.
//!
//! ## Design Invariants
//!
//! - **`#![no_std]`**: All types are platform-agnostic and TEE-compatible.
//! - **`Zeroize`-on-Drop**: All sensitive types implement `Zeroize` to prevent
//!   key material from persisting in memory after use (F-34 mitigation).
//! - **`repr(C)`**: All FFI-exposed types use C-compatible layout for the
//!   Julia simulation SDK bridge.

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

extern crate alloc;

pub mod batch;
pub mod error;
pub mod genesis;

/// Protocol version string for handshake validation (F-13 mitigation).
pub const PROTOCOL_VERSION: &str = "RCO-S1-24.1.0";

/// Maximum recursion depth for Bencode parsing (F-31 mitigation).
pub const BENCODE_MAX_DEPTH: usize = 16;

/// The mantissa projection exponent: k = 14 significant digits.
pub const P14_EXPONENT: u32 = 14;

/// Scaling factor for P14 projection: 10^14.
pub const P14_SCALE: i128 = 100_000_000_000_000;

/// Hash output size in bytes (Keccak-256 / SHA3-256).
pub const HASH_SIZE: usize = 32;

/// The fixed-size type for a cryptographic hash digest.
pub type HashDigest = [u8; HASH_SIZE];
