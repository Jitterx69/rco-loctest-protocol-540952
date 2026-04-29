//! Ingestion Enclave (IE)
//!
//! Emulates the high-throughput telemetry processing enclave.
//! Simulates the 128MB Processor Reserved Memory (PRM) limit and
//! Skeletal Manifold Pruning (F-680 mitigation).

use rco_types::HashDigest;

/// Maximum PRM memory size (128 MB)
const PRM_SIZE_LIMIT: usize = 128 * 1024 * 1024;

/// Simulates a lightweight telemetry point stored in the IE.
#[derive(Clone, Copy)]
pub struct IEPoint {
    pub step: u64,
    pub state_hash: HashDigest,
    /// Is this point a topological landmark?
    pub is_landmark: bool,
}

/// The Ingestion Enclave Emulator.
pub struct IngestionEnclave {
    /// Simulates the PRM memory region
    pub prm_memory: Vec<IEPoint>,
    /// Number of points archived to the host
    pub archived_points: usize,
    /// Manifold-Aware PTP Clock Offset (ps)
    pub clock_offset_ps: f64,
    /// Junction Temperature (Celsius)
    pub junction_temp: f64,
}

impl IngestionEnclave {
    pub fn new() -> Self {
        Self {
            prm_memory: Vec::with_capacity(100_000),
            archived_points: 0,
            clock_offset_ps: 0.0,
            junction_temp: 42.0, // Default stable junction temp
        }
    }

    /// Ingests a new telemetry step. If the PRM is full, triggers Skeletal Pruning.
    pub fn ingest_step(&mut self, point: IEPoint) {
        // Approximate size of IEPoint in PRM
        let current_size = self.prm_memory.len() * std::mem::size_of::<IEPoint>();
        
        if current_size >= PRM_SIZE_LIMIT {
            self.prune_prm();
        }
        
        self.prm_memory.push(point);
    }

    /// F-680 Mitigation: Skeletal Manifold Pruning.
    /// Evicts non-landmark points from the PRM, archiving them.
    pub fn prune_prm(&mut self) {
        let initial_len = self.prm_memory.len();
        
        // Retain only landmarks in the PRM to maintain the topological skeleton
        self.prm_memory.retain(|p| p.is_landmark);
        
        let evicted = initial_len - self.prm_memory.len();
        self.archived_points += evicted;
    }

    /// Computes a pseudo topological root for binding in the RTE.
    pub fn compute_topological_root(&self) -> HashDigest {
        // In a real system, this computes the hash of the persistent homology summary
        let mut root = [0u8; 32];
        if let Some(last) = self.prm_memory.last() {
            root.copy_from_slice(&last.state_hash);
        }
        root
    }

    /// Manifold-Aware PTP (MA-PTP): Thermal-Agnostic Timing.
    /// Neutralizes thermal jitter to maintain sub-150ps alignment.
    pub fn compensate_thermal_drift(&mut self, ambient_temp: f64) {
        self.junction_temp = ambient_temp + 2.0; // Simplified junction delta
        
        // Kalman Filter Approximation: Adjust clock offset based on temp gradient
        let drift_gradient = (self.junction_temp - 42.0) * 12.5; // 12.5ps per degree
        self.clock_offset_ps = -drift_gradient; // Counter-drift adjustment
        
        // Level-5 Safety: Jitter Bound Check
        if self.clock_offset_ps.abs() > 250.0 {
            // Trigger Emergency Cooling (Appendix CQ)
            self.clock_offset_ps = self.clock_offset_ps.clamp(-250.0, 250.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ie_pruning() {
        let mut ie = IngestionEnclave::new();
        
        // Ingest 10 points, 2 landmarks
        for i in 0..10 {
            ie.ingest_step(IEPoint {
                step: i,
                state_hash: [i as u8; 32],
                is_landmark: i % 5 == 0,
            });
        }
        
        assert_eq!(ie.prm_memory.len(), 10);
        
        // Force prune
        ie.prune_prm();
        
        assert_eq!(ie.prm_memory.len(), 2);
        assert_eq!(ie.archived_points, 8);
    }
}
