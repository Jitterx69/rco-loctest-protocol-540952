//! # RCO-Enclave
//!
//! Emulator for the Recursive Verification Enclaves (RVE).
//! Models the memory isolation between a Root-of-Trust Enclave (RTE)
//! and an Ingestion Enclave (IE) via a Secure Telemetry Shunt.

#![warn(missing_docs)]

pub mod ie;
pub mod rte;
pub mod shunt;
pub mod attestation;
pub mod routing;
pub mod oracle;
pub mod handshake;
pub mod dgq;
pub mod synthesis;
pub mod closure;
