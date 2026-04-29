//! Spectral Isolation Monitor
//!
//! Enforces isolation bounds (Sigma) and manages manifold evaporation.

use nalgebra::DVector;

/// Represents the Spectral Isolation Monitor.
pub struct SpectralMonitor {
    /// Spectral Isolation Bound (Sigma_max)
    pub sigma_max: f64,
    /// Simplicial Density (rho)
    pub density: f64,
}

impl SpectralMonitor {
    /// Creates a new SpectralMonitor.
    pub fn new(sigma_max: f64) -> Self {
        Self {
            sigma_max,
            density: 1.0,
        }
    }

    /// Calculates the spectral bleed between two simplicial state streams.
    /// Sigma = <Si, Sj> / (||Si|| * ||Sj||)
    pub fn calculate_bleed(si: &DVector<f64>, sj: &DVector<f64>) -> f64 {
        let dot = si.dot(sj);
        let norm_product = si.norm() * sj.norm();
        if norm_product < 1e-15 {
            return 0.0;
        }
        dot.abs() / norm_product
    }

    /// Executes the Simplicial Back-off (Manifold Evaporation) protocol.
    /// Reduces density to restore isolation.
    pub fn evaporate(&mut self, current_sigma: f64) -> f64 {
        if current_sigma > self.sigma_max {
            // Logarithmic evaporation of density
            self.density *= 0.85;
        } else {
            // Gradual condensation if stable
            self.density = (self.density * 1.05).min(1.0);
        }
        self.density
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectral_bleed() {
        let s1 = DVector::from_vec(vec![1.0, 0.0, 0.0]);
        let s2 = DVector::from_vec(vec![0.0, 1.0, 0.0]);
        let s3 = DVector::from_vec(vec![0.707, 0.707, 0.0]);

        assert_eq!(SpectralMonitor::calculate_bleed(&s1, &s2), 0.0);
        assert!((SpectralMonitor::calculate_bleed(&s1, &s3) - 0.707).abs() < 1e-3);
    }

    #[test]
    fn test_manifold_evaporation() {
        let mut monitor = SpectralMonitor::new(1e-12);
        
        // High bleed scenario
        let density = monitor.evaporate(1e-8);
        assert!(density < 1.0);
        
        // Stable scenario
        let density_new = monitor.evaporate(1e-15);
        assert!(density_new > density);
    }
}
