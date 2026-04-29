//! Root-of-Trust Enclave (RTE)
//!
//! Emulates the highly-restricted core enclave responsible for managing 
//! the hardware keys, the Enclave-Bound Manifold Root ($\Omega_{rve}$),
//! and generating the HRNG entropy pool.

use rco_types::HashDigest;
use sha3::{Keccak256, Digest};
use rand::{RngCore, SeedableRng};
use rand::rngs::StdRng;

/// Simulates the hardware manufacturer ID and PUF keys.
pub struct HardwareIdentity {
    /// Silicon Physical Unclonable Function (mocked as a 32-byte key)
    pub puf_key: [u8; 32],
    /// Manufacturer Attestation Key (publicly verifiable)
    pub manufacturer_cert_id: u64,
}

impl HardwareIdentity {
    /// Derives the $K_{hardware}$ used for Enclave-Bound HMACs.
    pub fn derive_hardware_key(&self, context: &[u8]) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        hasher.update(&self.puf_key);
        hasher.update(context);
        let mut out = [0u8; 32];
        out.copy_from_slice(&hasher.finalize());
        out
    }
}

/// The Root-of-Trust Enclave.
pub struct RootOfTrustEnclave {
    pub identity: HardwareIdentity,
    pub entropy_pool: [u8; 32],
    /// The current Enclave-Bound Manifold Root
    pub omega_rve: HashDigest,
}

impl RootOfTrustEnclave {
    /// Initializes a new simulated RTE.
    pub fn new(manufacturer_cert_id: u64) -> Self {
        // In a real TEE, the PUF key is fused at manufacture time.
        let mut puf_key = [0u8; 32];
        let mut rng = StdRng::from_entropy();
        rng.fill_bytes(&mut puf_key);

        let mut entropy_pool = [0u8; 32];
        rng.fill_bytes(&mut entropy_pool);

        Self {
            identity: HardwareIdentity {
                puf_key,
                manufacturer_cert_id,
            },
            entropy_pool,
            omega_rve: [0u8; 32],
        }
    }

    /// Derives the Enclave-Bound Manifold Root: $\Omega_{rve} = \text{HMAC}_{K_{hardware}}(\mathcal{M}_{topological})$
    pub fn update_manifold_root(&mut self, topological_root: &HashDigest) {
        let k_hw = self.identity.derive_hardware_key(b"RCO_MANIFOLD_BINDING");
        
        let mut hasher = Keccak256::new();
        hasher.update(&k_hw);
        hasher.update(topological_root);
        
        self.omega_rve.copy_from_slice(&hasher.finalize());
    }

    /// Cycles the entropy pool (simulating RDRAND mixing).
    pub fn mix_entropy(&mut self) {
        let mut rng = StdRng::from_entropy();
        let mut new_entropy = [0u8; 32];
        rng.fill_bytes(&mut new_entropy);
        
        for i in 0..32 {
            self.entropy_pool[i] ^= new_entropy[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rte_initialization_and_binding() {
        let mut rte = RootOfTrustEnclave::new(0x8086);
        let topo_root = [0xAA; 32];
        
        // Ensure omega_rve changes upon update
        let old_omega = rte.omega_rve;
        rte.update_manifold_root(&topo_root);
        assert_ne!(old_omega, rte.omega_rve);
    }
}
