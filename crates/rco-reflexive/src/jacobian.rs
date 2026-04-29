//! Reflexive Jacobian ($J_{re}$)
//!
//! Implements high-order operators for mapping manifold variances to 
//! parameter space updates.

use nalgebra::{DMatrix, DVector};

/// Represents the Reflexive Jacobian engine.
pub struct ReflexiveJacobian {
    /// Dimension of the parameter space (theta)
    pub param_dim: usize,
    /// Dimension of the manifold observation space (phi)
    pub obs_dim: usize,
}

impl ReflexiveJacobian {
    /// Creates a new ReflexiveJacobian engine.
    pub fn new(param_dim: usize, obs_dim: usize) -> Self {
        Self { param_dim, obs_dim }
    }

    /// Computes the parameter update delta_theta using the Holographic Projection.
    /// 
    /// delta_theta = -J_re^-1 * epsilon
    /// where epsilon is the manifold error vector.
    /// jacobian_matrix should be (obs_dim x param_dim).
    pub fn compute_update(&self, epsilon: &DVector<f64>, jacobian_matrix: &DMatrix<f64>) -> DVector<f64> {
        assert_eq!(jacobian_matrix.nrows(), self.obs_dim);
        assert_eq!(jacobian_matrix.ncols(), self.param_dim);
        
        // Holographic Projection: delta_theta = J^T * epsilon
        let damping = 0.01;
        let jt = jacobian_matrix.transpose();
        let jtj = &jt * jacobian_matrix; // (param_dim x param_dim)
        
        let mut inv_term = jtj;
        for i in 0..self.param_dim {
            inv_term[(i, i)] += damping;
        }

        let inv = inv_term.try_inverse().expect("Jacobian inversion failed");
        let update = inv * (jt * epsilon);
        
        -update
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jacobian_update() {
        let param_dim = 3;
        let obs_dim = 2;
        let rj = ReflexiveJacobian::new(param_dim, obs_dim);

        let epsilon = DVector::from_vec(vec![0.1, -0.05]);
        let jacobian = DMatrix::from_vec(obs_dim, param_dim, vec![
            1.0, 0.0, 0.5,
            0.0, 1.0, 0.5
        ]); // obs_dim (2) x param_dim (3)

        let update = rj.compute_update(&epsilon, &jacobian);
        assert_eq!(update.len(), param_dim);
        // Ensure the update moves in a direction that reduces epsilon
        // (Simplified check)
        assert!(update.norm() > 0.0);
    }
}
