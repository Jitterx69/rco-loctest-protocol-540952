//! Quorum Handshake and Coherence Index
//!
//! Evaluates the Global Coherence Invariant $\mathcal{C}$ across the cluster.

use rco_types::HashDigest;
use std::collections::HashMap;

/// Represents a topological diagram summary from a node.
/// In practice, this would be a full Persistence Diagram, but we abstract it for benchmarking.
#[derive(Clone, Debug, PartialEq)]
pub struct WitnessSummary {
    /// Betti-0 count (connected components)
    pub betti_0: usize,
    /// Betti-1 count (1-dimensional holes)
    pub betti_1: usize,
    /// A pseudo-Wasserstein representation
    pub w_metric: f64,
}

/// The Quorum Alignment Coordinator.
pub struct AlignmentCoordinator {
    /// Threshold tolerance for Wasserstein distance ($\epsilon_{gate}$)
    pub epsilon_gate: f64,
    /// Stored summaries from agents
    pub summaries: HashMap<u64, WitnessSummary>,
}

impl AlignmentCoordinator {
    /// Creates a new coordinator.
    pub fn new(epsilon_gate: f64) -> Self {
        Self {
            epsilon_gate,
            summaries: HashMap::new(),
        }
    }

    /// Registers a local witness summary from an agent.
    pub fn register_summary(&mut self, agent_id: u64, summary: WitnessSummary) {
        self.summaries.insert(agent_id, summary);
    }

    /// Computes the Wasserstein distance proxy between two summaries.
    pub fn wasserstein_dist_proxy(a: &WitnessSummary, b: &WitnessSummary) -> f64 {
        let b0_diff = (a.betti_0 as f64 - b.betti_0 as f64).abs();
        let b1_diff = (a.betti_1 as f64 - b.betti_1 as f64).abs();
        let w_diff = (a.w_metric - b.w_metric).abs();
        
        b0_diff + b1_diff * 1.5 + w_diff
    }

    /// Evaluates the Global Coherence Invariant $\mathcal{C}$.
    /// $\mathcal{C} = \sum_{i,j}^N d_W(PD_i, PD_j)$
    pub fn global_coherence_index(&self) -> f64 {
        let mut total_coherence = 0.0;
        let summaries: Vec<_> = self.summaries.values().collect();
        
        for i in 0..summaries.len() {
            for j in (i + 1)..summaries.len() {
                total_coherence += Self::wasserstein_dist_proxy(summaries[i], summaries[j]);
            }
        }
        total_coherence
    }

    /// Validates if the cluster is synchronized ($\mathcal{C} < \epsilon_{gate}$).
    pub fn is_synchronized(&self) -> bool {
        self.global_coherence_index() < self.epsilon_gate
    }

    /// Returns the median/centroid witness summary for an agent requesting alignment.
    pub fn fetch_centroid_witness(&self) -> Option<WitnessSummary> {
        if self.summaries.is_empty() {
            return None;
        }
        
        // Very simplified centroid: average the w_metric, take mode of betti numbers.
        // For the benchmark, we just return the first valid one if they are close, or an exact average.
        let mut sum_w = 0.0;
        let mut sum_b0 = 0;
        let mut sum_b1 = 0;
        let count = self.summaries.len() as f64;
        
        for summary in self.summaries.values() {
            sum_w += summary.w_metric;
            sum_b0 += summary.betti_0;
            sum_b1 += summary.betti_1;
        }
        
        Some(WitnessSummary {
            betti_0: (sum_b0 as f64 / count).round() as usize,
            betti_1: (sum_b1 as f64 / count).round() as usize,
            w_metric: sum_w / count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quorum_coherence() {
        let mut coordinator = AlignmentCoordinator::new(5.0);
        
        coordinator.register_summary(1, WitnessSummary { betti_0: 1, betti_1: 1, w_metric: 0.1 });
        coordinator.register_summary(2, WitnessSummary { betti_0: 1, betti_1: 1, w_metric: 0.15 });
        coordinator.register_summary(3, WitnessSummary { betti_0: 1, betti_1: 1, w_metric: 0.12 });
        
        assert!(coordinator.is_synchronized());
        
        let centroid = coordinator.fetch_centroid_witness().unwrap();
        assert_eq!(centroid.betti_0, 1);
        assert_eq!(centroid.betti_1, 1);
        
        // Inject an anomaly (F-380)
        coordinator.register_summary(4, WitnessSummary { betti_0: 2, betti_1: 5, w_metric: 9.0 });
        assert!(!coordinator.is_synchronized());
    }
}
