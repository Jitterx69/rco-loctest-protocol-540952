//! Riemannian Gradient Consensus (RGC)
//!
//! Implements localized manifold synchronization using exponential/logarithm maps.

use nalgebra::DVector;

/// Parameters for the RGC convergence.
pub struct RGCConfig {
    /// Learning rate (alpha)
    pub step_size: f64,
    /// Lyapunov damping factor
    pub damping: f64,
}

impl Default for RGCConfig {
    fn default() -> Self {
        Self {
            step_size: 0.1,
            damping: 0.05,
        }
    }
}

/// The RGC engine.
pub struct RGCEngine {
    config: RGCConfig,
}

impl RGCEngine {
    /// Creates a new RGC engine.
    pub fn new(config: RGCConfig) -> Self {
        Self { config }
    }

    /// Simulates the Riemannian Exponential Map.
    /// In a real system, this projects a tangent vector back onto the manifold.
    pub fn exp_map(&self, point: &DVector<f64>, tangent: &DVector<f64>) -> DVector<f64> {
        point + tangent
    }

    /// Simulates the Riemannian Logarithm Map.
    /// Returns the tangent vector at `p1` that points toward `p2`.
    pub fn log_map(&self, p1: &DVector<f64>, p2: &DVector<f64>) -> DVector<f64> {
        p2 - p1
    }

    /// Updates a node's state estimate by moving toward the average simplicial gradient.
    pub fn update_state(
        &self,
        current_state: &DVector<f64>,
        neighbor_states: &[DVector<f64>],
    ) -> DVector<f64> {
        if neighbor_states.is_empty() {
            return current_state.clone();
        }

        // Calculate average log map (Riemannian gradient)
        let mut avg_tangent = DVector::from_element(current_state.len(), 0.0);
        for neighbor in neighbor_states {
            avg_tangent += self.log_map(current_state, neighbor);
        }
        avg_tangent /= neighbor_states.len() as f64;

        // Apply step size and damping
        let update_vector = avg_tangent * self.config.step_size * (1.0 - self.config.damping);
        
        self.exp_map(current_state, &update_vector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgc_convergence() {
        let engine = RGCEngine::new(RGCConfig::default());
        let mut node_state = DVector::from_vec(vec![0.0, 0.0]);
        let neighbors = vec![
            DVector::from_vec(vec![1.0, 1.0]),
            DVector::from_vec(vec![2.0, 0.0]),
        ];

        // Perform one update
        node_state = engine.update_state(&node_state, &neighbors);
        
        // Expected average toward (1.5, 0.5)
        // With step_size 0.1 and damping 0.05, move is ~0.095 * (1.5, 0.5)
        assert!(node_state[0] > 0.1);
        assert!(node_state[1] > 0.04);
    }
}
