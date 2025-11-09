use alloc::{vec, vec::Vec};

use crate::serialization::{
  AggregateSchemaBuilder,
  FieldPath,
  FieldPathDisplay,
  FieldPathSegment,
  FieldTraversalEngine,
  TraversalPolicy,
};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct Leaf(u32);

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct RootAggregate {
  first: Leaf,
  second: Leaf,
  third: Leaf,
  fourth: Leaf,
}

const LEAF_SAMPLE: Leaf = Leaf(0);

#[test]
fn depth_first_plan_orders_by_path() {
  let schema = build_schema(TraversalPolicy::DepthFirst);
  let plan = FieldTraversalEngine::build(&schema).expect("plan");
  let order = plan
    .iter()
    .map(|index| schema.fields()[index].display().as_str())
    .collect::<Vec<_>>();
  assert_eq!(order, vec!["root.first", "root.first.grand", "root.first.sibling", "root.second"]);
}

#[test]
fn breadth_first_plan_orders_by_depth() {
  let schema = build_schema(TraversalPolicy::BreadthFirst);
  let plan = FieldTraversalEngine::build(&schema).expect("plan");
  let order = plan
    .iter()
    .map(|index| schema.fields()[index].display().as_str())
    .collect::<Vec<_>>();
  assert_eq!(order, vec!["root.first", "root.second", "root.first.grand", "root.first.sibling"]);
}

fn build_schema(policy: TraversalPolicy) -> crate::serialization::AggregateSchema {
  let mut builder = AggregateSchemaBuilder::<RootAggregate>::new(
    policy,
    FieldPathDisplay::from_str("root").expect("display"),
  );
  builder
    .add_value_field::<Leaf, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("root.first").expect("display"),
      false,
      |aggregate| &aggregate.first,
    )
    .expect("first field");
  builder
    .add_value_field::<Leaf, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(0), FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("root.first.grand").expect("display"),
      false,
      |_| &LEAF_SAMPLE,
    )
    .expect("nested field");
  builder
    .add_value_field::<Leaf, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(0), FieldPathSegment::new(1)]).expect("path"),
      FieldPathDisplay::from_str("root.first.sibling").expect("display"),
      false,
      |_| &LEAF_SAMPLE,
    )
    .expect("nested sibling");
  builder
    .add_value_field::<Leaf, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(1)]).expect("path"),
      FieldPathDisplay::from_str("root.second").expect("display"),
      false,
      |aggregate| &aggregate.second,
    )
    .expect("second field");
  builder.finish().expect("schema").into_parts().0
}
