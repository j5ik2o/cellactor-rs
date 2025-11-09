//! Immutable representation of an aggregate schema.

use core::any::TypeId;

use heapless::Vec;

use super::{
  constants::MAX_FIELDS_PER_AGGREGATE, field_node::FieldNode, field_path_display::FieldPathDisplay,
  traversal_policy::TraversalPolicy,
};

/// Aggregate schema metadata used during serialization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregateSchema {
  root_type:        TypeId,
  root_type_name:   &'static str,
  root_display:     FieldPathDisplay,
  traversal_policy: TraversalPolicy,
  fields:           Vec<FieldNode, MAX_FIELDS_PER_AGGREGATE>,
  version:          u32,
}

impl AggregateSchema {
  pub(crate) fn new(
    root_type: TypeId,
    root_type_name: &'static str,
    root_display: FieldPathDisplay,
    traversal_policy: TraversalPolicy,
    fields: Vec<FieldNode, MAX_FIELDS_PER_AGGREGATE>,
  ) -> Self {
    Self { root_type, root_type_name, root_display, traversal_policy, fields, version: 1 }
  }

  /// Returns the type identifier for the aggregate root.
  #[must_use]
  pub const fn root_type(&self) -> TypeId {
    self.root_type
  }

  /// Returns the aggregate root type name.
  #[must_use]
  pub const fn root_type_name(&self) -> &'static str {
    self.root_type_name
  }

  /// Returns the human-readable name for the root.
  #[must_use]
  pub const fn root_display(&self) -> &FieldPathDisplay {
    &self.root_display
  }

  /// Returns the traversal policy.
  #[must_use]
  pub const fn traversal_policy(&self) -> TraversalPolicy {
    self.traversal_policy
  }

  /// Returns the registered fields.
  #[must_use]
  pub fn fields(&self) -> &[FieldNode] {
    self.fields.as_slice()
  }

  /// Returns the schema version.
  #[must_use]
  pub const fn version(&self) -> u32 {
    self.version
  }
}
