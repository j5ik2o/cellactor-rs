//! Builds traversal plans for aggregate schemas.

use core::cmp::Ordering;

use heapless::Vec;

use super::{
  aggregate_schema::AggregateSchema,
  constants::MAX_FIELDS_PER_AGGREGATE,
  error::SerializationError,
  field_node::FieldNode,
  traversal_policy::TraversalPolicy,
};

#[cfg(test)]
mod tests;

/// Immutable list of field indices describing traversal order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldTraversalPlan {
  order: Vec<usize, MAX_FIELDS_PER_AGGREGATE>,
}

impl FieldTraversalPlan {
  /// Returns the number of entries in the plan.
  #[must_use]
  pub fn len(&self) -> usize {
    self.order.len()
  }

  /// Indicates whether the plan contains no entries.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.order.is_empty()
  }

  /// Returns an iterator over field indices following the traversal order.
  pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
    self.order.iter().copied()
  }
}

/// Computes traversal plans for aggregate schemas.
pub struct FieldTraversalEngine;

impl FieldTraversalEngine {
  /// Builds a traversal plan for the provided schema.
  pub fn build(schema: &AggregateSchema) -> Result<FieldTraversalPlan, SerializationError> {
    let mut indices = Vec::<usize, MAX_FIELDS_PER_AGGREGATE>::new();
    for (index, _) in schema.fields().iter().enumerate() {
      indices.push(index).map_err(|_| SerializationError::InvalidAggregateSchema("too many fields in schema"))?;
    }

    let policy = schema.traversal_policy();
    let fields = schema.fields();
    indices.as_mut_slice().sort_unstable_by(|lhs, rhs| compare(policy, &fields[*lhs], &fields[*rhs]));

    Ok(FieldTraversalPlan { order: indices })
  }
}

fn compare(policy: TraversalPolicy, lhs: &FieldNode, rhs: &FieldNode) -> Ordering {
  match policy {
    TraversalPolicy::DepthFirst => compare_depth_first(lhs, rhs),
    TraversalPolicy::BreadthFirst => {
      let depth_cmp = lhs.path().segments().len().cmp(&rhs.path().segments().len());
      if depth_cmp == Ordering::Equal {
        compare_depth_first(lhs, rhs)
      } else {
        depth_cmp
      }
    },
  }
}

fn compare_depth_first(lhs: &FieldNode, rhs: &FieldNode) -> Ordering {
  let lhs_segments = lhs.path().segments();
  let rhs_segments = rhs.path().segments();
  for (a, b) in lhs_segments.iter().zip(rhs_segments.iter()) {
    let cmp = a.value().cmp(&b.value());
    if cmp != Ordering::Equal {
      return cmp;
    }
  }
  lhs_segments.len().cmp(&rhs_segments.len())
}
