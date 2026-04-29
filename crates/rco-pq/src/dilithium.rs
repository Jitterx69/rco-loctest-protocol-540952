//! Dilithium Signatures
//!
//! Wraps `pqcrypto-dilithium` to provide Witness B.

use pqcrypto_dilithium::dilithium5::*;
use pqcrypto_traits::sign::{PublicKey, SecretKey, DetachedSignature};

/// A Dilithium Keypair.
pub struct Keypair {
    public: PublicKeyBox,
    secret: SecretKeyBox,
}

// We wrap the pqcrypto types because they don't derive standard traits like Clone/Debug by default.
pub struct PublicKeyBox(pub pqcrypto_dilithium::dilithium5::PublicKey);
pub struct SecretKeyBox(pub pqcrypto_dilithium::dilithium5::SecretKey);

impl Keypair {
    /// Generates a new Dilithium5 keypair.
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        Self {
            public: PublicKeyBox(pk),
            secret: SecretKeyBox(sk),
        }
    }

    /// Signs a message hash.
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let sig = detached_sign(message, &self.secret.0);
        sig.as_bytes().to_vec()
    }

    /// Returns the public key.
    pub fn public_key(&self) -> PublicKeyBox {
        PublicKeyBox(self.public.0.clone())
    }
}

impl Clone for PublicKeyBox {
    fn clone(&self) -> Self {
        PublicKeyBox(pqcrypto_dilithium::dilithium5::PublicKey::from_bytes(self.0.as_bytes()).unwrap())
    }
}

/// Verifies a Dilithium signature.
pub fn verify(message: &[u8], signature_bytes: &[u8], public_key: &PublicKeyBox) -> bool {
    if let Ok(sig) = pqcrypto_dilithium::dilithium5::DetachedSignature::from_bytes(signature_bytes) {
        verify_detached_signature(&sig, message, &public_key.0).is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dilithium_sign_verify() {
        let keypair = Keypair::generate();
        let message = b"post-quantum test message";

        let signature = keypair.sign(message);
        let pk = keypair.public_key();

        let is_valid = verify(message, &signature, &pk);
        assert!(is_valid);
        
        // Test failure
        let invalid_message = b"tampered message";
        let is_invalid = verify(invalid_message, &signature, &pk);
        assert!(!is_invalid);
    }
}
