//! TPM Simulator
//!
//! Emulates a hardware TPM's PCR registers and key hierarchies.

use rco_types::HashDigest;
use sha2::{Sha256, Digest};
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
    pub fn pcr_extend(&mut self, index: u32, data: &[u8]) {
        if let Some(pcr) = self.pcrs.get_mut(&index) {
            pcr.extend(data);
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

impl Default for TpmSimulator {
    fn default() -> Self {
        Self::new()
    }
}
