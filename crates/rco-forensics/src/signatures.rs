//! Forensic Bit-Signatures
//!
//! Detects Byzantine Evasion modes.

use crate::tda::{TdaEngine, TelemetryPoint};

/// Forensic alert types
#[derive(Debug, PartialEq, Eq)]
pub enum ForensicAlert {
    /// F-51: Sub-Threshold Commitment Attempt
    QuorumUnderflow,
    /// F-60: Cognitive Mirroring (Identity Forgery)
    RedundantCognitiveHash,
    /// F-61: Low-Entropy Telemetry Injection
    TdaPersistentStasis,
    /// F-64: Delayed Witness Disclosure
    SystematicLatencyOutlier,
}

/// Evaluates a stream of telemetry for anomalies.
pub fn evaluate_forensics(engine: &TdaEngine, window: &[TelemetryPoint]) -> Option<ForensicAlert> {
    let current_betti = engine.evaluate_manifold(window);
    let d_b = engine.bottleneck_distance(&current_betti);

    if d_b > 2.0 {
        return Some(ForensicAlert::TdaPersistentStasis); // Simulating F-61
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tda::{TdaEngine, BettiNumbers, TelemetryPoint};

    #[test]
    fn test_tfs01_forensic_manifold_detection() {
        // Baseline: b0=1, b1=0
        let engine = TdaEngine::new(BettiNumbers { b0: 1, b1: 0 });

        // Simulate normal telemetry
        let normal_window = vec![TelemetryPoint { vector: vec![0.1, 0.2] }];
        assert_eq!(evaluate_forensics(&engine, &normal_window), None);

        // Simulate anomaly F-61 (TdaPersistentStasis) by injecting a large distance in the mock engine.
        // For the sake of the test, we'll create an engine with a baseline that triggers the alert.
        let anomalous_engine = TdaEngine::new(BettiNumbers { b0: 5, b1: 0 }); // d_b will be 4.0 > 2.0
        let alert = evaluate_forensics(&anomalous_engine, &normal_window);
        assert_eq!(alert, Some(ForensicAlert::TdaPersistentStasis));
    }
}
