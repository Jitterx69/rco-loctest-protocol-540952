//! Secure Telemetry Shunt
//!
//! Emulates the hardware-bound DMA channels and memory-mapped boundaries
//! that isolate the RTE, IE, and Host.

use std::sync::{Arc, Mutex};
use rco_types::HashDigest;

/// Represents the shared memory regions defined in Phase-V.
pub struct MemoryMap {
    /// 0x0000 - 0x1000 : Shared Root Hash Register (RTE write, IE read)
    pub rte_root_register: Arc<Mutex<HashDigest>>,
    /// 0x1000 - 0x5000 : Simplex Data Buffer (IE write, RTE read-only)
    pub ie_simplex_buffer: Arc<Mutex<Vec<u8>>>,
    /// 0x5000 - 0x6000 : Attestation Challenge Slot (Host write, Enclaves read)
    pub attestation_slot: Arc<Mutex<[u8; 32]>>,
}

impl MemoryMap {
    pub fn new() -> Self {
        Self {
            rte_root_register: Arc::new(Mutex::new([0u8; 32])),
            ie_simplex_buffer: Arc::new(Mutex::new(Vec::new())),
            attestation_slot: Arc::new(Mutex::new([0u8; 32])),
        }
    }
}

/// The Secure Shunt connecting the enclaves.
pub struct SecureShunt {
    memory_map: MemoryMap,
}

impl SecureShunt {
    pub fn new() -> Self {
        Self {
            memory_map: MemoryMap::new(),
        }
    }

    /// Host writes an attestation challenge to the slot.
    pub fn host_write_challenge(&self, challenge: &[u8; 32]) {
        let mut slot = self.memory_map.attestation_slot.lock().unwrap();
        slot.copy_from_slice(challenge);
    }

    /// Enclave reads the attestation challenge.
    pub fn enclave_read_challenge(&self) -> [u8; 32] {
        *self.memory_map.attestation_slot.lock().unwrap()
    }

    /// RTE writes the Enclave-Bound Manifold Root.
    pub fn rte_write_root(&self, root: &HashDigest) {
        let mut register = self.memory_map.rte_root_register.lock().unwrap();
        register.copy_from_slice(root);
    }

    /// Host attempts to read the RTE root.
    /// In a real system, this triggers an access violation. Here, we simulate it
    /// by returning a permission error.
    pub fn host_read_rte_root(&self) -> Result<HashDigest, &'static str> {
        Err("Hardware Exception: Host cannot read PRM/RTE memory directly.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shunt_isolation() {
        let shunt = SecureShunt::new();
        
        let root = [0xBB; 32];
        shunt.rte_write_root(&root);
        
        // Host should fail to read the RTE register (C-501 requirement)
        let result = shunt.host_read_rte_root();
        assert!(result.is_err());
    }
}
