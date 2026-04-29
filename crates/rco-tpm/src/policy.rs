//! Policy-Based Access Control (PBAC)
//!
//! Emulates TPM2_PolicyAuthorize and sealed storage.

use crate::simulator::TpmSimulator;
use rco_types::HashDigest;
use rco_crypto::bls::PrivateShare;

/// A sealed blob containing the BLS private key share.
pub struct SealedShare {
    encrypted_share: PrivateShare,
    expected_pcr_composite: HashDigest,
    pcr_selection: Vec<u32>,
}

impl SealedShare {
    /// Seals a private share against a specific PCR composite state.
    pub fn seal(tpm: &TpmSimulator, share: PrivateShare, pcr_selection: &[u32]) -> Self {
        let expected = tpm.pcr_composite_digest(pcr_selection);
        Self {
            encrypted_share: share, // Emulate encryption by storing it
            expected_pcr_composite: expected,
            pcr_selection: pcr_selection.to_vec(),
        }
    }

    /// Attempts to unseal the private share by asserting the current PCR state.
    /// Emulates TPM2_PolicyPCR and TPM2_Unseal.
    pub fn unseal(&self, tpm: &TpmSimulator) -> Result<PrivateShare, &'static str> {
        let current_composite = tpm.pcr_composite_digest(&self.pcr_selection);
        
        if current_composite == self.expected_pcr_composite {
            // Emulate decryption
            Ok(self.encrypted_share.clone())
        } else {
            Err("TPM2_PolicyPCR assertion failed: PCR mismatch")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rco_crypto::bls::PrivateShare;
    use rand::thread_rng;

    #[test]
    fn test_ths01_pcr_policy_enforcement() {
        let mut tpm = TpmSimulator::new();
        let mut rng = thread_rng();

        // 1. Emulate Measured Boot sequence
        tpm.pcr_extend(0, b"BIOS_MEASUREMENT");
        tpm.pcr_extend(4, b"KERNEL_HASH");
        tpm.pcr_extend(8, b"RCO_LOADER");
        
        // The selection of PCRs we want to bind to
        let selection = vec![0, 4, 8];

        // 2. Seal the share
        let original_share = PrivateShare::random(&mut rng);
        let sealed = SealedShare::seal(&tpm, original_share.clone(), &selection);

        // 3. Try unsealing immediately (should succeed)
        let unsealed = sealed.unseal(&tpm).expect("Should unseal cleanly");
        assert_eq!(original_share.0, unsealed.0);

        // 4. Emulate a Byzantine modification to the kernel (THS-01 Test)
        // In real life, the system reboots and PCR 4 hashes a DIFFERENT kernel.
        // We simulate this by modifying PCR 4 post-sealing.
        tpm.pcr_extend(4, b"MALICIOUS_KERNEL_MODULE");

        // 5. Try unsealing again (should fail)
        let result = sealed.unseal(&tpm);
        assert!(result.is_err(), "Access failure rate must be 100% on modified PCR baseline");
    }
}
