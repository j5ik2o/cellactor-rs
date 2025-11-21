//! Provisioning metrics collection for observability.

extern crate alloc;
extern crate std;

use alloc::vec::Vec;
use std::sync::Mutex;
use std::time::Duration;

/// Captures provisioning-related counters and timings.
pub struct ProvisioningMetrics {
  snapshot_latencies: Mutex<Vec<(u64, Duration)>>,
  failovers:          Mutex<Vec<u64>>,
  interruptions:      Mutex<Vec<u64>>,
}

impl ProvisioningMetrics {
  /// Creates an empty metrics collector.
  #[must_use]
  pub fn new() -> Self {
    Self {
      snapshot_latencies: Mutex::new(Vec::new()),
      failovers:          Mutex::new(Vec::new()),
      interruptions:      Mutex::new(Vec::new()),
    }
  }

  /// Record latency for a snapshot with its sequence number.
  pub fn record_snapshot_latency(&self, seq_no: u64, latency: Duration) {
    self.snapshot_latencies.lock().expect("poisoned").push((seq_no, latency));
  }

  /// Record a failover occurrence tagged by sequence number.
  pub fn record_failover(&self, seq_no: u64) {
    self.failovers.lock().expect("poisoned").push(seq_no);
  }

  /// Record a stream interruption tagged by sequence number.
  pub fn record_stream_interrupt(&self, seq_no: u64) {
    self.interruptions.lock().expect("poisoned").push(seq_no);
  }

  /// Expose recorded snapshot latencies (for tests/diagnostics).
  #[must_use]
  pub fn snapshot_latencies(&self) -> Vec<(u64, Duration)> {
    self.snapshot_latencies.lock().expect("poisoned").clone()
  }

  /// Expose recorded failover seq numbers.
  #[must_use]
  pub fn failovers(&self) -> Vec<u64> {
    self.failovers.lock().expect("poisoned").clone()
  }

  /// Expose recorded stream interruptions.
  #[must_use]
  pub fn interruptions(&self) -> Vec<u64> {
    self.interruptions.lock().expect("poisoned").clone()
  }
}

impl Default for ProvisioningMetrics {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests;
