use alloc::{
  string::{String, ToString},
  vec,
  vec::Vec,
};
use core::time::Duration;

use cellactor_utils_core_rs::sync::ArcShared;
use spin::Mutex;

use super::NestedSerializerOrchestrator;
use crate::{
  NoStdToolbox,
  event_stream::SerializationFallbackReason,
  serialization::{
    AggregateSchemaBuilder, BincodeSerializer, FieldPath, FieldPathDisplay, FieldPathHash, FieldPathSegment,
    NoopSerializationTelemetry, SerializationError, SerializationTelemetry, SerializerHandle, SerializerRegistry,
    TraversalPolicy,
  },
};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct Leaf(u32);

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct EnvelopeAggregate {
  first:  Leaf,
  second: Leaf,
}

fn decode_leaf(bytes: &[u8]) -> Result<Leaf, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode::config::standard().with_fixed_int_encoding())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
}

fn decode_envelope(bytes: &[u8]) -> Result<EnvelopeAggregate, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode::config::standard().with_fixed_int_encoding())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
}

fn encoded_leaf_len(value: &Leaf) -> usize {
  bincode::serde::encode_to_vec(value, bincode::config::standard().with_fixed_int_encoding())
    .expect("encode")
    .len()
}

#[test]
fn orchestrator_serializes_aggregate_as_field_envelope() {
  let registry = ArcShared::new(SerializerRegistry::<NoStdToolbox>::new());
  let handle = SerializerHandle::new(BincodeSerializer::new());
  registry.register_serializer(handle.clone()).expect("register handle");
  registry.bind_type::<Leaf, _>(&handle, Some("leaf".into()), decode_leaf).expect("bind leaf");
  registry.bind_type::<EnvelopeAggregate, _>(&handle, Some("envelope".into()), decode_envelope).expect("bind env");

  register_envelope_schema(&registry);

  let orchestrator =
    NestedSerializerOrchestrator::new(registry.clone(), ArcShared::new(NoopSerializationTelemetry::new()));
  let aggregate = EnvelopeAggregate { first: Leaf(7), second: Leaf(9) };
  let payload = orchestrator.serialize(&aggregate).expect("serialize");

  assert_eq!(payload.serializer_id(), handle.identifier());
  assert_eq!(payload.manifest(), "envelope");

  let mut cursor = payload.bytes().as_ref();
  assert_eq!(read_bytes(&mut cursor, 4).as_slice(), b"AGGR");
  let count = read_u16(&mut cursor);
  assert_eq!(count, 2);

  let first = read_entry(&mut cursor);
  assert_eq!(first.serializer_id, handle.identifier());
  assert_eq!(first.manifest, "leaf");
  assert_eq!(decode_leaf(first.bytes.as_slice()).expect("decode"), Leaf(7));

  let second = read_entry(&mut cursor);
  assert_eq!(second.serializer_id, handle.identifier());
  assert_eq!(second.manifest, "leaf");
  assert_eq!(decode_leaf(second.bytes.as_slice()).expect("decode"), Leaf(9));
}

#[test]
fn orchestrator_notifies_telemetry_hooks() {
  let registry = ArcShared::new(SerializerRegistry::<NoStdToolbox>::new());
  let handle = SerializerHandle::new(BincodeSerializer::new());
  registry.register_serializer(handle.clone()).expect("register handle");
  registry.bind_type::<Leaf, _>(&handle, Some("leaf".into()), decode_leaf).expect("bind leaf");
  registry.bind_type::<EnvelopeAggregate, _>(&handle, Some("envelope".into()), decode_envelope).expect("bind env");
  register_envelope_schema(&registry);
  let schema = registry.load_schema::<EnvelopeAggregate>().expect("schema");
  let hashes = schema.fields().iter().map(|node| node.path_hash()).collect::<Vec<_>>();

  let telemetry = ArcShared::new(TelemetryProbe::new());
  let orchestrator = NestedSerializerOrchestrator::new(registry.clone(), telemetry.clone());
  let aggregate = EnvelopeAggregate { first: Leaf(1), second: Leaf(2) };
  let _ = orchestrator.serialize(&aggregate).expect("serialize");
  let first_size = encoded_leaf_len(&aggregate.first);
  let second_size = encoded_leaf_len(&aggregate.second);

  let events = telemetry.snapshot();
  assert_eq!(events, vec![
    TelemetryCall::AggregateStart,
    TelemetryCall::FieldSuccess(hashes[0]),
    TelemetryCall::FieldDebugTrace(hashes[0], String::from("leaf"), first_size),
    TelemetryCall::FieldLatency(hashes[0], Duration::ZERO),
    TelemetryCall::FieldSuccess(hashes[1]),
    TelemetryCall::FieldDebugTrace(hashes[1], String::from("leaf"), second_size),
    TelemetryCall::FieldLatency(hashes[1], Duration::ZERO),
    TelemetryCall::AggregateFinish,
  ]);
}

#[test]
fn orchestrator_notifies_failure_telemetry() {
  let registry = ArcShared::new(SerializerRegistry::<NoStdToolbox>::new());
  let handle = SerializerHandle::new(BincodeSerializer::new());
  registry.register_serializer(handle.clone()).expect("register handle");
  registry.bind_type::<EnvelopeAggregate, _>(&handle, Some("envelope".into()), decode_envelope).expect("bind env");
  register_envelope_schema(&registry);
  let schema = registry.load_schema::<EnvelopeAggregate>().expect("schema");
  let hash = schema.fields()[0].path_hash();

  let telemetry = ArcShared::new(TelemetryProbe::new());
  let orchestrator = NestedSerializerOrchestrator::new(registry.clone(), telemetry.clone());
  let aggregate = EnvelopeAggregate { first: Leaf(1), second: Leaf(2) };
  let err = orchestrator.serialize(&aggregate).expect_err("should fail");
  assert!(matches!(
    err,
    SerializationError::SerializationFailed(message)
      if message.contains("external serializer not allowed for field envelope.first")
  ));

  let events = telemetry.snapshot();
  assert_eq!(events, vec![
    TelemetryCall::AggregateStart,
    TelemetryCall::Fallback(hash, SerializationFallbackReason::MissingSerializer),
    TelemetryCall::Fallback(hash, SerializationFallbackReason::ExternalNotAllowed),
    TelemetryCall::FieldFailure(hash),
    TelemetryCall::FieldLatency(hash, Duration::ZERO),
    TelemetryCall::AggregateFinish,
  ]);
}

#[test]
fn orchestrator_serializes_allowed_field_with_external_adapter() {
  let registry = ArcShared::new(SerializerRegistry::<NoStdToolbox>::new());
  let handle = SerializerHandle::new(BincodeSerializer::new());
  registry.register_serializer(handle.clone()).expect("register handle");
  registry.bind_type::<EnvelopeAggregate, _>(&handle, Some("envelope".into()), decode_envelope).expect("bind env");
  register_external_schema(&registry);

  let orchestrator =
    NestedSerializerOrchestrator::new(registry.clone(), ArcShared::new(NoopSerializationTelemetry::new()));
  let aggregate = EnvelopeAggregate { first: Leaf(11), second: Leaf(0) };
  let payload = orchestrator.serialize(&aggregate).expect("serialize");

  let mut cursor = payload.bytes().as_ref();
  assert_eq!(read_bytes(&mut cursor, 4).as_slice(), b"AGGR");
  assert_eq!(read_u16(&mut cursor), 1);
  let entry = read_entry(&mut cursor);
  assert_eq!(entry.serializer_id, handle.identifier());
  assert_eq!(entry.manifest, core::any::type_name::<Leaf>());
  assert_eq!(decode_leaf(entry.bytes.as_slice()).expect("decode"), Leaf(11));
  assert!(cursor.is_empty());
}

#[test]
fn orchestrator_reports_external_telemetry_events() {
  let registry = ArcShared::new(SerializerRegistry::<NoStdToolbox>::new());
  let handle = SerializerHandle::new(BincodeSerializer::new());
  registry.register_serializer(handle.clone()).expect("register handle");
  registry.bind_type::<EnvelopeAggregate, _>(&handle, Some("external".into()), decode_envelope).expect("bind env");
  register_external_schema(&registry);
  let schema = registry.load_schema::<EnvelopeAggregate>().expect("schema");
  let hash = schema.fields()[0].path_hash();

  let telemetry = ArcShared::new(TelemetryProbe::new());
  let orchestrator = NestedSerializerOrchestrator::new(registry.clone(), telemetry.clone());
  let aggregate = EnvelopeAggregate { first: Leaf(11), second: Leaf(0) };
  let _ = orchestrator.serialize(&aggregate).expect("serialize");

  let leaf_size = encoded_leaf_len(&aggregate.first);
  let events = telemetry.snapshot();
  assert_eq!(events, vec![
    TelemetryCall::AggregateStart,
    TelemetryCall::Fallback(hash, SerializationFallbackReason::MissingSerializer),
    TelemetryCall::ExternalSuccess(hash),
    TelemetryCall::FieldSuccess(hash),
    TelemetryCall::FieldDebugTrace(hash, core::any::type_name::<Leaf>().to_string(), leaf_size),
    TelemetryCall::FieldLatency(hash, Duration::ZERO),
    TelemetryCall::AggregateFinish,
  ]);
}

fn register_envelope_schema(registry: &ArcShared<SerializerRegistry<NoStdToolbox>>) {
  let mut builder = AggregateSchemaBuilder::<EnvelopeAggregate>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("envelope").expect("display"),
  );
  builder
    .add_value_field::<Leaf, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("envelope.first").expect("display"),
      false,
      |aggregate| &aggregate.first,
    )
    .expect("first");
  builder
    .add_value_field::<Leaf, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(1)]).expect("path"),
      FieldPathDisplay::from_str("envelope.second").expect("display"),
      false,
      |aggregate| &aggregate.second,
    )
    .expect("second");
  registry.register_aggregate_schema(builder.finish().expect("schema")).expect("register schema");
}

fn register_external_schema(registry: &ArcShared<SerializerRegistry<NoStdToolbox>>) {
  let mut builder = AggregateSchemaBuilder::<EnvelopeAggregate>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("external").expect("display"),
  );
  builder
    .add_value_field::<Leaf, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("external.first").expect("display"),
      true,
      |aggregate| &aggregate.first,
    )
    .expect("allow_external");
  registry.register_aggregate_schema(builder.finish().expect("schema")).expect("register schema");
}

fn read_entry(buffer: &mut &[u8]) -> ParsedEntry {
  let _field_hash = read_u128(buffer);
  let serializer_id = read_u32(buffer);
  let manifest_len = read_u16(buffer) as usize;
  let manifest = read_bytes(buffer, manifest_len);
  let payload_len = read_u32(buffer) as usize;
  let bytes = read_bytes(buffer, payload_len);
  ParsedEntry { _field_hash, serializer_id, manifest: String::from_utf8(manifest).expect("utf8"), bytes }
}

fn read_bytes(buffer: &mut &[u8], len: usize) -> Vec<u8> {
  let (head, tail) = buffer.split_at(len);
  *buffer = tail;
  head.to_vec()
}

fn read_u16(buffer: &mut &[u8]) -> u16 {
  let (value, tail) = buffer.split_at(core::mem::size_of::<u16>());
  *buffer = tail;
  u16::from_le_bytes(value.try_into().expect("u16 slice"))
}

fn read_u32(buffer: &mut &[u8]) -> u32 {
  let (value, tail) = buffer.split_at(core::mem::size_of::<u32>());
  *buffer = tail;
  u32::from_le_bytes(value.try_into().expect("u32 slice"))
}

fn read_u128(buffer: &mut &[u8]) -> u128 {
  let (value, tail) = buffer.split_at(core::mem::size_of::<u128>());
  *buffer = tail;
  u128::from_le_bytes(value.try_into().expect("u128 slice"))
}

struct ParsedEntry {
  _field_hash:   u128,
  serializer_id: u32,
  manifest:      String,
  bytes:         Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TelemetryCall {
  AggregateStart,
  AggregateFinish,
  FieldSuccess(FieldPathHash),
  FieldFailure(FieldPathHash),
  FieldLatency(FieldPathHash, Duration),
  FieldDebugTrace(FieldPathHash, String, usize),
  Fallback(FieldPathHash, SerializationFallbackReason),
  ExternalSuccess(FieldPathHash),
  ExternalFailure(FieldPathHash),
}

#[derive(Default)]
struct TelemetryProbe {
  calls: Mutex<Vec<TelemetryCall>>,
}

impl TelemetryProbe {
  fn new() -> Self {
    Self { calls: Mutex::new(Vec::new()) }
  }

  fn snapshot(&self) -> Vec<TelemetryCall> {
    self.calls.lock().clone()
  }

  fn push(&self, call: TelemetryCall) {
    self.calls.lock().push(call);
  }
}

impl SerializationTelemetry for TelemetryProbe {
  fn on_aggregate_start(&self) {
    self.push(TelemetryCall::AggregateStart);
  }

  fn on_aggregate_finish(&self) {
    self.push(TelemetryCall::AggregateFinish);
  }

  fn record_latency(&self, field_path_hash: FieldPathHash, elapsed: Duration) {
    self.push(TelemetryCall::FieldLatency(field_path_hash, elapsed));
  }

  fn record_success(&self, field_path_hash: FieldPathHash) {
    self.push(TelemetryCall::FieldSuccess(field_path_hash));
  }

  fn record_failure(&self, field_path_hash: FieldPathHash, _error: &SerializationError) {
    self.push(TelemetryCall::FieldFailure(field_path_hash));
  }

  fn record_fallback(&self, field_path_hash: FieldPathHash, reason: SerializationFallbackReason) {
    self.push(TelemetryCall::Fallback(field_path_hash, reason));
  }

  fn record_external_success(&self, field_path_hash: FieldPathHash) {
    self.push(TelemetryCall::ExternalSuccess(field_path_hash));
  }

  fn record_external_failure(&self, field_path_hash: FieldPathHash, _error: &SerializationError) {
    self.push(TelemetryCall::ExternalFailure(field_path_hash));
  }

  fn record_debug_trace(&self, field_path_hash: FieldPathHash, manifest: &str, size_bytes: usize) {
    self.push(TelemetryCall::FieldDebugTrace(field_path_hash, manifest.to_string(), size_bytes));
  }
}
