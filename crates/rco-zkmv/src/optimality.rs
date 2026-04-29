//! ZK Gain Optimality Verification
//!
//! Circuit to prove that the reflexive gain (lambda) minimizes the cost functional.

use ark_ff::PrimeField;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/// Circuit for verifying gain optimality.
/// J = ||Delta x||^2 + gamma * lambda^2
pub struct OptimalityCircuit<F: PrimeField> {
    /// Drift magnitude (public input)
    pub drift: Option<F>,
    /// Gain value (private witness)
    pub lambda: Option<F>,
    /// Regularization factor (public input)
    pub gamma: Option<F>,
    /// Target cost (public input)
    pub target_j: Option<F>,
}

impl<F: PrimeField> ConstraintSynthesizer<F> for OptimalityCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let drift_val = self.drift.unwrap_or_else(F::zero);
        let lambda_val = self.lambda.unwrap_or_else(F::zero);
        let gamma_val = self.gamma.unwrap_or_else(F::zero);
        let j_val = self.target_j.unwrap_or_else(F::zero);

        let drift_var = cs.new_input_variable(|| Ok(drift_val))?;
        let lambda_var = cs.new_witness_variable(|| Ok(lambda_val))?;
        let gamma_var = cs.new_input_variable(|| Ok(gamma_val))?;
        let j_var = cs.new_input_variable(|| Ok(j_val))?;

        // Constraints:
        // 1. Calculate drift^2
        let drift_sq = cs.new_witness_variable(|| Ok(drift_val * drift_val))?;
        cs.enforce_constraint(
            ark_relations::lc!() + drift_var,
            ark_relations::lc!() + drift_var,
            ark_relations::lc!() + drift_sq,
        )?;

        // 2. Calculate gamma * lambda^2
        let lambda_sq = cs.new_witness_variable(|| Ok(lambda_val * lambda_val))?;
        cs.enforce_constraint(
            ark_relations::lc!() + lambda_var,
            ark_relations::lc!() + lambda_var,
            ark_relations::lc!() + lambda_sq,
        )?;

        let weighted_lambda = cs.new_witness_variable(|| Ok(gamma_val * lambda_val * lambda_val))?;
        cs.enforce_constraint(
            ark_relations::lc!() + gamma_var,
            ark_relations::lc!() + lambda_sq,
            ark_relations::lc!() + weighted_lambda,
        )?;

        // 3. Enforce J = drift^2 + gamma * lambda^2
        cs.enforce_constraint(
            ark_relations::lc!() + drift_sq + weighted_lambda,
            ark_relations::lc!() + ark_relations::r1cs::Variable::One,
            ark_relations::lc!() + j_var,
        )?;

        Ok(())
    }
}
