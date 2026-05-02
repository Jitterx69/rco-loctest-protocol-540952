//! RCO Hardware Inversion Layer
//!
//! Provides a unified interface for Intel SGX, AMD SEV, and software emulation.

pub mod sgx;
pub mod sev;

use crate::rte::HardwareIdentity;

/// Supported Hardware Trusted Execution Environments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TEEType {
    /// Intel Software Guard Extensions
    SGX,
    /// AMD Secure Encrypted Virtualization
    SEV,
    /// Software Emulation (Non-Stage-IV)
    Emulator,
}

/// Interface for Hardware-Bound Identity
pub trait TEEProvider {
    /// Detect if the current hardware supports this TEE
    fn is_available(&self) -> bool;
    
    /// Get the hardware-bound identity root
    fn get_identity(&self) -> HardwareIdentity;
    
    /// Generate a hardware-signed attestation report
    fn generate_report(&self, data: &[u8]) -> Vec<u8>;
}

/// Detects the best available TEE on the current host.
pub fn detect_tee() -> TEEType {
    // In a real implementation, we would use CPUID checks here.
    // For now, we return Emulator unless explicitly configured.
    if cfg!(feature = "sgx") {
        TEEType::SGX
    } else if cfg!(feature = "sev") {
        TEEType::SEV
    } else {
        TEEType::Emulator
    }
}
