//! Lasing Controller
//!
//! Governs the reflexive feedback force and manifold coherence.

use nalgebra::{DMatrix, DVector};
use rco_reflexive::jacobian::ReflexiveJacobian;

/// Represents the Lasing Controller.
pub struct LasingController {
    /// The Lasing Constant (Lambda)
    pub lambda: f64,
    /// Topological Gain (G)
    pub gain: f64,
    /// Reflexive Jacobian engine
    pub rj: ReflexiveJacobian,
}

impl LasingController {
    /// Creates a new LasingController.
    pub fn new(param_dim: usize, obs_dim: usize, lambda: f64, gain: f64) -> Self {
        Self {
            lambda,
            gain,
            rj: ReflexiveJacobian::new(param_dim, obs_dim),
        }
    }

    /// Computes the coherent force update using Holographic Projection (Gradient-only).
    pub fn compute_lasing_force(&self, epsilon: &DVector<f64>, jacobian: &DMatrix<f64>) -> DVector<f64> {
        let jt = jacobian.transpose();
        let gradient = jt * epsilon;
        
        // Active Lasing Force: F = -Lambda * G * Gradient
        -(gradient * (self.lambda * self.gain))
    }

    /// Dynamically adjusts the gain based on coherence.
    pub fn adjust_gain(&mut self, current_coherence: f64, target_coherence: f64) {
        let drift = target_coherence - current_coherence;
        if drift > 0.0 {
            self.gain *= 1.05; // Increase gain to restore coherence
        } else {
            self.gain *= 0.95; // Reduce gain to prevent oscillation
        }
        
        // Safety bound: G < G_max
        if self.gain > 2.0 {
            self.gain = 2.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lasing_force() {
        let controller = LasingController::new(3, 2, 0.5, 1.0);
        let epsilon = DVector::from_vec(vec![0.1, -0.05]);
        let jacobian = DMatrix::from_vec(2, 3, vec![
            1.0, 0.0, 0.5,
            0.0, 1.0, 0.5
        ]);

        let force = controller.compute_lasing_force(&epsilon, &jacobian);
        assert_eq!(force.len(), 3);
        assert!(force.norm() > 0.0);
    }
}
