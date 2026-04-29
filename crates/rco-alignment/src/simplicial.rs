//! Simplicial Flow and Differentiable Homology Surrogate
//!
//! Implements the 1st Simplicial Laplacian $\Delta_1$ and the Acceleration-Based Surrogate (ABS)
//! for topological alignment without requiring full autodiff over persistence diagrams.

use nalgebra::{DMatrix, DVector};

/// Represents the Acceleration-Based Surrogate (ABS) for topological loss.
/// $\mathcal{L}_{topo} = \frac{\|\ddot{s}_t\|^2}{\|\dot{s}_t\|^2 + \epsilon} \cdot \exp(-\text{Entropy}(PD))$
pub struct AccelerationSurrogate {
    /// Epsilon value to prevent division by zero
    pub epsilon: f64,
}

impl AccelerationSurrogate {
    /// Creates a new ABS calculator.
    pub fn new(epsilon: f64) -> Self {
        Self { epsilon }
    }

    /// Computes the surrogate loss given the velocity $\dot{s}_t$, acceleration $\ddot{s}_t$,
    /// and the diagram entropy.
    pub fn compute_loss(&self, velocity: &DVector<f64>, acceleration: &DVector<f64>, diagram_entropy: f64) -> f64 {
        let v_norm_sq = velocity.norm_squared();
        let a_norm_sq = acceleration.norm_squared();
        
        let kinetic_term = a_norm_sq / (v_norm_sq + self.epsilon);
        let topological_weight = (-diagram_entropy).exp();
        
        kinetic_term * topological_weight
    }

    /// Computes the gradient of the surrogate loss with respect to the acceleration vector.
    /// Used for backpropagation in the agent's policy update.
    pub fn compute_gradient(&self, velocity: &DVector<f64>, acceleration: &DVector<f64>, diagram_entropy: f64) -> DVector<f64> {
        let v_norm_sq = velocity.norm_squared();
        let topological_weight = (-diagram_entropy).exp();
        let factor = 2.0 * topological_weight / (v_norm_sq + self.epsilon);
        
        acceleration * factor
    }
}

/// Computes the 1st Simplicial Laplacian $\Delta_1 = \partial_2 \partial_2^* + \partial_1^* \partial_1$
/// For our telemetry graph, we approximate the spectral gap by looking at the graph Laplacian (since telemetry is sequential).
pub fn compute_graph_laplacian(adjacency_matrix: &DMatrix<f64>) -> DMatrix<f64> {
    let n = adjacency_matrix.nrows();
    let mut degree_matrix = DMatrix::zeros(n, n);
    for i in 0..n {
        let sum: f64 = adjacency_matrix.row(i).iter().sum();
        degree_matrix[(i, i)] = sum;
    }
    degree_matrix - adjacency_matrix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abs_surrogate() {
        let abs = AccelerationSurrogate::new(1e-6);
        let v = DVector::from_vec(vec![1.0, 0.5, -0.2]);
        let a = DVector::from_vec(vec![0.1, -0.1, 0.05]);
        
        let loss = abs.compute_loss(&v, &a, 1.2);
        assert!(loss > 0.0);
        
        let grad = abs.compute_gradient(&v, &a, 1.2);
        assert_eq!(grad.len(), 3);
    }
}
