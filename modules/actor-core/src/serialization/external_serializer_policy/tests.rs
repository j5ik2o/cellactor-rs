use cellactor_utils_core_rs::sync::ArcShared;

use super::ExternalSerializerPolicy;
use crate::{
  NoStdToolbox,
  serialization::{
    AggregateSchemaBuilder, FieldPath, FieldPathDisplay, FieldPathSegment, SerializerRegistry, TraversalPolicy,
  },
};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct SampleAggregate {
  allowed: u32,
  denied:  u32,
}

#[test]
fn policy_allows_registered_external_field() {
  let (registry, schema) = register_schema(true);
  let policy = ExternalSerializerPolicy::new(registry.clone());
  let allowed =
    schema.fields().iter().find(|field| field.display().as_str() == "aggregate.allowed").expect("allowed node");
  assert!(policy.enforce(allowed).is_ok());
}

#[test]
fn policy_rejects_forbidden_field() {
  let (registry, schema) = register_schema(false);
  let policy = ExternalSerializerPolicy::new(registry.clone());
  let denied =
    schema.fields().iter().find(|field| field.display().as_str() == "aggregate.denied").expect("denied node");
  let err = policy.enforce(denied).expect_err("field must be rejected");
  assert!(matches!(err, crate::serialization::SerializationError::SerializationFailed(message)
    if message.contains("aggregate.denied")));
}

fn register_schema(
  include_allowed: bool,
) -> (ArcShared<SerializerRegistry<NoStdToolbox>>, ArcShared<crate::serialization::AggregateSchema>) {
  let registry = ArcShared::new(SerializerRegistry::<NoStdToolbox>::new());
  let mut builder = AggregateSchemaBuilder::<SampleAggregate>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("aggregate").expect("display"),
  );
  if include_allowed {
    builder
      .add_value_field::<u32, _>(
        FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
        FieldPathDisplay::from_str("aggregate.allowed").expect("display"),
        true,
        |aggregate| &aggregate.allowed,
      )
      .expect("allowed field");
  }
  builder
    .add_value_field::<u32, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(1)]).expect("path"),
      FieldPathDisplay::from_str("aggregate.denied").expect("display"),
      false,
      |aggregate| &aggregate.denied,
    )
    .expect("denied field");

  let registration = builder.finish().expect("registration");
  registry.register_aggregate_schema(registration).expect("register schema");
  let schema = registry.load_schema::<SampleAggregate>().expect("schema");
  (registry, schema)
}
