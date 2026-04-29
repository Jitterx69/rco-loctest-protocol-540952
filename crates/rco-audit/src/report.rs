//! # Audit Report Logic

use crate::scanner::AuditReport;
use std::fmt;

impl fmt::Display for AuditReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "── RCO Audit Report ──")?;
        writeln!(f, "Status:    {}", if self.corrupted_count == 0 { "PASS ✅" } else { "FAIL ❌" })?;
        writeln!(f, "Verified:  {} batches", self.verified_count)?;
        writeln!(f, "Corrupted: {} batches", self.corrupted_count)?;
        writeln!(f, "Duration:  {:?}", self.duration)?;
        writeln!(f, "Throughput: {:.2} batches/sec", self.throughput)?;
        write!(f, "Final Head: 0x")?;
        for byte in self.head_hash {
            write!(f, "{:02x}", byte)?;
        }
        writeln!(f)
    }
}
