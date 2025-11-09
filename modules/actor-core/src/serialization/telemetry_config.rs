//! Configuration shared across serialization telemetry hooks.

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Tunable configuration influencing telemetry emission.
pub struct TelemetryConfig {
  latency_threshold_us: AtomicU64,
  debug_trace_enabled:  AtomicBool,
}

impl TelemetryConfig {
  /// Default latency threshold (microseconds) before emitting latency events.
  pub const DEFAULT_LATENCY_THRESHOLD_US: u64 = 250;

  /// Creates a new config with the provided latency threshold and debug flag.
  #[must_use]
  pub const fn new(latency_threshold_us: u64, debug_trace_enabled: bool) -> Self {
    Self {
      latency_threshold_us: AtomicU64::new(latency_threshold_us),
      debug_trace_enabled:  AtomicBool::new(debug_trace_enabled),
    }
  }

  /// Returns the latency threshold in microseconds.
  #[must_use]
  pub fn latency_threshold_us(&self) -> u64 {
    self.latency_threshold_us.load(Ordering::Relaxed)
  }

  /// Updates the latency threshold in microseconds.
  pub fn set_latency_threshold_us(&self, threshold: u64) {
    self.latency_threshold_us.store(threshold, Ordering::Relaxed);
  }

  /// Indicates whether debug tracing is enabled.
  #[must_use]
  pub fn debug_trace_enabled(&self) -> bool {
    self.debug_trace_enabled.load(Ordering::Relaxed)
  }

  /// Toggles debug tracing.
  pub fn set_debug_trace_enabled(&self, enabled: bool) {
    self.debug_trace_enabled.store(enabled, Ordering::Relaxed);
  }
}

impl Default for TelemetryConfig {
  fn default() -> Self {
    Self::new(Self::DEFAULT_LATENCY_THRESHOLD_US, false)
  }
}
