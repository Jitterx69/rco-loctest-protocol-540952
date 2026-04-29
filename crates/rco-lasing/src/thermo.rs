//! Simplicial Thermodynamics
//!
//! Models manifold noise as temperature and entropy.

use std::collections::HashMap;

/// Represents the thermodynamic state of the manifold.
pub struct ManifoldThermo {
    /// Topological Temperature (T_topo)
    pub temperature: f64,
    /// Manifold Entropy (S_M)
    pub entropy: f64,
}

impl ManifoldThermo {
    /// Calculates the entropy of a simplicial distribution.
    pub fn calculate_entropy(observations: &[f64]) -> f64 {
        let mut counts = HashMap::new();
        for &obs in observations {
            let bucket = (obs * 100.0) as i64;
            *counts.entry(bucket).or_insert(0) += 1;
        }

        let total = observations.len() as f64;
        let mut entropy = 0.0;
        for &count in counts.values() {
            let p = count as f64 / total;
            entropy -= p * p.ln();
        }
        entropy
    }

    /// Maxwell's Demon Pruning: Filters out high-entropy simplicial updates.
    pub fn prune_high_entropy(&self, update_entropy: f64, threshold: f64) -> bool {
        // If the new update increases entropy beyond the threshold, reject it.
        update_entropy > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_calculation() {
        let stable_obs = vec![1.0, 1.0, 1.0, 1.0];
        let unstable_obs = vec![1.0, 2.0, 3.0, 4.0];
        
        let s_stable = ManifoldThermo::calculate_entropy(&stable_obs);
        let s_unstable = ManifoldThermo::calculate_entropy(&unstable_obs);
        
        assert!(s_stable < s_unstable);
    }
}
