//! Lasing Controller
//!
//! Governs the reflexive feedback force and manifold coherence.

use nalgebra::{DMatrix, DVector};
use rco_reflexive::jacobian::ReflexiveJacobian;

/// Represents the Lasing Controller.
pub struct LasingController {
    /// The Lasing Constant (Lambda) - Proportional Gain
    pub lambda: f64,
    /// Integral Gain (K_i)
    pub ki: f64,
    /// Reflexive/Predictive Gain (K_r)
    pub kr: f64,
    /// Accumulated homological error (Integral)
    pub integral_error: f64,
    /// Previous drift (for Derivative/Reflexive calculation)
    pub prev_drift: f64,
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
            ki: 0.01, // Default integral coefficient
            kr: 0.05, // Default reflexive coefficient
            integral_error: 0.0,
            prev_drift: 0.0,
            gain,
            rj: ReflexiveJacobian::new(param_dim, obs_dim),
        }
    }

    /// Computes the coherent force update using PID-Reflexive Loop.
    pub fn compute_lasing_force(&mut self, epsilon: &DVector<f64>, jacobian: &DMatrix<f64>) -> DVector<f64> {
        let current_drift = epsilon.norm();
        
        // 1. Proportional: Instantaneous drift correction
        let jt = jacobian.transpose();
        let gradient = jt * epsilon;
        
        // 2. Integral: Summed errors over time
        self.integral_error += current_drift;
        
        // 3. Reflexive: Predictive compensation based on curvature change
        let reflexive_comp = current_drift - self.prev_drift;
        self.prev_drift = current_drift;

        // Total Optimization Force: F = -[lambda*P + ki*I + kr*R] * G * Gradient
        let total_gain = (self.lambda + self.ki * self.integral_error + self.kr * reflexive_comp) * self.gain;
        
        // Level-1 Safety: Gain Clamp (TC-III-14 override)
        let clamped_gain = total_gain.clamp(0.0, 5.0);

        -(gradient * clamped_gain)
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
        let mut controller = LasingController::new(3, 2, 0.5, 1.0);
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
