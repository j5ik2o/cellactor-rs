use alloc::{
  string::{String, ToString},
  vec::Vec,
};

use cellactor_utils_core_rs::sync::ArcShared;

use super::SERIALIZATION_EXTENSION;
use crate::{
  NoStdToolbox,
  serialization::{
    AggregateSchemaBuilder, FieldPath, FieldPathDisplay, FieldPathSegment, PekkoSerializable, TraversalPolicy,
    error::SerializationError, registry::SerializerRegistry,
  },
  system::ActorSystemGeneric,
};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct TestMessage(String);

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct AutoPekkoMessage(u32);

impl PekkoSerializable for AutoPekkoMessage {}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct Leaf(u32);

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct EnvelopeAggregate {
  first:  Leaf,
  second: Leaf,
}

fn decode(bytes: &[u8]) -> Result<TestMessage, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode::config::standard().with_fixed_int_encoding())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
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

#[test]
fn serialization_extension_roundtrip() {
  let system = ActorSystemGeneric::<NoStdToolbox>::new_empty();
  let serialization = system.register_extension(&SERIALIZATION_EXTENSION);
  let registry: ArcShared<SerializerRegistry<NoStdToolbox>> = serialization.registry();
  let serializer = registry.find_serializer_by_id(1).expect("built-in serializer");
  registry.bind_type::<TestMessage, _>(&serializer, Some("TestMessage".into()), decode).expect("bind");

  let message = TestMessage("hello".into());
  let payload = serialization.serialize(&message).expect("serialize");
  let roundtrip: TestMessage =
    serialization.deserialize(payload.bytes().as_ref(), payload.manifest()).expect("deserialize");
  assert_eq!(roundtrip, message);

  let boxed = serialization.deserialize_payload(&payload).expect("payload");
  assert!(boxed.downcast::<TestMessage>().is_ok());
}

#[test]
fn serialization_extension_registers_pekko_serializable_types() {
  let system = ActorSystemGeneric::<NoStdToolbox>::new_empty();
  let serialization = system.register_extension(&SERIALIZATION_EXTENSION);

  serialization.register_pekko_serializable::<AutoPekkoMessage>().expect("assign default");

  let sample = AutoPekkoMessage(7);
  let payload = serialization.serialize(&sample).expect("serialize");
  let recovered: AutoPekkoMessage =
    serialization.deserialize(payload.bytes().as_ref(), payload.manifest()).expect("deserialize");
  assert_eq!(recovered, sample);
}

#[test]
fn serialization_extension_serializes_aggregate_envelope() {
  let system = ActorSystemGeneric::<NoStdToolbox>::new_empty();
  let serialization = system.register_extension(&SERIALIZATION_EXTENSION);
  let registry: ArcShared<SerializerRegistry<NoStdToolbox>> = serialization.registry();
  let serializer = registry.find_serializer_by_id(1).expect("built-in serializer");
  registry.bind_type::<Leaf, _>(&serializer, Some("leaf".into()), decode_leaf).expect("bind leaf");
  registry
    .bind_type::<EnvelopeAggregate, _>(&serializer, Some("envelope".into()), decode_envelope)
    .expect("bind envelope");

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
    .expect("first field");
  builder
    .add_value_field::<Leaf, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(1)]).expect("path"),
      FieldPathDisplay::from_str("envelope.second").expect("display"),
      false,
      |aggregate| &aggregate.second,
    )
    .expect("second field");
  registry.register_aggregate_schema(builder.finish().expect("schema")).expect("register schema");

  let aggregate = EnvelopeAggregate { first: Leaf(5), second: Leaf(8) };
  let payload = serialization.serialize(&aggregate).expect("serialize");
  assert_eq!(payload.manifest(), "envelope");

  let mut cursor = payload.bytes().as_ref();
  assert_eq!(read_bytes(&mut cursor, 4).as_slice(), b"AGGR");
  let count = read_u16(&mut cursor);
  assert_eq!(count, 2);

  let first = read_entry(&mut cursor);
  assert_eq!(decode_leaf(first.as_slice()).expect("decode"), Leaf(5));
  let second = read_entry(&mut cursor);
  assert_eq!(decode_leaf(second.as_slice()).expect("decode"), Leaf(8));
}

fn read_entry(buffer: &mut &[u8]) -> Vec<u8> {
  let _hash = read_u128(buffer);
  let _serializer_id = read_u32(buffer);
  let manifest_len = read_u16(buffer) as usize;
  let _manifest = read_bytes(buffer, manifest_len);
  let payload_len = read_u32(buffer) as usize;
  read_bytes(buffer, payload_len)
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
