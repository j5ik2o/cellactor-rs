//! Cached policy entries for external serializer checks.

use super::{field_node::FieldNode, field_path_hash::FieldPathHash};

/// Policy entry describing whether a field may use external serialization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ExternalSerializerPolicyEntry {
  field_path_hash:  FieldPathHash,
  external_allowed: bool,
}

#[allow(dead_code)]
impl ExternalSerializerPolicyEntry {
  /// Creates a policy entry from a field node.
  #[must_use]
  pub(super) fn from_field_node(node: &FieldNode) -> Self {
    Self { field_path_hash: node.path_hash(), external_allowed: node.external_serializer_allowed() }
  }

  /// Returns the path hash associated with this policy.
  #[must_use]
  pub(super) const fn field_path_hash(&self) -> FieldPathHash {
    self.field_path_hash
  }

  /// Indicates whether external serializers are allowed for this path.
  #[must_use]
  pub(super) const fn external_allowed(&self) -> bool {
    self.external_allowed
  }
}
