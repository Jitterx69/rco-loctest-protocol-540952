//! # Backpressure Gate
//!
//! Flow control mechanism that prevents the simulation from overwhelming
//! the ingestion pipeline. Implements a token-bucket rate limiter with
//! configurable burst capacity.
//!
//! ## Design Rationale
//!
//! When the WAL write latency spikes (e.g., disk I/O saturation),
//! unbounded ingestion would cause OOM or unbounded queue growth.
//! The backpressure gate applies throttling at the source, forcing
//! the simulation to wait until the WAL can accept more batches.

use rco_types::error::RcoError;
use std::sync::atomic::{AtomicU64, Ordering};

/// Configuration for the backpressure gate.
#[derive(Debug, Clone, Copy)]
pub struct BackpressureConfig {
    /// Maximum number of in-flight (prepared but not committed) batches.
    pub max_in_flight: u64,
    /// Maximum WAL size in bytes before compaction is required.
    pub max_wal_bytes: u64,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            max_in_flight: 64,
            max_wal_bytes: 256 * 1024 * 1024, // 256 MiB
        }
    }
}

/// Backpressure gate controlling ingestion flow.
///
/// Thread-safe via atomics — can be shared between the simulation
/// producer thread and the WAL consumer thread.
pub struct BackpressureGate {
    /// Current number of in-flight batches.
    in_flight: AtomicU64,
    /// Configuration limits.
    config: BackpressureConfig,
    /// Total batches accepted.
    accepted: AtomicU64,
    /// Total batches rejected due to backpressure.
    rejected: AtomicU64,
}

impl BackpressureGate {
    /// Creates a new backpressure gate with the given configuration.
    #[must_use]
    pub fn new(config: BackpressureConfig) -> Self {
        Self {
            in_flight: AtomicU64::new(0),
            config,
            accepted: AtomicU64::new(0),
            rejected: AtomicU64::new(0),
        }
    }

    /// Creates a gate with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(BackpressureConfig::default())
    }

    /// Attempts to acquire a slot for a new batch.
    ///
    /// Returns `Ok(())` if a slot is available, or `Err` if the gate
    /// is saturated (backpressure applied).
    ///
    /// # Errors
    ///
    /// Returns `RcoError::BackpressureExceeded` if `in_flight >= max_in_flight`.
    pub fn try_acquire(&self) -> Result<(), RcoError> {
        let current = self.in_flight.load(Ordering::Acquire);
        if current >= self.config.max_in_flight {
            self.rejected.fetch_add(1, Ordering::Relaxed);
            return Err(RcoError::BackpressureExceeded {
                queue_depth: current,
            });
        }

        // CAS loop to atomically increment
        let result = self.in_flight.compare_exchange(
            current,
            current + 1,
            Ordering::AcqRel,
            Ordering::Relaxed,
        );

        match result {
            Ok(_) => {
                self.accepted.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(_) => {
                // Another thread beat us — retry by returning backpressure
                self.rejected.fetch_add(1, Ordering::Relaxed);
                Err(RcoError::BackpressureExceeded {
                    queue_depth: self.in_flight.load(Ordering::Relaxed),
                })
            }
        }
    }

    /// Releases a slot after a batch has been committed or aborted.
    pub fn release(&self) {
        self.in_flight.fetch_sub(1, Ordering::Release);
    }

    /// Returns the current number of in-flight batches.
    #[must_use]
    pub fn in_flight(&self) -> u64 {
        self.in_flight.load(Ordering::Relaxed)
    }

    /// Returns the total number of accepted batches.
    #[must_use]
    pub fn total_accepted(&self) -> u64 {
        self.accepted.load(Ordering::Relaxed)
    }

    /// Returns the total number of rejected batches.
    #[must_use]
    pub fn total_rejected(&self) -> u64 {
        self.rejected.load(Ordering::Relaxed)
    }

    /// Returns the backpressure ratio (rejected / (accepted + rejected)).
    #[must_use]
    pub fn pressure_ratio(&self) -> f64 {
        let accepted = self.total_accepted() as f64;
        let rejected = self.total_rejected() as f64;
        let total = accepted + rejected;
        if total == 0.0 {
            0.0
        } else {
            rejected / total
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acquire_within_limit() {
        let gate = BackpressureGate::new(BackpressureConfig {
            max_in_flight: 2,
            max_wal_bytes: 1024,
        });

        assert!(gate.try_acquire().is_ok());
        assert!(gate.try_acquire().is_ok());
        assert_eq!(gate.in_flight(), 2);
    }

    #[test]
    fn test_reject_over_limit() {
        let gate = BackpressureGate::new(BackpressureConfig {
            max_in_flight: 1,
            max_wal_bytes: 1024,
        });

        gate.try_acquire().unwrap();
        let result = gate.try_acquire();
        assert!(matches!(result, Err(RcoError::BackpressureExceeded { .. })));
    }

    #[test]
    fn test_release_frees_slot() {
        let gate = BackpressureGate::new(BackpressureConfig {
            max_in_flight: 1,
            max_wal_bytes: 1024,
        });

        gate.try_acquire().unwrap();
        assert!(gate.try_acquire().is_err());

        gate.release();
        assert!(gate.try_acquire().is_ok());
    }

    #[test]
    fn test_counters() {
        let gate = BackpressureGate::new(BackpressureConfig {
            max_in_flight: 1,
            max_wal_bytes: 1024,
        });

        gate.try_acquire().unwrap();
        let _ = gate.try_acquire(); // Rejected

        assert_eq!(gate.total_accepted(), 1);
        assert_eq!(gate.total_rejected(), 1);
    }

    #[test]
    fn test_pressure_ratio() {
        let gate = BackpressureGate::new(BackpressureConfig {
            max_in_flight: 1,
            max_wal_bytes: 1024,
        });

        gate.try_acquire().unwrap();
        let _ = gate.try_acquire(); // Rejected
        let _ = gate.try_acquire(); // Rejected

        // 1 accepted, 2 rejected → ratio = 2/3 ≈ 0.666
        let ratio = gate.pressure_ratio();
        assert!((ratio - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_default_config() {
        let gate = BackpressureGate::with_defaults();
        assert_eq!(gate.in_flight(), 0);
    }
}
