//! # RCO-Merkle — Recursive Merkle-Lineage (RML) Chain Engine
//!
//! Implements the cryptographic hash-chaining that forms the backbone of the
//! RCO Protocol's tamper-evidence guarantee.
//!
//! ## Recurrence Relation
//!
//! ```text
//! L_n = Keccak-256(Bencode(B_n) ‖ L_{n-1})
//! L_0 = Keccak-256(Bencode(B_0))        (Genesis Root)
//! ```
//!
//! ## Security Properties
//!
//! - **Forward Integrity**: If ∃ k < t where D_k' ≠ D_k, then P(L_t = L_t') ≤ 2⁻²⁵⁶
//! - **Collision Bound**: P(L_T¹ = L_T²) < T / 2²⁵⁶
//! - **Lineage Continuity**: Every anchor L_n causally depends on ALL preceding batches
//!
//! ## Acceptance Criterion (TC-03)
//!
//! 10% random historical audit → 100% anchor match.

#![no_std]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

extern crate alloc;

pub mod chain;
pub mod genesis;
pub mod verify;

pub use chain::{RmlAnchor, RmlChain};
pub use genesis::compute_genesis_root;
pub use verify::{verify_anchor, verify_chain_segment};
