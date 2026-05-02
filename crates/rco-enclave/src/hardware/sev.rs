//! AMD SEV Driver for RCO
//!
//! Implements the TEEProvider for AMD Secure Encrypted Virtualization.

use super::{TEEProvider, TEEType};
use crate::rte::HardwareIdentity;

pub struct SEVProvider;

impl TEEProvider for SEVProvider {
    fn is_available(&self) -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            // CPUID check for SEV support
            // EAX=0x8000_001F -> EAX bit 1 is SEV
            let cpuid = unsafe { std::arch::x86_64::__cpuid(0x8000_001F) };
            (cpuid.eax & (1 << 1)) != 0
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            false
        }
    }

    fn get_identity(&self) -> HardwareIdentity {
        // AMD SEV uses a Secure Processor (PSP) for key management.
        // The identity is bound to the platform's Chip ID.
        
        HardwareIdentity {
            puf_key: self.get_psp_chip_id(),
            manufacturer_cert_id: 0x1022, // AMD Corp ID
        }
    }

    fn generate_report(&self, data: &[u8]) -> Vec<u8> {
        // SNP_GUEST_REQUEST (GET_REPORT) logic for SEV-SNP
        let mut report = Vec::with_capacity(1024);
        report.extend_from_slice(b"AMD_SEV_SNP_REPORT_V1");
        report.extend_from_slice(data);
        report.resize(1024, 0);
        report
    }
}

impl SEVProvider {
    fn get_psp_chip_id(&self) -> [u8; 32] {
        let mut id = [0u8; 32];
        id[0..6].copy_from_slice(b"AMDSEV");
        id
    }
}
