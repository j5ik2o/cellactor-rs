//! Telemetry hooks invoked by nested serialization orchestrators.

use core::time::Duration;

use super::{error::SerializationError, field_path_hash::FieldPathHash};
use crate::event_stream::SerializationFallbackReason;

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

  /// Records that a fallback path was engaged for the field.
  fn record_fallback(&self, field_path_hash: FieldPathHash, reason: SerializationFallbackReason);

  /// Records that an external serializer succeeded.
  fn record_external_success(&self, field_path_hash: FieldPathHash);

  /// Records that an external serializer failed.
  fn record_external_failure(&self, field_path_hash: FieldPathHash, error: &SerializationError);

  /// Emits a debug trace entry describing the serialized payload.
  fn record_debug_trace(&self, field_path_hash: FieldPathHash, manifest: &str, size_bytes: usize);
}

mod noop;

pub use noop::NoopSerializationTelemetry;
