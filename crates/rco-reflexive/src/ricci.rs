//! Simplicial Ricci Flow
//!
//! Regularizes the telemetry manifold by smoothing discrete curvature.

use nalgebra::DMatrix;

/// Regularizes a metric matrix using a discrete Ricci-flow step.
pub fn apply_ricci_flow(metric: &mut DMatrix<f64>, alpha: f64) {
    let dim = metric.nrows();
    let mut laplacian = DMatrix::zeros(dim, dim);
    
    // Discrete Laplacian approximation
    for i in 0..dim {
        for j in 0..dim {
            if i == j {
                laplacian[(i, j)] = -2.0;
            } else if (i as isize - j as isize).abs() == 1 {
                laplacian[(i, j)] = 1.0;
            }
        }
    }

    // Ricci Update: G = G + alpha * Delta * G
    let delta_g = &laplacian * &(*metric);
    *metric += delta_g * alpha;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ricci_smoothing() {
        let mut metric = DMatrix::from_vec(3, 3, vec![
            1.0, 0.0, 0.0,
            0.0, 2.0, 0.0, // High curvature peak
            0.0, 0.0, 1.0
        ]);
        
        let initial_variance = metric.as_slice().iter().map(|x| x*x).sum::<f64>();
        apply_ricci_flow(&mut metric, 0.1);
        let final_variance = metric.as_slice().iter().map(|x| x*x).sum::<f64>();
        
        // Smoothing should reduce the "energy" of the curvature peak
        assert!(final_variance < initial_variance);
    }
}
