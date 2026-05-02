//! TPM Simulator
//!
//! Emulates a hardware TPM's PCR registers and key hierarchies.

use crate::TpmProvider;
use rco_types::HashDigest;
use rco_types::error::RcoError;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// A Platform Configuration Register (PCR).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pcr {
    /// The current 256-bit value of the PCR.
    pub value: HashDigest,
}

impl Pcr {
    /// Initializes a PCR to all zeros.
    pub fn new() -> Self {
        Self { value: [0u8; 32] }
    }

    /// Extends the PCR with new data: PCR_new = Hash(PCR_old || data).
    pub fn extend(&mut self, data: &[u8]) {
        let mut hasher = Sha256::new();
        hasher.update(self.value);
        hasher.update(data);
        self.value = hasher.finalize().into();
    }
}

/// The simulated TPM device.
pub struct TpmSimulator {
    /// Platform Configuration Registers. Index 0-23.
    pcrs: HashMap<u32, Pcr>,
}

impl TpmSimulator {
    /// Creates a new TPM simulator with cleared PCRs.
    pub fn new() -> Self {
        let mut pcrs = HashMap::new();
        for i in 0..24 {
            pcrs.insert(i, Pcr::new());
        }
        Self { pcrs }
    }

    /// Extends a specific PCR.
    pub fn pcr_extend(&mut self, index: u32, data: &[u8]) -> Result<(), RcoError> {
        if let Some(pcr) = self.pcrs.get_mut(&index) {
            pcr.extend(data);
            Ok(())
        } else {
            Err(RcoError::NumericalIntegrityFault)
        }
    }

    /// Reads a specific PCR.
    pub fn pcr_read(&self, index: u32) -> Option<HashDigest> {
        self.pcrs.get(&index).map(|p| p.value)
    }

    /// Generates a composite digest of a selected set of PCRs.
    /// This is used to form the policy hash.
    pub fn pcr_composite_digest(&self, selection: &[u32]) -> HashDigest {
        let mut hasher = Sha256::new();
        for &idx in selection {
            if let Some(val) = self.pcr_read(idx) {
                hasher.update(val);
            }
        }
        hasher.finalize().into()
    }
}

impl TpmProvider for TpmSimulator {
    fn pcr_read(&self, index: u32) -> Result<HashDigest, RcoError> {
        self.pcrs
            .get(&index)
            .map(|p| p.value)
            .ok_or(RcoError::NumericalIntegrityFault) // Use a suitable error
    }

    fn pcr_extend(&mut self, index: u32, data: &[u8]) -> Result<(), RcoError> {
        if let Some(pcr) = self.pcrs.get_mut(&index) {
            pcr.extend(data);
            Ok(())
        } else {
            Err(RcoError::NumericalIntegrityFault)
        }
    }

    fn quote(&self, selection: &[u32], _nonce: &[u8]) -> Result<Vec<u8>, RcoError> {
        // Mock quote: just the composite digest
        Ok(self.pcr_composite_digest(selection).to_vec())
    }

    fn get_ek_public(&self) -> Result<Vec<u8>, RcoError> {
        // Mock EK public key
        Ok(vec![0xDE, 0xAD, 0xBE, 0xEF])
    }
}

impl Default for TpmSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tpm_provider_trait() {
        let mut tpm_sim = TpmSimulator::new();
        let tpm: &mut dyn TpmProvider = &mut tpm_sim;

        // Initial state
        let pcr0 = tpm.pcr_read(0).unwrap();
        assert_eq!(pcr0, [0u8; 32]);

        // Extension
        tpm.pcr_extend(0, b"measurement").unwrap();
        let pcr0_ext = tpm.pcr_read(0).unwrap();
        assert_ne!(pcr0, pcr0_ext);

        // Quote
        let quote = tpm.quote(&[0], b"nonce").unwrap();
        assert_eq!(quote, pcr0_ext.to_vec());

        // EK
        let ek = tpm.get_ek_public().unwrap();
        assert!(!ek.is_empty());
    }
}
