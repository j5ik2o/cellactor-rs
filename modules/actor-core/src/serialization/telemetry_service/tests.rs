use alloc::vec::Vec;

use cellactor_utils_core_rs::sync::ArcShared;
use spin::Mutex;

use crate::{
  NoStdToolbox,
  event_stream::{EventStream, EventStreamEvent, EventStreamSubscriber, SerializationEventKind},
  serialization::{SerializationError, SerializationTelemetry, TelemetryConfig, TelemetryService},
};

#[test]
fn telemetry_service_emits_success_event() {
  let stream = ArcShared::new(EventStream::default());
  let telemetry = TelemetryService::<NoStdToolbox>::new(stream.clone(), TelemetryConfig::default());
  let subscriber = ArcShared::new(RecordingSubscriber::default());
  let subscriber_dyn: ArcShared<dyn EventStreamSubscriber<NoStdToolbox>> = subscriber.clone();
  let _subscription = crate::event_stream::EventStream::subscribe_arc(&stream, &subscriber_dyn);

  SerializationTelemetry::record_success(&telemetry, 11u128);

  let events = subscriber.events();
  assert!(matches!(
    events.as_slice(),
    [EventStreamEvent::Serialization(event)]
      if event.field_path_hash() == 11u128
        && matches!(event.kind(), SerializationEventKind::Success)
  ));
}

#[test]
fn telemetry_service_emits_failure_event() {
  let stream = ArcShared::new(EventStream::default());
  let telemetry = TelemetryService::<NoStdToolbox>::new(stream.clone(), TelemetryConfig::default());
  let subscriber = ArcShared::new(RecordingSubscriber::default());
  let subscriber_dyn: ArcShared<dyn EventStreamSubscriber<NoStdToolbox>> = subscriber.clone();
  let _subscription = crate::event_stream::EventStream::subscribe_arc(&stream, &subscriber_dyn);

  SerializationTelemetry::record_failure(&telemetry, 22u128, &SerializationError::NoSerializerForType("Example"));

  let events = subscriber.events();
  assert!(matches!(
    events.as_slice(),
    [EventStreamEvent::Serialization(event)]
      if event.field_path_hash() == 22u128
        && matches!(event.kind(), SerializationEventKind::Failure(_))
  ));
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
