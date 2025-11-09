#![cfg(not(target_os = "none"))]

extern crate alloc;

use alloc::vec::Vec;
use core::hint::spin_loop;

use bincode::config::{Config, standard};
use cellactor_actor_core_rs::{
  NoStdToolbox,
  actor_prim::{Actor, ActorContextGeneric},
  event_stream::{EventStreamEvent, EventStreamSubscriber, SerializationEventKind},
  messaging::AnyMessageView,
  props::Props,
  serialization::{
    AggregateSchemaBuilder, FieldPath, FieldPathDisplay, FieldPathSegment, SERIALIZATION_EXTENSION, Serialization,
    SerializationError, SerializerHandle, SerializerImpl, SerializerRegistry, TraversalPolicy,
  },
  system::ActorSystem,
};
use cellactor_utils_core_rs::sync::{ArcShared, NoStdMutex};
use erased_serde::Serialize as ErasedSerialize;
use serde::{Deserialize, Serialize};

#[test]
fn serialization_success_emits_events_and_metrics() {
  let mut fixture = SystemFixture::new();
  let serialization = fixture.serialization();
  let registry = serialization.registry();
  let serializer = register_test_serializer(&serialization);
  register_allowed_bindings(&registry, &serializer);
  register_allowed_schema(&registry, false);

  serialization.serialize(&AllowedAggregate { allowed: AllowedLeaf(7) }).expect("serialize");

  wait_until(|| fixture.events().iter().any(is_success_event));
  wait_until(|| serialization.telemetry().counters().success_total() == 1);

  let monitor = fixture.system.serialization_runtime_monitor();
  assert!(monitor.last_binding_error().is_none());

  fixture.shutdown();
}

#[test]
fn external_fallback_produces_event_and_deadletter() {
  let mut fixture = SystemFixture::new();
  let serialization = fixture.serialization();
  let registry = serialization.registry();
  let serializer = register_test_serializer(&serialization);
  register_allowed_schema(&registry, true);
  // Register only the aggregate root to force fallback for the field.
  bind_type::<AllowedAggregate>(&registry, &serializer, "allowed.aggregate", decode_allowed_aggregate);

  serialization.serialize(&AllowedAggregate { allowed: AllowedLeaf(9) }).expect("serialize");

  wait_until(|| fixture.events().iter().any(is_fallback_event));
  wait_until(|| !fixture.system.dead_letters().is_empty());
  assert_eq!(serialization.telemetry().counters().external_success_total(), 1);

  fixture.shutdown();
}

#[test]
fn missing_binding_records_binding_error() {
  let mut fixture = SystemFixture::new();
  let serialization = fixture.serialization();
  let registry = serialization.registry();
  let serializer = register_test_serializer(&serialization);
  register_strict_schema(&registry);
  bind_type::<StrictAggregate>(&registry, &serializer, "strict.aggregate", decode_strict_aggregate);

  let err = serialization.serialize(&StrictAggregate { strict: StrictLeaf(3) }).expect_err("should fail");
  assert!(matches!(err, SerializationError::SerializationFailed(message) if message.contains("strict.strict")));

  wait_until(|| fixture.events().iter().any(is_failure_event));
  let monitor = fixture.system.serialization_runtime_monitor();
  wait_until(|| monitor.last_binding_error().is_some());

  fixture.shutdown();
}

struct SystemFixture {
  system:        ActorSystem,
  subscriber:    ArcShared<EventProbe>,
  _subscription: cellactor_actor_core_rs::event_stream::EventStreamSubscription,
}

impl SystemFixture {
  fn new() -> Self {
    let props = Props::from_fn(|| TestGuardian);
    let system = ActorSystem::new(&props).expect("actor system");
    let subscriber = ArcShared::new(EventProbe::default());
    let subscriber_dyn: ArcShared<dyn EventStreamSubscriber<NoStdToolbox>> = subscriber.clone();
    let subscription = system.subscribe_event_stream(&subscriber_dyn);
    Self { system, subscriber, _subscription: subscription }
  }

  fn serialization(&self) -> ArcShared<cellactor_actor_core_rs::serialization::Serialization<NoStdToolbox>> {
    self.system.extension(&SERIALIZATION_EXTENSION).expect("serialization extension")
  }

  fn events(&self) -> Vec<EventStreamEvent<NoStdToolbox>> {
    self.subscriber.events()
  }

  fn shutdown(&mut self) {
    self.system.terminate().expect("terminate");
    self.system.run_until_terminated();
  }
}

struct EventProbe {
  events: NoStdMutex<Vec<EventStreamEvent<NoStdToolbox>>>,
}

impl Default for EventProbe {
  fn default() -> Self {
    Self { events: NoStdMutex::new(Vec::new()) }
  }
}

impl EventProbe {
  fn events(&self) -> Vec<EventStreamEvent<NoStdToolbox>> {
    self.events.lock().clone()
  }
}

impl EventStreamSubscriber<NoStdToolbox> for EventProbe {
  fn on_event(&self, event: &EventStreamEvent<NoStdToolbox>) {
    self.events.lock().push(event.clone());
  }
}

struct TestGuardian;

impl Actor for TestGuardian {
  fn receive(
    &mut self,
    _ctx: &mut ActorContextGeneric<'_, NoStdToolbox>,
    _message: AnyMessageView<'_, NoStdToolbox>,
  ) -> Result<(), cellactor_actor_core_rs::error::ActorError> {
    Ok(())
  }
}

#[derive(Clone)]
struct TestSerializer;

impl SerializerImpl for TestSerializer {
  fn identifier(&self) -> u32 {
    77
  }

  fn serialize_erased(
    &self,
    value: &dyn ErasedSerialize,
  ) -> Result<cellactor_actor_core_rs::serialization::Bytes, SerializationError> {
    bincode::serde::encode_to_vec(value, bincode_config())
      .map(cellactor_actor_core_rs::serialization::Bytes::from_vec)
      .map_err(|error| SerializationError::SerializationFailed(error.to_string()))
  }

  fn deserialize(
    &self,
    _bytes: &[u8],
    manifest: &str,
  ) -> Result<alloc::boxed::Box<dyn core::any::Any + Send>, SerializationError> {
    Err(SerializationError::UnknownManifest { serializer_id: self.identifier(), manifest: manifest.to_string() })
  }
}

fn register_test_serializer(serialization: &ArcShared<Serialization<NoStdToolbox>>) -> SerializerHandle {
  let handle = SerializerHandle::new(TestSerializer);
  let registry = serialization.registry();
  registry.register_serializer(handle.clone()).expect("register serializer");
  handle
}

fn register_allowed_bindings(registry: &ArcShared<SerializerRegistry<NoStdToolbox>>, handle: &SerializerHandle) {
  bind_type::<AllowedLeaf>(registry, handle, "allowed.leaf", decode_allowed_leaf);
  bind_type::<AllowedAggregate>(registry, handle, "allowed.aggregate", decode_allowed_aggregate);
}

fn register_allowed_schema(registry: &ArcShared<SerializerRegistry<NoStdToolbox>>, external_allowed: bool) {
  let mut builder = AggregateSchemaBuilder::<AllowedAggregate>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("allowed").expect("display"),
  );
  builder
    .add_value_field::<AllowedLeaf, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("allowed.field").expect("display"),
      external_allowed,
      |aggregate| &aggregate.allowed,
    )
    .expect("field");
  registry.register_aggregate_schema(builder.finish().expect("schema")).expect("register schema");
}

fn register_strict_schema(registry: &ArcShared<SerializerRegistry<NoStdToolbox>>) {
  let mut builder = AggregateSchemaBuilder::<StrictAggregate>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("strict").expect("display"),
  );
  builder
    .add_value_field::<StrictLeaf, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("strict.strict").expect("display"),
      false,
      |aggregate| &aggregate.strict,
    )
    .expect("field");
  registry.register_aggregate_schema(builder.finish().expect("schema")).expect("register schema");
}

fn bind_type<T>(
  registry: &ArcShared<SerializerRegistry<NoStdToolbox>>,
  handle: &SerializerHandle,
  manifest: &str,
  decoder: fn(&[u8]) -> Result<T, SerializationError>,
) where
  T: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static, {
  registry.bind_type::<T, _>(handle, Some(manifest.to_string()), decoder).expect("bind type");
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct AllowedLeaf(u32);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct AllowedAggregate {
  allowed: AllowedLeaf,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct StrictLeaf(u32);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct StrictAggregate {
  strict: StrictLeaf,
}

fn decode_allowed_leaf(bytes: &[u8]) -> Result<AllowedLeaf, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode_config())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
}

fn decode_allowed_aggregate(bytes: &[u8]) -> Result<AllowedAggregate, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode_config())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
}

fn decode_strict_aggregate(bytes: &[u8]) -> Result<StrictAggregate, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode_config())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
}

fn bincode_config() -> impl Config {
  standard().with_fixed_int_encoding()
}

fn wait_until(condition: impl Fn() -> bool) {
  for _ in 0..10_000 {
    if condition() {
      return;
    }
    spin_loop();
  }
  assert!(condition());
}

fn is_success_event(event: &EventStreamEvent<NoStdToolbox>) -> bool {
  matches!(event, EventStreamEvent::Serialization(runtime) if matches!(runtime.kind(), SerializationEventKind::Success))
}

fn is_fallback_event(event: &EventStreamEvent<NoStdToolbox>) -> bool {
  matches!(event, EventStreamEvent::Serialization(runtime)
    if matches!(runtime.kind(), SerializationEventKind::Fallback(_)))
}

fn is_failure_event(event: &EventStreamEvent<NoStdToolbox>) -> bool {
  matches!(event, EventStreamEvent::Serialization(runtime)
    if matches!(runtime.kind(), SerializationEventKind::Failure(_)))
}
