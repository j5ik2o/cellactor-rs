use crate::serialization::{
  AggregateSchemaBuilder, FieldPath, FieldPathDisplay, FieldPathSegment, TraversalPolicy, error::SerializationError,
};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct ChildAggregate(u32);

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct ParentAggregate {
  child: ChildAggregate,
  count: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct NonPure(u32);

impl Drop for NonPure {
  fn drop(&mut self) {}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct ImpureAggregate {
  value: NonPure,
}

#[test]
fn builder_exposes_field_accessors() {
  let mut builder = AggregateSchemaBuilder::<ParentAggregate>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("parent").expect("display"),
  );
  builder
    .add_value_field::<ChildAggregate, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("parent.child").expect("display"),
      false,
      |parent| &parent.child,
    )
    .expect("add child");
  builder
    .add_value_field::<u32, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(1)]).expect("path"),
      FieldPathDisplay::from_str("parent.count").expect("display"),
      false,
      |parent| &parent.count,
    )
    .expect("add count");

  let registration = builder.finish().expect("registration");
  let (schema, accessors) = registration.into_parts();

  assert_eq!(schema.fields().len(), 2);
  let aggregate = ParentAggregate { child: ChildAggregate(42), count: 7 };
  let first_index = schema.fields()[0].accessor_index() as usize;
  let second_index = schema.fields()[1].accessor_index() as usize;
  let first = accessors.extract(first_index, &aggregate).expect("child value");
  let second = accessors.extract(second_index, &aggregate).expect("count value");

  assert_eq!(first.as_any().downcast_ref::<ChildAggregate>().expect("child").0, 42);
  assert_eq!(*second.as_any().downcast_ref::<u32>().expect("count"), 7);
}

#[test]
fn finish_fails_when_no_fields() {
  let builder = AggregateSchemaBuilder::<ParentAggregate>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("parent").expect("display"),
  );
  let err = builder.finish().expect_err("should fail");
  assert!(matches!(err, SerializationError::InvalidAggregateSchema(_)));
}

#[test]
fn external_serializer_requires_pure_value() {
  let mut builder = AggregateSchemaBuilder::<ImpureAggregate>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("impure").expect("display"),
  );
  let err = builder
    .add_value_field::<NonPure, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("impure.value").expect("display"),
      true,
      |aggregate| &aggregate.value,
    )
    .expect_err("non pure value must fail");
  assert!(matches!(err, SerializationError::InvalidAggregateSchema(message) if message.contains("pure value")));
}
