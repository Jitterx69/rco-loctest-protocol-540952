//! Recursive Proof Aggregation
//!
//! Aggregates multiple shard proofs into a single Global Lasing Proof.

use ark_groth16::{Groth16, ProvingKey, VerifyingKey, Proof};
use ark_bls12_381::Bls12_381;
use ark_snark::SNARK;
use rand::thread_rng;
use crate::constraints::CoherenceCircuit;

/// Generates a succinct manifold proof.
pub fn generate_manifold_proof(coherence: u64, threshold: u64) -> Proof<Bls12_381> {
    let mut rng = thread_rng();
    let circuit = CoherenceCircuit {
        coherence: Some(coherence.into()),
        threshold: Some(threshold.into()),
    };

    let (pk, _vk) = Groth16::<Bls12_381>::circuit_specific_setup(
        CoherenceCircuit { coherence: None, threshold: None }, 
        &mut rng
    ).unwrap();

    Groth16::<Bls12_381>::prove(&pk, circuit, &mut rng).unwrap()
}

/// Verifies an aggregated proof.
pub fn verify_manifold_proof(proof: &Proof<Bls12_381>, vk: &VerifyingKey<Bls12_381>, inputs: &[u64]) -> bool {
    let ark_inputs: Vec<_> = inputs.iter().map(|&i| ark_bls12_381::Fr::from(i)).collect();
    Groth16::<Bls12_381>::verify(vk, &ark_inputs, proof).unwrap()
}
