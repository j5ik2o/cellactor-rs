//! Telemetry service that emits serialization runtime events.

use core::time::Duration;

use cellactor_utils_core_rs::sync::ArcShared;

use super::{
  SerializationError, SerializationTelemetry, field_path_hash::FieldPathHash, telemetry_config::TelemetryConfig,
  telemetry_counters::TelemetryCounters,
};
use crate::{
  RuntimeToolbox,
  event_stream::{
    EventStreamEvent, EventStreamGeneric, SerializationEvent, SerializationEventKind, SerializationFailureKind,
  },
};

#[cfg(test)]
mod tests;

/// Concrete telemetry backend wired to the runtime event stream.
pub struct TelemetryService<TB: RuntimeToolbox + 'static> {
  event_stream: ArcShared<EventStreamGeneric<TB>>,
  config:       TelemetryConfig,
  counters:     TelemetryCounters,
}

impl<TB: RuntimeToolbox + 'static> TelemetryService<TB> {
  /// Creates a new telemetry service for the provided event stream.
  #[must_use]
  pub fn new(event_stream: ArcShared<EventStreamGeneric<TB>>, config: TelemetryConfig) -> Self {
    Self { event_stream, config, counters: TelemetryCounters::new() }
  }

  /// Returns the mutable telemetry configuration.
  #[must_use]
  pub fn config(&self) -> &TelemetryConfig {
    &self.config
  }

  fn publish(&self, event: SerializationEvent) {
    self.event_stream.publish(&EventStreamEvent::Serialization(event));
  }

  fn failure_kind(error: &SerializationError) -> SerializationFailureKind {
    match error {
      | SerializationError::NoSerializerForType(_) => SerializationFailureKind::MissingSerializer,
      | SerializationError::InvalidAggregateSchema(_) => SerializationFailureKind::InvalidAggregate,
      | SerializationError::SerializationFailed(_) => SerializationFailureKind::SerializationFailed,
      | SerializationError::DeserializationFailed(_) => SerializationFailureKind::DeserializationFailed,
      | _ => SerializationFailureKind::Other,
    }
  }
}

impl<TB: RuntimeToolbox + 'static> SerializationTelemetry for TelemetryService<TB> {
  fn on_aggregate_start(&self) {}

  fn on_aggregate_finish(&self) {}

  fn record_latency(&self, _field_path_hash: FieldPathHash, _elapsed: Duration) {
    // Latency publishing will be implemented once threshold logic (Task 3.1) is active.
  }

  fn record_success(&self, field_path_hash: FieldPathHash) {
    let _ = self.counters.record_success();
    let event = SerializationEvent::new(field_path_hash, SerializationEventKind::Success);
    self.publish(event);
  }

  fn record_failure(&self, field_path_hash: FieldPathHash, error: &SerializationError) {
    let _ = self.counters.record_failure();
    let kind = Self::failure_kind(error);
    let event = SerializationEvent::new(field_path_hash, SerializationEventKind::Failure(kind));
    self.publish(event);
  }
}
