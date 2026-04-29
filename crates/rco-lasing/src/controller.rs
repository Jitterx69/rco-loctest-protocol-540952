//! Lasing Controller
//!
//! Governs the reflexive feedback force and manifold coherence.

use nalgebra::{DMatrix, DVector};
use rco_reflexive::jacobian::ReflexiveJacobian;
use crate::rfc::RecursiveFeedbackController;
use crate::damper::ActiveResonantDamper;

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
    /// Recursive Feedback Controller (Phase-IV)
    pub rfc: RecursiveFeedbackController,
    /// Active Resonant Damper (Phase-IV)
    pub ard: ActiveResonantDamper,
}

impl LasingController {
    /// Creates a new LasingController.
    pub fn new(param_dim: usize, obs_dim: usize, lambda: f64, gain: f64) -> Self {
        Self {
            lambda,
            ki: 0.01,
            kr: 0.05,
            integral_error: 0.0,
            prev_drift: 0.0,
            gain,
            rj: ReflexiveJacobian::new(param_dim, obs_dim),
            rfc: RecursiveFeedbackController::new(),
            ard: ActiveResonantDamper::new(),
        }
    }

    /// Computes the coherent force update using PID-Reflexive Loop and ARD.
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

        // 4. RFC: Hierarchical gain synchronization
        let target_gain = self.rfc.synchronize_step(self.lambda, 0.99, current_drift);
        
        // Total Optimization Force: F = -[target_gain*P + ki*I + kr*R] * G * Gradient
        let total_gain = (target_gain + self.ki * self.integral_error + self.kr * reflexive_comp) * self.gain;
        
        // Level-1 Safety: Gain Clamp
        let clamped_gain = total_gain.clamp(0.0, 5.0);

        let mut force = -(gradient * clamped_gain);

        // 5. ARD: Mode 3.4 Harmonic Suppression
        let echoes = self.ard.detect_echoes(&force);
        let counter_pulse = self.ard.generate_counter_pulse(&echoes);
        
        // Apply counter-pulse if dimensions match (simplified)
        if counter_pulse.len() == force.len() {
            force += counter_pulse;
        }

        // 6. RMC: Riemannian Manifold Contraction
        self.apply_rmc(&mut force, jacobian);

        force
    }

    /// Riemannian Manifold Contraction (RMC): Neutralizes Gradient Black Holes.
    /// Pulls vectors away from singularities where |g| -> 0.
    fn apply_rmc(&self, force: &mut DVector<f64>, jacobian: &DMatrix<f64>) {
        // Metric tensor g = J^T * J
        let g = jacobian.transpose() * jacobian;
        let det_g = g.determinant();

        // If approaching singularity threshold (|g| < 1e-9)
        if det_g.abs() < 1e-9 {
            // Apply "Geometric Tension" - reverse the force direction to escape singularity
            *force *= -0.5;
        }
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
