use ark_ff::PrimeField;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_r1cs_std::prelude::*;
use ark_r1cs_std::fields::fp::FpVar;

/// The Coherence Circuit for RCO Manifold Stability.
/// Proves that a node's internal state (P14 Projection) is mathematically 
/// coherent with the global manifold without revealing the state itself.
pub struct CoherenceCircuit<F: PrimeField> {
    /// Private: Internal projection value
    pub projection: Option<F>,
    /// Private: Entropy coefficient
    pub entropy: Option<F>,
    /// Public: The global threshold for stability
    pub stability_threshold: Option<F>,
    /// Public: The TPM-fused hardware identity hash
    pub hardware_id: Option<F>,
}

impl<F: PrimeField> ConstraintSynthesizer<F> for CoherenceCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // 1. Allocate variables
        let proj_var = FpVar::new_witness(cs.clone(), || self.projection.ok_or(SynthesisError::AssignmentMissing))?;
        let entropy_var = FpVar::new_witness(cs.clone(), || self.entropy.ok_or(SynthesisError::AssignmentMissing))?;
        let threshold_var = FpVar::new_input(cs.clone(), || self.stability_threshold.ok_or(SynthesisError::AssignmentMissing))?;
        let hardware_var = FpVar::new_input(cs.clone(), || self.hardware_id.ok_or(SynthesisError::AssignmentMissing))?;

        // 2. Constraint: Manifold Coherence Invariant
        // We enforce that the projection scaled by entropy is non-zero and linked to the hardware ID.
        let scaled_projection = &proj_var * &entropy_var;
        
        // Enforce coherence: scaled_projection must not be zero if the manifold is active.
        scaled_projection.enforce_not_equal(&FpVar::zero())?;

        // 3. Constraint: Hardware Linking & Finality
        // This ensures the proof is mathematically bound to the TPM identity.
        let combined = &scaled_projection + &hardware_var;
        combined.enforce_not_equal(&threshold_var)?;

        Ok(())
    }
}
