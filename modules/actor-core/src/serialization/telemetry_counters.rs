//! Lock-free counters tracking serialization outcomes.

use portable_atomic::{AtomicU64, Ordering};

/// Aggregated success/failure counters exposed via telemetry.
pub struct TelemetryCounters {
  success_total: AtomicU64,
  failure_total: AtomicU64,
  external_success_total: AtomicU64,
  external_failure_total: AtomicU64,
}

impl TelemetryCounters {
  /// Creates a new counter set initialised to zero.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      success_total: AtomicU64::new(0),
      failure_total: AtomicU64::new(0),
      external_success_total: AtomicU64::new(0),
      external_failure_total: AtomicU64::new(0),
    }
  }

  /// Increments the success counter and returns the new value.
  pub fn record_success(&self) -> u64 {
    self.success_total.fetch_add(1, Ordering::Relaxed) + 1
  }

  /// Increments the failure counter and returns the new value.
  pub fn record_failure(&self) -> u64 {
    self.failure_total.fetch_add(1, Ordering::Relaxed) + 1
  }

  /// Increments the external success counter and returns the new value.
  pub fn record_external_success(&self) -> u64 {
    self.external_success_total.fetch_add(1, Ordering::Relaxed) + 1
  }

  /// Increments the external failure counter and returns the new value.
  pub fn record_external_failure(&self) -> u64 {
    self.external_failure_total.fetch_add(1, Ordering::Relaxed) + 1
  }

  /// Returns the success counter.
  #[must_use]
  pub fn success_total(&self) -> u64 {
    self.success_total.load(Ordering::Relaxed)
  }

  /// Returns the failure counter.
  #[must_use]
  pub fn failure_total(&self) -> u64 {
    self.failure_total.load(Ordering::Relaxed)
  }

  /// Returns the external success counter.
  #[must_use]
  pub fn external_success_total(&self) -> u64 {
    self.external_success_total.load(Ordering::Relaxed)
  }

  /// Returns the external failure counter.
  #[must_use]
  pub fn external_failure_total(&self) -> u64 {
    self.external_failure_total.load(Ordering::Relaxed)
  }
}

impl Default for TelemetryCounters {
  fn default() -> Self {
    Self::new()
  }
}
