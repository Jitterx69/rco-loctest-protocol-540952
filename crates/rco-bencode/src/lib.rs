//! # RCO-Bencode — Canonical Serialization Engine
//!
//! Implements the RCO-Bencode grammar `G_RCO` for deterministic, cross-platform
//! serialization. This is the first operator in the cryptographic pipeline:
//!
//! ```text
//! K = C ∘ H ∘ S   (where S = this crate)
//! ```
//!
//! ## Design Invariants
//!
//! 1. **Lexicographic Key Ordering**: Dictionary keys are emitted in strict
//!    byte-by-byte ascending order. Violation is a fatal error.
//! 2. **No Floating Points**: All numerical values must be pre-projected
//!    through P14 and represented as integers.
//! 3. **Depth Limit**: Recursion is hard-capped at 16 levels (F-31).
//! 4. **No Duplicate Keys**: Fatal parse error on duplicates (F-11).
//! 5. **Arena Allocation**: The encoder uses a pre-allocated buffer
//!    to achieve zero-copy, zero-allocation serialization.
//!
//! ## Cross-Platform Guarantee
//!
//! ```text
//! B(X) = δ(Enc_Julia(X), Enc_Rust(X)) = 1
//! ```
//! Verified across 1,000,000 random object permutations (TC-01).

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

extern crate alloc;

pub mod decoder;
pub mod encoder;
pub mod grammar;
pub mod sort;

pub use decoder::BencodeDecoder;
/// Re-export core types for convenience.
pub use encoder::BencodeEncoder;
pub use grammar::BencodeValue;
