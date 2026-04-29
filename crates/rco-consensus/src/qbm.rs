//! Quorum-Bound Manifold (QBM)
//!
//! Implements the definitive topological state logic, ensuring only
//! supermajority-supported simplices are included in the global manifold.

use std::collections::HashMap;
use rco_types::HashDigest;

/// Represents a simplex in the manifold.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Simplex {
    /// Dimension of the simplex
    pub dim: usize,
    /// Indices of the vertices
    pub vertices: Vec<u64>,
}

/// Manages the construction of the Quorum-Bound Manifold.
pub struct QBMConstructor {
    num_nodes: usize,
    /// Maps a simplex to the number of nodes supporting it
    simplex_votes: HashMap<Simplex, usize>,
}

impl QBMConstructor {
    /// Creates a new QBM constructor for a quorum of size `n`.
    pub fn new(num_nodes: usize) -> Self {
        Self {
            num_nodes,
            simplex_votes: HashMap::new(),
        }
    }

    /// Registers a set of simplices from a single node.
    pub fn register_node_contribution(&mut self, simplices: Vec<Simplex>) {
        for s in simplices {
            *self.simplex_votes.entry(s).or_insert(0) += 1;
        }
    }

    /// Finalizes the QBM by including only simplices supported by > 2/3 nodes.
    pub fn finalize_qbm(&self) -> Vec<Simplex> {
        let threshold = (2 * self.num_nodes) / 3;
        self.simplex_votes
            .iter()
            .filter(|&(_, &votes)| votes > threshold)
            .map(|(s, _)| s.clone())
            .collect()
    }

    /// Computes the QBM-Root hash (mocked for Phase-VI).
    pub fn compute_qbm_root(&self, qbm: &[Simplex]) -> HashDigest {
        // In a real system, this would hash the persistent homology of the QBM.
        let mut root = [0u8; 32];
        if !qbm.is_empty() {
            root[0] = qbm.len() as u8;
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qbm_supermajority_inclusion() {
        let mut constructor = QBMConstructor::new(3); // n=3, 2n/3 = 2, threshold > 2 => 3 votes required
        
        let s1 = Simplex { dim: 0, vertices: vec![1] };
        let s2 = Simplex { dim: 0, vertices: vec![2] };
        
        constructor.register_node_contribution(vec![s1.clone(), s2.clone()]);
        constructor.register_node_contribution(vec![s1.clone()]);
        constructor.register_node_contribution(vec![s1.clone()]);
        
        let finalized = constructor.finalize_qbm();
        
        // s1 has 3 votes (supported by all), s2 has 1 vote.
        assert!(finalized.contains(&s1));
        assert!(!finalized.contains(&s2));
    }
}
