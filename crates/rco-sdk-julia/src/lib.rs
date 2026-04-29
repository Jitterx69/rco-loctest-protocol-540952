//! # RCO-SDK-Julia — C-ABI FFI Bridge
//!
//! Exposes the RCO Protocol core to Julia simulation SDKs via a stable
//! C-ABI interface. Julia calls into this shared library using `ccall`.
//!
//! ## Architecture
//!
//! ```text
//! Julia Simulation
//!       │
//!       │ ccall(:rco_p14_project, ...)
//!       ▼
//! ┌─────────────────────┐
//! │  librco_sdk_julia.so │  ← This crate (cdylib)
//! │                     │
//! │  C-ABI functions:   │
//! │  - rco_p14_project  │
//! │  - rco_bencode_*    │
//! │  - rco_chain_*      │
//! └─────────┬───────────┘
//!           │
//!           ▼
//!    rco-p14, rco-bencode, rco-merkle (Rust crates)
//! ```
//!
//! ## Safety Contract
//!
//! This is the **only** crate in the workspace that uses `unsafe` code.
//! All `unsafe` is confined to FFI boundary marshalling — converting
//! between C pointers/lengths and Rust slices. The invariants are:
//!
//! 1. All pointer+length pairs must form valid, non-null, aligned slices.
//! 2. Output buffers must be pre-allocated by the caller.
//! 3. Returned status codes encode success (0) or error categories (>0).
//!
//! ## ABI Stability
//!
//! All exported functions use `extern "C"` with `#[no_mangle]`.
//! The C header is generated via `cbindgen` and checked into the repo.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

pub mod p14_ffi;
pub mod bencode_ffi;
pub mod chain_ffi;
pub mod ingestion_ffi;
pub mod alignment_ffi;
pub mod error_codes;

// Re-export for Rust-side tests
pub use error_codes::RcoStatus;
