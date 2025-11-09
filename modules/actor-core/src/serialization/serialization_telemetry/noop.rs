//! No-op telemetry implementation used as a default placeholder.

use core::time::Duration;

use crate::serialization::{FieldPathHash, SerializationError};

use super::SerializationTelemetry;

/// Telemetry handler that discards all events.
pub struct NoopSerializationTelemetry;

impl NoopSerializationTelemetry {
  /// Creates a new telemetry handler that performs no work.
  #[must_use]
  pub const fn new() -> Self {
    Self
  }
}

impl SerializationTelemetry for NoopSerializationTelemetry {
  fn on_aggregate_start(&self) {}

  fn on_aggregate_finish(&self) {}

  fn record_latency(&self, _field_path_hash: FieldPathHash, _elapsed: Duration) {}

  fn record_success(&self, _field_path_hash: FieldPathHash) {}

  fn record_failure(&self, _field_path_hash: FieldPathHash, _error: &SerializationError) {}
}
