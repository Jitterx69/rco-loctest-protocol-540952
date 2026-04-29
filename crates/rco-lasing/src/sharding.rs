//! Synchrony Sharding
//!
//! Partitioning the manifold and maintaining cross-shard phase locking.

use rco_types::HashDigest;

/// Represents a manifold shard.
pub struct ManifoldShard {
    pub shard_id: u64,
    /// Reference frequency for phase locking (omega_ref)
    pub ref_freq: f64,
    /// Current internal phase
    pub phase: f64,
}

impl ManifoldShard {
    /// Synchronizes the shard phase with the global reference.
    pub fn phase_lock(&mut self, global_ref: f64) {
        // Simple Phase-Locked Loop (PLL) approximation
        let delta = global_ref - self.phase;
        self.phase += delta * 0.1; // Locking gain
    }

    /// Generates a shard anchor hash anchored to the phase.
    pub fn generate_anchor(&self, state_hash: HashDigest) -> HashDigest {
        // Salt the state hash with the phase-derived value
        let mut salted = state_hash;
        let phase_bytes = (self.phase * 1e9) as u64;
        for (i, b) in phase_bytes.to_le_bytes().iter().enumerate() {
            salted[i % 32] ^= b;
        }
        salted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_locking() {
        let mut shard = ManifoldShard {
            shard_id: 1,
            ref_freq: 1.0,
            phase: 0.0,
        };
        
        shard.phase_lock(1.0);
        assert!(shard.phase > 0.0);
        
        for _ in 0..200 {
            shard.phase_lock(1.0);
        }
        
        // Should be converged
        assert!((shard.phase - 1.0).abs() < 1e-6);
    }
}
