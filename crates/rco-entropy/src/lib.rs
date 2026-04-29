//! # RCO-Entropy — Genesis Entropy Generation
//!
//! Generates cryptographically strong entropy for the genesis block's
//! `entropy_vector`. This module implements the Multi-Source Whitening Gate:
//!
//! ```text
//! E_genesis = HRNG ⊕ E_jitter ⊕ Keccak-256(auxiliary_entropy)
//! ```
//!
//! ## Entropy Sources
//!
//! 1. **HRNG**: Hardware random number generator (`getrandom()` → RDRAND/RDSEED).
//! 2. **Jitter**: Software-based jitter entropy (CPU timing variations).
//! 3. **Auxiliary**: Additional user-provided entropy (optional salt).
//!
//! ## Acceptance Criterion (TC-04)
//!
//! NIST SP 800-22 (15 tests) pass on 10,000 genesis anchors.
//! Shannon entropy: `H(L_0[i..i+31]) > 7.99 bits/byte`.
//!
//! ## Security Model
//!
//! - No single entropy source is trusted alone (defense-in-depth).
//! - XOR whitening ensures that even if one source is compromised,
//!   the output entropy is at least as strong as the strongest source.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

pub mod hwrng;
pub mod jitter;
pub mod validation;
pub mod whitening;

pub use validation::validate_entropy;
pub use whitening::generate_genesis_entropy;
