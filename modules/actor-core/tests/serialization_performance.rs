#![cfg(not(target_os = "none"))]

extern crate alloc;

use alloc::vec::Vec;

use bincode::config::{Config, standard};
use cellactor_actor_core_rs::{
  NoStdToolbox,
  actor_prim::{Actor, ActorContextGeneric},
  error::ActorError,
  event_stream::{EventStreamEvent, EventStreamSubscriber, EventStreamSubscription, SerializationEventKind},
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

const PERF_SERIALIZER_ID: u32 = 9001;
const PERF_LATENCY_THRESHOLD_US: u64 = 250;
const HIGH_VOLUME_MESSAGES: usize = 10_000;

#[test]
fn deep_schema_serialization_stays_within_latency_threshold() {
  let fixture = PerformanceFixture::new();
  let serialization = fixture.serialization();
  let registry = serialization.registry();
  let serializer = register_perf_serializer(&serialization);
  bind_type::<PerfLeaf>(&registry, &serializer, "perf.leaf", decode_perf_leaf);
  bind_type::<DeepAggregate>(&registry, &serializer, "perf.deep", decode_deep_aggregate);
  install_deep_schema(&registry, 32);

  serialization.telemetry().config().set_latency_threshold_us(PERF_LATENCY_THRESHOLD_US);

  let aggregate = DeepAggregate { leaf: PerfLeaf(42) };
  serialization.serialize(&aggregate).expect("serialize");

  let events = fixture.events();
  let latency_events = events
    .iter()
    .filter(|event| {
      matches!(event, EventStreamEvent::Serialization(runtime)
      if matches!(runtime.kind(), SerializationEventKind::Latency(_)))
    })
    .count();
  assert_eq!(latency_events, 0, "unexpected latency events detected");
}

#[test]
fn serialize_10000_messages_without_metric_overflow() {
  let fixture = PerformanceFixture::new();
  let serialization = fixture.serialization();
  let registry = serialization.registry();
  let serializer = register_perf_serializer(&serialization);
  bind_type::<PerfLeaf>(&registry, &serializer, "perf.leaf", decode_perf_leaf);
  bind_type::<ThroughputAggregate>(&registry, &serializer, "perf.throughput", decode_throughput_aggregate);
  install_throughput_schema(&registry);

  for index in 0..HIGH_VOLUME_MESSAGES {
    let aggregate = ThroughputAggregate { value: PerfLeaf(index as u32) };
    serialization.serialize(&aggregate).expect("serialize");
  }

  let telemetry = serialization.telemetry();
  let counters = telemetry.counters();
  assert_eq!(counters.success_total(), HIGH_VOLUME_MESSAGES as u64);
  assert_eq!(counters.failure_total(), 0);
  assert_eq!(counters.external_success_total(), 0);
  assert_eq!(counters.external_failure_total(), 0);

  let events = fixture.events();
  assert!(events.iter().all(|event| !matches!(
    event,
    EventStreamEvent::Serialization(runtime) if matches!(runtime.kind(), SerializationEventKind::Fallback(_))
  )));
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct PerfLeaf(u32);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct DeepAggregate {
  leaf: PerfLeaf,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ThroughputAggregate {
  value: PerfLeaf,
}

struct PerformanceFixture {
  system:        ActorSystem,
  serialization: ArcShared<Serialization<NoStdToolbox>>,
  subscriber:    ArcShared<EventCollector>,
  _subscription: EventStreamSubscription,
}

impl PerformanceFixture {
  fn new() -> Self {
    let props = Props::from_fn(|| PerfGuardian);
    let system = ActorSystem::new(&props).expect("actor system");
    let serialization = system.extension(&SERIALIZATION_EXTENSION).expect("serialization extension");
    let subscriber = ArcShared::new(EventCollector::default());
    let subscriber_dyn: ArcShared<dyn EventStreamSubscriber<NoStdToolbox>> = subscriber.clone();
    let subscription = system.subscribe_event_stream(&subscriber_dyn);
    Self { system, serialization, subscriber, _subscription: subscription }
  }

  fn serialization(&self) -> ArcShared<Serialization<NoStdToolbox>> {
    self.serialization.clone()
  }

  fn events(&self) -> Vec<EventStreamEvent<NoStdToolbox>> {
    self.subscriber.events()
  }
}

impl Drop for PerformanceFixture {
  fn drop(&mut self) {
    let _ = self.system.terminate();
    self.system.run_until_terminated();
  }
}

struct PerfGuardian;

impl Actor for PerfGuardian {
  fn receive(
    &mut self,
    _ctx: &mut ActorContextGeneric<'_, NoStdToolbox>,
    _message: AnyMessageView<'_, NoStdToolbox>,
  ) -> Result<(), ActorError> {
    Ok(())
  }
}

struct EventCollector {
  events: NoStdMutex<Vec<EventStreamEvent<NoStdToolbox>>>,
}

impl EventCollector {
  fn new() -> Self {
    Self { events: NoStdMutex::new(Vec::new()) }
  }

  fn events(&self) -> Vec<EventStreamEvent<NoStdToolbox>> {
    self.events.lock().clone()
  }
}

impl Default for EventCollector {
  fn default() -> Self {
    Self::new()
  }
}

impl EventStreamSubscriber<NoStdToolbox> for EventCollector {
  fn on_event(&self, event: &EventStreamEvent<NoStdToolbox>) {
    self.events.lock().push(event.clone());
  }
}

fn register_perf_serializer(serialization: &ArcShared<Serialization<NoStdToolbox>>) -> SerializerHandle {
  let handle = SerializerHandle::new(PerfSerializer);
  serialization.registry().register_serializer(handle.clone()).expect("register serializer");
  handle
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

fn install_deep_schema(registry: &ArcShared<SerializerRegistry<NoStdToolbox>>, depth: usize) {
  let mut builder = AggregateSchemaBuilder::<DeepAggregate>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("deep").expect("display"),
  );
  let segments = build_segments(depth);
  builder
    .add_value_field::<PerfLeaf, _>(
      FieldPath::from_segments(&segments).expect("path"),
      FieldPathDisplay::from_str("deep.leaf").expect("display"),
      false,
      |aggregate| &aggregate.leaf,
    )
    .expect("field");
  registry.register_aggregate_schema(builder.finish().expect("schema")).expect("register schema");
}

fn install_throughput_schema(registry: &ArcShared<SerializerRegistry<NoStdToolbox>>) {
  let mut builder = AggregateSchemaBuilder::<ThroughputAggregate>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("throughput").expect("display"),
  );
  builder
    .add_value_field::<PerfLeaf, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("throughput.value").expect("display"),
      false,
      |aggregate| &aggregate.value,
    )
    .expect("field");
  registry.register_aggregate_schema(builder.finish().expect("schema")).expect("register schema");
}

fn build_segments(depth: usize) -> Vec<FieldPathSegment> {
  (0..depth).map(|index| FieldPathSegment::new(index as u16)).collect()
}

#[derive(Clone)]
struct PerfSerializer;

impl SerializerImpl for PerfSerializer {
  fn identifier(&self) -> u32 {
    PERF_SERIALIZER_ID
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

fn decode_perf_leaf(bytes: &[u8]) -> Result<PerfLeaf, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode_config())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
}

fn decode_deep_aggregate(bytes: &[u8]) -> Result<DeepAggregate, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode_config())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
}

fn decode_throughput_aggregate(bytes: &[u8]) -> Result<ThroughputAggregate, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode_config())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
}

fn bincode_config() -> impl Config {
  standard().with_fixed_int_encoding()
}
