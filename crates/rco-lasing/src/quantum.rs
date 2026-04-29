//! Quantum-Bound Jitter Control
//!
//! Implements Heisenberg-Invariant Gains for femtosecond-scale coordination.

use nalgebra::DVector;

/// Quantum Bound Jitter Controller.
pub struct QuantumBoundJitterController {
    /// Planck-scale jitter floor (simulated in femtoseconds)
    pub jitter_fs: f64,
    /// Heisenberg Uncertainty Parameter (delta_p * delta_q >= h_bar/2)
    pub h_bar_eff: f64,
}

impl QuantumBoundJitterController {
    pub fn new() -> Self {
        Self {
            jitter_fs: 100.0, // 100fs base
            h_bar_eff: 0.1,   // Normalized uncertainty floor
        }
    }

    /// Calculates the Heisenberg-Invariant Gain.
    /// Dampens the gain as the jitter approaches the quantum floor.
    pub fn compute_heisenberg_gain(&self, base_gain: f64, observation_noise: f64) -> f64 {
        // Heisenberg-like damping: g = g0 / (1 + h_bar / (sigma * delta_t))
        let uncertainty_factor = self.h_bar_eff / (observation_noise.max(1e-15) * self.jitter_fs);
        base_gain / (1.0 + uncertainty_factor)
    }

    /// Updates the jitter floor based on junction temperature (from IE).
    pub fn update_jitter_floor(&mut self, temp_k: f64) {
        // Linear scaling with temperature (simplified quantum thermal noise)
        // At 1.0K, we reach the ~30fs floor.
        self.jitter_fs = 30.0 * temp_k.max(0.1);
    }
}
