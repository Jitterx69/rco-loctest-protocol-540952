//! Active Manifold Lasering
//!
//! Applies simplicial gradient flow to active agent trajectories to enforce topological coherence.

use nalgebra::DVector;

/// Configuration for the Manifold Lasering loop.
pub struct LaseringConfig {
    /// The alignment gain $c_3$ ($\alpha$)
    pub alignment_gain: f64,
    /// Damping factor to prevent Reflexive Oscillation (F-260)
    pub damping_factor: f64,
    /// Landmark threshold multiplier to mitigate Manifold Ghosting (F-251)
    pub landmark_sigma_multiplier: f64,
}

impl Default for LaseringConfig {
    fn default() -> Self {
        Self {
            alignment_gain: 0.05,
            damping_factor: 0.95,
            landmark_sigma_multiplier: 3.0,
        }
    }
}

/// Applies a single step of damped lasering to the agent's state vector.
/// $\frac{\partial s}{\partial t} = -\nabla \mathcal{L}_{topo}(s) + \alpha \cdot \text{Align}(\mathcal{M}_{ref})$
pub fn apply_damped_lasering(
    agent_state: &mut DVector<f64>,
    topological_gradient: &DVector<f64>,
    alignment_vector: &DVector<f64>,
    velocity: &mut DVector<f64>,
    config: &LaseringConfig,
) {
    // 1. Compute the force applied by the simplicial gradient and alignment vector
    let force = -(topological_gradient.clone()) + alignment_vector * config.alignment_gain;
    
    // 2. Update velocity with damping (F-260 mitigation)
    *velocity = (velocity.clone() + force) * config.damping_factor;
    
    // 3. Update state
    *agent_state += velocity.clone();
}

/// Filters landmarks that are below the persistence threshold (F-251 mitigation).
/// Returns true if the feature lifetime exceeds the noise floor.
pub fn is_landmark_valid(lifetime: f64, noise_sigma: f64, config: &LaseringConfig) -> bool {
    lifetime > (config.landmark_sigma_multiplier * noise_sigma)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damped_lasering() {
        let config = LaseringConfig::default();
        let mut state = DVector::from_vec(vec![0.0, 0.0]);
        let mut velocity = DVector::from_vec(vec![0.0, 0.0]);
        let topo_grad = DVector::from_vec(vec![0.1, -0.1]);
        let align_vec = DVector::from_vec(vec![1.0, 1.0]);
        
        apply_damped_lasering(&mut state, &topo_grad, &align_vec, &mut velocity, &config);
        
        // Velocity should be updated and damped
        assert_eq!(velocity[0], (-0.1 + 0.05) * 0.95);
        assert_eq!(state[0], velocity[0]);
    }

    #[test]
    fn test_landmark_thresholding() {
        let config = LaseringConfig::default();
        assert!(!is_landmark_valid(2.0, 1.0, &config)); // 2.0 < 3 * 1.0
        assert!(is_landmark_valid(4.0, 1.0, &config));  // 4.0 > 3 * 1.0
    }
}
