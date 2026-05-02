//! Hyper-Finality Proof Generation
//!
//! Generates unforgeable proofs of manifold stability linked to hardware identity.

use ark_groth16::{Groth16, ProvingKey, VerifyingKey, Proof};
use ark_bls12_381::{Bls12_381, Fr};
use ark_snark::SNARK;
use rand::thread_rng;
use crate::constraints::CoherenceCircuit;

/// Generates a forensic ZK-proof of manifold stability.
/// 
/// # Arguments
/// * `projection` - The private P14 projection value
/// * `entropy` - The private entropy coefficient
/// * `threshold` - The public stability threshold
/// * `hardware_id` - The public TPM hardware identity
pub fn generate_forensic_proof(
    projection: u64, 
    entropy: u64, 
    threshold: u64, 
    hardware_id: u64
) -> (Proof<Bls12_381>, ProvingKey<Bls12_381>, VerifyingKey<Bls12_381>) {
    let mut rng = thread_rng();
    
    let circuit = CoherenceCircuit {
        projection: Some(Fr::from(projection)),
        entropy: Some(Fr::from(entropy)),
        stability_threshold: Some(Fr::from(threshold)),
        hardware_id: Some(Fr::from(hardware_id)),
    };

    // In production, the setup should be done once (Trusted Setup).
    // Here we generate it per proof for demonstration of the "Psi" logic.
    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(
        CoherenceCircuit { 
            projection: None, 
            entropy: None, 
            stability_threshold: None, 
            hardware_id: None 
        }, 
        &mut rng
    ).unwrap();

    let proof = Groth16::<Bls12_381>::prove(&pk, circuit, &mut rng).unwrap();
    (proof, pk, vk)
}

/// Verifies a forensic manifold proof against public parameters.
pub fn verify_forensic_proof(
    proof: &Proof<Bls12_381>, 
    vk: &VerifyingKey<Bls12_381>, 
    threshold: u64, 
    hardware_id: u64
) -> bool {
    let public_inputs = vec![
        Fr::from(threshold),
        Fr::from(hardware_id),
    ];
    Groth16::<Bls12_381>::verify(vk, &public_inputs, proof).unwrap()
}
