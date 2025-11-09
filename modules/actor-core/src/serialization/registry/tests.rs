use alloc::string::ToString;

use super::SerializerRegistry;
use crate::{
  NoStdToolbox,
  serialization::{
    AggregateSchemaBuilder, BincodeSerializer, EnvelopeMode, FieldOptions, FieldPath, FieldPathDisplay,
    FieldPathSegment, TraversalPolicy, error::SerializationError, serializer::SerializerHandle,
  },
};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct Message(u32);

fn decode(bytes: &[u8]) -> Result<Message, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode::config::standard().with_fixed_int_encoding())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
}

#[test]
fn registers_serializers() {
  let registry = SerializerRegistry::<NoStdToolbox>::new();
  let handle = SerializerHandle::new(BincodeSerializer::new());
  registry.register_serializer(handle.clone()).expect("first register");
  let err = registry.register_serializer(handle).expect_err("duplicate");
  assert!(matches!(err, SerializationError::DuplicateSerializerId(1)));
}

#[test]
fn binds_and_recovers_types() {
  let registry = SerializerRegistry::<NoStdToolbox>::new();
  let handle = SerializerHandle::new(BincodeSerializer::new());
  registry.register_serializer(handle.clone()).expect("register");
  registry.bind_type::<Message, _>(&handle, Some("Message".into()), decode).expect("bind");
  assert!(registry.has_binding_for::<Message>());
  let binding = registry.find_binding_by_manifest(handle.identifier(), "Message").expect("manifest");
  let sample = Message(9);
  let erased: &dyn erased_serde::Serialize = &sample;
  let bytes = handle.serialize_erased(erased).expect("serialize");
  let recovered: Message = binding.deserialize_as(bytes.as_ref()).expect("deserialize");
  assert_eq!(recovered, Message(9));
}

#[derive(Debug)]
struct Parent;

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
struct Child(u32);

#[test]
fn registers_aggregate_schema_and_loads_it() {
  let registry = SerializerRegistry::<NoStdToolbox>::new();
  let mut builder = AggregateSchemaBuilder::<Parent>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("parent").expect("parent"),
  );
  builder
    .add_field::<Child>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("parent.child").expect("display"),
      FieldOptions::new(EnvelopeMode::PreserveOrder),
    )
    .expect("add child");
  let schema = builder.finish().expect("schema");
  registry.register_aggregate_schema(schema).expect("register schema");
  let loaded = registry.load_schema::<Parent>().expect("load");
  assert_eq!(loaded.fields().len(), 1);
}

#[test]
fn rejects_external_serializer_for_non_pure_value() {
  let mut builder = AggregateSchemaBuilder::<Parent>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("parent").expect("parent"),
  );
  let err = builder
    .add_field::<alloc::vec::Vec<u8>>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("parent.bytes").expect("display"),
      FieldOptions::new(EnvelopeMode::PreserveOrder).with_external_serializer_allowed(true),
    )
    .expect_err("should reject non-pure value");
  assert!(matches!(err, SerializationError::InvalidAggregateSchema(_)));
}

#[test]
fn duplicate_schema_registration_fails() {
  let registry = SerializerRegistry::<NoStdToolbox>::new();
  let mut builder = AggregateSchemaBuilder::<Parent>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("parent").expect("display"),
  );
  builder
    .add_value_field::<Child>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("parent.child").expect("display"),
      false,
    )
    .expect("add field");
  let schema = builder.finish().expect("schema");
  registry.register_aggregate_schema(schema.clone()).expect("register first");
  let err = registry.register_aggregate_schema(schema).expect_err("duplicate schema");
  assert!(matches!(err, SerializationError::AggregateSchemaAlreadyRegistered(_)));
}

#[test]
fn load_schema_returns_error_for_unknown_type() {
  let registry = SerializerRegistry::<NoStdToolbox>::new();
  let err = registry.load_schema::<Parent>().expect_err("should fail");
  assert!(matches!(err, SerializationError::AggregateSchemaNotFound(_)));
}

#[test]
fn field_policy_lookup_succeeds() {
  let registry = SerializerRegistry::<NoStdToolbox>::new();
  let mut builder = AggregateSchemaBuilder::<Parent>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("parent").expect("display"),
  );
  builder
    .add_value_field::<Child>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("parent.child").expect("display"),
      true,
    )
    .expect("add field");
  let schema = builder.finish().expect("schema");
  registry.register_aggregate_schema(schema.clone()).expect("register");
  let loaded = registry.load_schema::<Parent>().expect("load");
  let hash = loaded.fields().first().expect("field").path_hash();
  assert_eq!(registry.field_policy(hash), Some(true));
}
