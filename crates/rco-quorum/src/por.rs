//! Proof of Reflexion (PoR) Construction
//!
//! Binds Agent Neural Weights with Merkle-link and Telemetry.

use rco_types::HashDigest;
use sha2::{Sha256, Digest};
use bls12_381::G2Projective;
use group::Curve;

/// Constructs the Hash-to-Curve message for the PoR signature.
/// M_PoR = H(W_t) || L_t || S(B_t)
pub fn construct_por_message(
    weight_hash: HashDigest,
    merkle_link: HashDigest,
    telemetry_batch: &[u8],
) -> G2Projective {
    let mut hasher = Sha256::new();
    hasher.update(weight_hash);
    hasher.update(merkle_link);
    hasher.update(telemetry_batch);
    let digest = hasher.finalize();

    // RFC 9380 Hash-to-Curve mapping to G2.
    // In a full implementation, this uses expand_message_xmd and SSWU.
    // For this simulation SDK, we approximate by multiplying the generator by the scalar derived from the hash.
    // Note: This is a placeholder for the actual indifferentiable hash-to-curve.
    let mut scalar_bytes = [0u8; 32];
    scalar_bytes.copy_from_slice(&digest);
    
    // Reverse for endianness if needed, we just use from_bytes for scalar
    let scalar = bls12_381::Scalar::from_bytes(&scalar_bytes).unwrap_or(bls12_381::Scalar::one());
    
    G2Projective::generator() * scalar
}

#[cfg(test)]
mod tests {
    use super::*;
    use rco_crypto::bls::PrivateShare;
    use rand::thread_rng;

    #[test]
    fn test_por_construction_and_signing() {
        let weight_hash = [1u8; 32];
        let merkle_link = [2u8; 32];
        let batch = b"telemetry_data";

        let message_point = construct_por_message(weight_hash, merkle_link, batch);

        let mut rng = thread_rng();
        let share = PrivateShare::random(&mut rng);

        // Sign the PoR message
        let _signature = share.sign(&message_point);
    }
}
