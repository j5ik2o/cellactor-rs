//! Immutable list of field indices describing traversal order.

use heapless::Vec;

use super::constants::MAX_FIELDS_PER_AGGREGATE;

/// Immutable plan describing how to visit aggregate fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldTraversalPlan {
  order: Vec<usize, MAX_FIELDS_PER_AGGREGATE>,
}

impl FieldTraversalPlan {
  /// Constructs a plan from an index vector.
  #[must_use]
  pub fn new(order: Vec<usize, MAX_FIELDS_PER_AGGREGATE>) -> Self {
    Self { order }
  }

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
