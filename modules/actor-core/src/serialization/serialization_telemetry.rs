//! Telemetry hooks invoked by nested serialization orchestrators.

use core::time::Duration;

use super::{error::SerializationError, field_path_hash::FieldPathHash};

/// Records serialization outcomes for observability backends.
pub trait SerializationTelemetry: Send + Sync {
  /// Called when an aggregate serialization begins.
  fn on_aggregate_start(&self);

  /// Called when an aggregate serialization ends, regardless of success.
  fn on_aggregate_finish(&self);

  /// Records the latency spent processing a field.
  fn record_latency(&self, field_path_hash: FieldPathHash, elapsed: Duration);

  /// Records that a field finished serializing successfully.
  fn record_success(&self, field_path_hash: FieldPathHash);

  /// Records that serialization failed for a field.
  fn record_failure(&self, field_path_hash: FieldPathHash, error: &SerializationError);
}

mod noop;

pub use noop::NoopSerializationTelemetry;
