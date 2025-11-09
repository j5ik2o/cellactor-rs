use alloc::{string::{String, ToString}, vec::Vec};

use cellactor_utils_core_rs::sync::ArcShared;

use super::NestedSerializerOrchestrator;
use crate::{
  NoStdToolbox,
  serialization::{
    AggregateSchemaBuilder, BincodeSerializer, FieldPath, FieldPathDisplay, FieldPathSegment, SerializerHandle,
    SerializerRegistry, TraversalPolicy, SerializationError,
  },
};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct Leaf(u32);

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct EnvelopeAggregate {
  first: Leaf,
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

#[test]
fn orchestrator_serializes_aggregate_as_field_envelope() {
  let registry = ArcShared::new(SerializerRegistry::<NoStdToolbox>::new());
  let handle = SerializerHandle::new(BincodeSerializer::new());
  registry.register_serializer(handle.clone()).expect("register handle");
  registry.bind_type::<Leaf, _>(&handle, Some("leaf".into()), decode_leaf).expect("bind leaf");
  registry
    .bind_type::<EnvelopeAggregate, _>(&handle, Some("envelope".into()), decode_envelope)
    .expect("bind env");

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

  let orchestrator = NestedSerializerOrchestrator::new(registry.clone());
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
