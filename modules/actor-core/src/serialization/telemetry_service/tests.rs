use alloc::vec::Vec;
use core::time::Duration;

use cellactor_utils_core_rs::sync::ArcShared;
use spin::Mutex;

use crate::{
  NoStdToolbox,
  dead_letter::DeadLetterReason,
  event_stream::{
    EventStream, EventStreamEvent, EventStreamSubscriber, EventStreamSubscription, SerializationEventKind,
    SerializationFallbackReason,
  },
  monitoring::serialization_runtime_monitor::SerializationRuntimeMonitor,
  serialization::{SerializationError, SerializationTelemetry, TelemetryConfig, TelemetryService},
  system::SystemStateGeneric,
};

#[test]
fn telemetry_service_emits_success_event() {
  let fixture = TelemetryFixture::new(TelemetryConfig::default());

  SerializationTelemetry::record_success(&fixture.telemetry, 11u128);

  let events = fixture.events();
  assert!(matches!(
    events.as_slice(),
    [EventStreamEvent::Serialization(event)]
      if event.field_path_hash() == 11u128
        && matches!(event.kind(), SerializationEventKind::Success)
  ));
}

#[test]
fn telemetry_service_emits_failure_event() {
  let fixture = TelemetryFixture::new(TelemetryConfig::default());

  SerializationTelemetry::record_failure(
    &fixture.telemetry,
    22u128,
    &SerializationError::NoSerializerForType("Example"),
  );

  let events = fixture.events();
  assert!(matches!(
    events.as_slice(),
    [EventStreamEvent::Serialization(event)]
      if event.field_path_hash() == 22u128
        && matches!(event.kind(), SerializationEventKind::Failure(_))
  ));
}

#[test]
fn telemetry_service_applies_latency_threshold() {
  let fixture = TelemetryFixture::new(TelemetryConfig::default());

  SerializationTelemetry::record_latency(&fixture.telemetry, 33u128, Duration::from_micros(100));
  assert!(fixture.events().is_empty());

  SerializationTelemetry::record_latency(&fixture.telemetry, 33u128, Duration::from_micros(400));

  let events = fixture.events();
  assert!(matches!(
    events.as_slice(),
    [EventStreamEvent::Serialization(event)]
      if matches!(event.kind(), SerializationEventKind::Latency(400))
  ));
}

#[test]
fn telemetry_service_emits_fallback_event_and_deadletter() {
  let fixture = TelemetryFixture::new(TelemetryConfig::default());

  SerializationTelemetry::record_fallback(&fixture.telemetry, 77u128, SerializationFallbackReason::ExternalNotAllowed);

  let events = fixture.events();
  assert!(events.iter().any(|stream_event| matches!(
    stream_event,
    EventStreamEvent::Serialization(event)
      if event.field_path_hash() == 77u128
        && matches!(event.kind(), SerializationEventKind::Fallback(reason)
          if matches!(reason, SerializationFallbackReason::ExternalNotAllowed))
  )));

  let dead_letters = fixture.state.dead_letters();
  assert!(dead_letters.iter().any(|entry| entry.reason() == DeadLetterReason::ExplicitRouting));
}

#[test]
fn telemetry_counters_track_all_paths() {
  let fixture = TelemetryFixture::new(TelemetryConfig::default());

  SerializationTelemetry::record_success(&fixture.telemetry, 1u128);
  SerializationTelemetry::record_failure(
    &fixture.telemetry,
    2u128,
    &SerializationError::SerializationFailed("boom".into()),
  );
  SerializationTelemetry::record_external_success(&fixture.telemetry, 3u128);
  SerializationTelemetry::record_external_failure(
    &fixture.telemetry,
    4u128,
    &SerializationError::SerializationFailed("oops".into()),
  );

  let counters = fixture.telemetry.counters();
  assert_eq!(counters.success_total(), 1);
  assert_eq!(counters.failure_total(), 1);
  assert_eq!(counters.external_success_total(), 1);
  assert_eq!(counters.external_failure_total(), 1);
}

#[test]
fn telemetry_service_emits_debug_trace_when_enabled() {
  let fixture = TelemetryFixture::new(TelemetryConfig::default());
  SerializationTelemetry::record_debug_trace(&fixture.telemetry, 90u128, "demo", 8);
  assert!(fixture.events().is_empty());

  fixture.telemetry.config().set_debug_trace_enabled(true);
  SerializationTelemetry::record_debug_trace(&fixture.telemetry, 90u128, "demo", 8);

  let events = fixture.events();
  assert!(matches!(
    events.as_slice(),
    [EventStreamEvent::Serialization(event)]
      if matches!(event.kind(), SerializationEventKind::DebugTrace(info)
        if info.manifest() == "demo" && info.size_bytes() == 8)
  ));
}

#[test]
fn runtime_monitor_reports_binding_error() {
  let fixture = TelemetryFixture::new(TelemetryConfig::default());
  let monitor = SerializationRuntimeMonitor::new(fixture.state.clone());

  SerializationTelemetry::record_failure(
    &fixture.telemetry,
    123u128,
    &SerializationError::NoSerializerForType("Missing"),
  );

  assert!(monitor.last_binding_error().is_some());
}

#[derive(Default)]
struct RecordingSubscriber {
  events: Mutex<Vec<EventStreamEvent<NoStdToolbox>>>,
}

impl RecordingSubscriber {
  fn events(&self) -> Vec<EventStreamEvent<NoStdToolbox>> {
    self.events.lock().clone()
  }
}

impl EventStreamSubscriber<NoStdToolbox> for RecordingSubscriber {
  fn on_event(&self, event: &EventStreamEvent<NoStdToolbox>) {
    self.events.lock().push(event.clone());
  }
}

struct TelemetryFixture {
  state:         ArcShared<SystemStateGeneric<NoStdToolbox>>,
  telemetry:     TelemetryService<NoStdToolbox>,
  subscriber:    ArcShared<RecordingSubscriber>,
  _subscription: EventStreamSubscription,
}

impl TelemetryFixture {
  fn new(config: TelemetryConfig) -> Self {
    let state = ArcShared::new(SystemStateGeneric::new());
    let telemetry = TelemetryService::new(state.clone(), config);
    let stream = state.event_stream();
    let subscriber = ArcShared::new(RecordingSubscriber::default());
    let subscriber_dyn: ArcShared<dyn EventStreamSubscriber<NoStdToolbox>> = subscriber.clone();
    let subscription = EventStream::subscribe_arc(&stream, &subscriber_dyn);

    Self { state, telemetry, subscriber, _subscription: subscription }
  }

  fn events(&self) -> Vec<EventStreamEvent<NoStdToolbox>> {
    self.subscriber.events()
  }
}
