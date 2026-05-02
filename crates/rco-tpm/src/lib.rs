//! # RCO-TPM
//!
//! TPM 2.0 Interface for Hardware-Bound Trust Anchors.
//! Implements PCR registries, Attestation, and Policy-Based Access Control.

#![warn(missing_docs)]

use rco_types::HashDigest;
use rco_types::error::RcoError;

pub mod simulator;
pub mod policy;
pub mod hardware;

/// Abstraction over TPM 2.0 devices (Hardware or Simulator).
pub trait TpmProvider {
    /// Reads a specific PCR (Platform Configuration Register).
    fn pcr_read(&self, index: u32) -> Result<HashDigest, RcoError>;

    /// Extends a specific PCR with new measurement data.
    fn pcr_extend(&mut self, index: u32, data: &[u8]) -> Result<(), RcoError>;

    /// Generates an Attestation Quote signed by the TPM's Attestation Key (AK).
    /// This proves the state of the PCRs to a remote verifier.
    fn quote(&self, selection: &[u32], nonce: &[u8]) -> Result<Vec<u8>, RcoError>;

    /// Returns the TPM's unique Endorsement Key (EK) public part.
    fn get_ek_public(&self) -> Result<Vec<u8>, RcoError>;
}
