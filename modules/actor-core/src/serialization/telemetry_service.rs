//! Telemetry service that emits serialization runtime events.

use alloc::string::ToString;
use core::time::Duration;

use cellactor_utils_core_rs::sync::ArcShared;

use super::{
  SerializationError, SerializationTelemetry, field_path_hash::FieldPathHash, telemetry_config::TelemetryConfig,
  telemetry_counters::TelemetryCounters,
};
use crate::{
  RuntimeToolbox,
  dead_letter::DeadLetterReason,
  event_stream::{
    EventStreamEvent, SerializationDebugInfo, SerializationEvent, SerializationEventKind, SerializationFailureKind,
    SerializationFallbackReason,
  },
  messaging::AnyMessageGeneric,
  system::SystemStateGeneric,
};

#[cfg(test)]
mod tests;

/// Concrete telemetry backend wired to the runtime event stream.
pub struct TelemetryService<TB: RuntimeToolbox + 'static> {
  system_state: ArcShared<SystemStateGeneric<TB>>,
  config:       TelemetryConfig,
  counters:     TelemetryCounters,
}

impl<TB: RuntimeToolbox + 'static> TelemetryService<TB> {
  /// Creates a new telemetry service for the provided event stream.
  #[must_use]
  pub fn new(system_state: ArcShared<SystemStateGeneric<TB>>, config: TelemetryConfig) -> Self {
    Self { system_state, config, counters: TelemetryCounters::new() }
  }

  /// Returns the mutable telemetry configuration.
  #[must_use]
  pub fn config(&self) -> &TelemetryConfig {
    &self.config
  }

  /// Returns a reference to the aggregated counters.
  #[must_use]
  pub fn counters(&self) -> &TelemetryCounters {
    &self.counters
  }

  fn publish(&self, event: SerializationEvent) {
    self.system_state.publish_event(&EventStreamEvent::Serialization(event));
  }

  fn to_micros(elapsed: Duration) -> u64 {
    let micros = elapsed.as_micros();
    if micros > u64::MAX as u128 { u64::MAX } else { micros as u64 }
  }

  fn clamp_size(size_bytes: usize) -> u32 {
    if size_bytes > u32::MAX as usize { u32::MAX } else { size_bytes as u32 }
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

  fn record_latency(&self, field_path_hash: FieldPathHash, elapsed: Duration) {
    let micros = Self::to_micros(elapsed);
    if micros < self.config.latency_threshold_us() {
      return;
    }
    let event = SerializationEvent::new(field_path_hash, SerializationEventKind::Latency(micros));
    self.publish(event);
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
    if matches!(kind, SerializationFailureKind::MissingSerializer) {
      self.system_state.record_serialization_binding_error(&event);
    }
    self.publish(event);
  }

  fn record_fallback(&self, field_path_hash: FieldPathHash, reason: SerializationFallbackReason) {
    let event = SerializationEvent::new(field_path_hash, SerializationEventKind::Fallback(reason));
    self.publish(event.clone());
    let payload = AnyMessageGeneric::<TB>::new(event);
    self.system_state.record_dead_letter(payload, DeadLetterReason::ExplicitRouting, None);
  }

  fn record_external_success(&self, _field_path_hash: FieldPathHash) {
    let _ = self.counters.record_external_success();
  }

  fn record_external_failure(&self, _field_path_hash: FieldPathHash, _error: &SerializationError) {
    let _ = self.counters.record_external_failure();
  }

  fn record_debug_trace(&self, field_path_hash: FieldPathHash, manifest: &str, size_bytes: usize) {
    if !self.config.debug_trace_enabled() {
      return;
    }
    let info = SerializationDebugInfo::new(manifest.to_string(), Self::clamp_size(size_bytes));
    let event = SerializationEvent::new(field_path_hash, SerializationEventKind::DebugTrace(info));
    self.publish(event);
  }
}
