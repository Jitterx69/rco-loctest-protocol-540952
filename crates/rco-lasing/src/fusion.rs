//! Lorentz-Invariant Manifold Fusion
//!
//! Implements Entropy-Damped State Fusion for global manifold entanglement.

use nalgebra::DVector;

/// Lorentz Invariant Fusion Kernel.
pub struct LorentzInvariantFusion {
    pub c_logical: f64, // Logical speed of information (normalized)
}

impl LorentzInvariantFusion {
    pub fn new() -> Self {
        Self {
            c_logical: 1.0,
        }
    }

    /// Computes the Lorentz Factor (gamma) for a shard with logical velocity v.
    pub fn compute_gamma(&self, v: f64) -> f64 {
        let beta = (v / self.c_logical).min(0.999);
        1.0 / (1.0 - beta * beta).sqrt()
    }

    /// Fuses multiple shard states into a unified global tensor.
    /// Uses Entropy-Damping: States with higher entropy (instability) are weighted less.
    pub fn fuse_states(&self, states: Vec<DVector<f64>>, entropies: &[f64], velocities: &[f64]) -> DVector<f64> {
        if states.is_empty() {
            return DVector::zeros(0);
        }

        let dim = states[0].len();
        let mut fused = DVector::from_element(dim, 0.0);
        let mut total_weight = 0.0;

        for (i, state) in states.iter().enumerate() {
            // Relativistic weighting: shards moving "faster" (more lag) have more temporal inertia.
            let gamma = self.compute_gamma(velocities[i]);
            
            // Entropy-damping: S is the local Ricci entropy.
            let weight = (1.0 / (1.0 + entropies[i])) * gamma;
            
            fused += state * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            fused /= total_weight;
        }

        fused
    }
}
