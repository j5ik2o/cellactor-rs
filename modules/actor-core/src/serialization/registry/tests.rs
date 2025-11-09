use alloc::string::ToString;
use core::any::TypeId;

use cellactor_utils_core_rs::sync::ArcShared;

use super::{super::type_binding::TypeBinding, SerializerRegistry};
use crate::{
  NoStdToolbox,
  serialization::{
    AggregateSchemaBuilder, BincodeSerializer, EnvelopeMode, FieldOptions, FieldPath, FieldPathDisplay,
    FieldPathSegment, PekkoSerializable, TraversalPolicy, error::SerializationError, serializer::SerializerHandle,
  },
};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct Message(u32);

fn decode(bytes: &[u8]) -> Result<Message, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode::config::standard().with_fixed_int_encoding())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
}

fn decode_child_value(bytes: &[u8]) -> Result<Child, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode::config::standard().with_fixed_int_encoding())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
}

fn decode_manifest_a(_bytes: &[u8]) -> Result<ManifestA, SerializationError> {
  Ok(ManifestA)
}

fn decode_manifest_b(_bytes: &[u8]) -> Result<ManifestB, SerializationError> {
  Ok(ManifestB)
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

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
struct Child(u32);

#[derive(Debug)]
struct CycleA;

#[derive(Debug)]
struct CycleB;

#[derive(Debug)]
struct ManifestA;

#[derive(Debug)]
struct ManifestB;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct AutoType(u32);

impl PekkoSerializable for AutoType {}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct CustomManifestType(u32);

impl PekkoSerializable for CustomManifestType {
  fn pekko_manifest() -> Option<&'static str> {
    Some("pekko.custom")
  }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct DuplicateManifestType(u32);

impl PekkoSerializable for DuplicateManifestType {
  fn pekko_manifest() -> Option<&'static str> {
    Some("pekko.custom")
  }
}

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
fn rejects_external_serializer_when_envelope_mode_not_supported() {
  let mut builder = AggregateSchemaBuilder::<Parent>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("parent").expect("parent"),
  );
  let err = builder
    .add_field::<Child>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("parent.child").expect("display"),
      FieldOptions::new(EnvelopeMode::Raw).with_external_serializer_allowed(true),
    )
    .expect_err("should reject raw envelope");
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

#[test]
fn audit_reports_missing_serializer() {
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
    .expect("add child");
  let schema = builder.finish().expect("schema");
  registry.register_aggregate_schema(schema).expect("register schema");
  let report = registry.audit();
  assert!(!report.success());
  assert_eq!(report.issues.len(), 1);
  assert_eq!(report.issues[0].reason, "serializer not registered");
}

#[test]
fn audit_succeeds_when_all_fields_are_bound() {
  let registry = SerializerRegistry::<NoStdToolbox>::new();
  let handle = SerializerHandle::new(BincodeSerializer::new());
  registry.register_serializer(handle.clone()).expect("register serializer");
  registry.bind_type::<Child, _>(&handle, Some("child".into()), decode_child_value).expect("bind child");

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
    .expect("add child");
  let schema = builder.finish().expect("schema");
  registry.register_aggregate_schema(schema).expect("register schema");

  let report = registry.audit();
  assert!(report.success());
  assert_eq!(report.schemas_checked, 1);
  assert!(report.issues.is_empty());
}

#[test]
fn audit_detects_cycles_in_registered_schemas() {
  let registry = SerializerRegistry::<NoStdToolbox>::new();

  let mut builder_a = AggregateSchemaBuilder::<CycleA>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("cycle_a").expect("display"),
  );
  builder_a
    .add_value_field::<CycleB>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("cycle_a.next").expect("display"),
      false,
    )
    .expect("add field");

  let mut builder_b = AggregateSchemaBuilder::<CycleB>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("cycle_b").expect("display"),
  );
  builder_b
    .add_value_field::<CycleA>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("cycle_b.next").expect("display"),
      false,
    )
    .expect("add field");

  let _ = registry.register_aggregate_schema(builder_a.finish().expect("schema"));
  let _ = registry.register_aggregate_schema(builder_b.finish().expect("schema"));

  let report = registry.audit();
  assert!(!report.success());
  assert!(report.issues.iter().any(|issue| issue.reason.contains("cycle detected")));
}

#[test]
fn audit_detects_manifest_collisions() {
  let registry = SerializerRegistry::<NoStdToolbox>::new();
  let handle = SerializerHandle::new(BincodeSerializer::new());
  registry
    .bind_type::<ManifestA, _>(&handle, Some("shared-manifest".into()), decode_manifest_a)
    .expect("bind manifest a");

  let duplicate_binding = ArcShared::new(TypeBinding::new(
    TypeId::of::<ManifestB>(),
    "shared-manifest".into(),
    handle.identifier(),
    &handle,
    decode_manifest_b,
  ));
  registry.type_bindings.lock().insert(TypeId::of::<ManifestB>(), duplicate_binding);

  let report = registry.audit();
  assert!(!report.success());
  assert!(report.issues.iter().any(|issue| issue.reason.contains("manifest collision")));
}

#[test]
fn assign_default_serializer_for_pekko_serializable() {
  let registry = SerializerRegistry::<NoStdToolbox>::new();
  let handle = SerializerHandle::new(BincodeSerializer::new());
  registry.register_serializer(handle.clone()).expect("register serializer");

  registry.assign_default_serializer::<AutoType>().expect("assign default");
  registry.assign_default_serializer::<AutoType>().expect("idempotent assign");

  assert!(registry.has_binding_for::<AutoType>());
  let binding =
    registry.find_binding_by_manifest(handle.identifier(), core::any::type_name::<AutoType>()).expect("binding");
  let sample = AutoType(99);
  let erased: &dyn erased_serde::Serialize = &sample;
  let bytes = handle.serialize_erased(erased).expect("serialize");
  let recovered: AutoType = binding.deserialize_as(bytes.as_ref()).expect("deserialize");
  assert_eq!(recovered, sample);
}

#[test]
fn assign_default_serializer_respects_manifest_override() {
  let registry = SerializerRegistry::<NoStdToolbox>::new();
  let handle = SerializerHandle::new(BincodeSerializer::new());
  registry.register_serializer(handle.clone()).expect("register serializer");

  registry.assign_default_serializer::<CustomManifestType>().expect("assign default");
  assert!(registry.find_binding_by_manifest(handle.identifier(), "pekko.custom").is_ok());
}

#[test]
fn assign_default_serializer_detects_manifest_collision() {
  let registry = SerializerRegistry::<NoStdToolbox>::new();
  let handle = SerializerHandle::new(BincodeSerializer::new());
  registry.register_serializer(handle.clone()).expect("register serializer");

  registry.assign_default_serializer::<CustomManifestType>().expect("first assign");
  let error = registry.assign_default_serializer::<DuplicateManifestType>().expect_err("collision should fail");
  assert!(matches!(error, SerializationError::InvalidManifest(manifest) if manifest == "pekko.custom"));
}
