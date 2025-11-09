//! Cached policy entries for external serializer checks.

use alloc::format;

use cellactor_utils_core_rs::sync::ArcShared;

use super::{SerializationError, field_node::FieldNode, field_path_hash::FieldPathHash, registry::SerializerRegistry};
use crate::RuntimeToolbox;

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

/// Enforces external serializer access rules for registered fields.
#[derive(Clone)]
pub(super) struct ExternalSerializerPolicy<TB: RuntimeToolbox + 'static> {
  registry: ArcShared<SerializerRegistry<TB>>,
}

impl<TB: RuntimeToolbox + 'static> ExternalSerializerPolicy<TB> {
  /// Creates a new policy view backed by the serializer registry.
  #[must_use]
  pub(super) fn new(registry: ArcShared<SerializerRegistry<TB>>) -> Self {
    Self { registry }
  }

  /// Ensures the provided field may leverage external serializers.
  pub(super) fn enforce(&self, field: &FieldNode) -> Result<(), SerializationError> {
    match self.registry.field_policy(field.path_hash()) {
      | Some(true) => Ok(()),
      | _ => Err(SerializationError::SerializationFailed(format!(
        "external serializer not allowed for field {}",
        field.display().as_str()
      ))),
    }
  }
}
