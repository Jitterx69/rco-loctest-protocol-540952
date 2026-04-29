//! # RCO-Enclave
//!
//! Emulator for the Recursive Verification Enclaves (RVE).
//! Models the memory isolation between a Root-of-Trust Enclave (RTE)
//! and an Ingestion Enclave (IE) via a Secure Telemetry Shunt.

#![warn(missing_docs)]

pub mod rte;
pub mod ie;
pub mod shunt;
pub mod attestation;
