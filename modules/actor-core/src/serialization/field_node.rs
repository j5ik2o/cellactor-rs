//! Immutable metadata describing a single aggregate field.

use core::any::{Any, TypeId};

use super::{
  envelope_mode::EnvelopeMode,
  field_options::FieldOptions,
  field_path::FieldPath,
  field_path_display::FieldPathDisplay,
  field_path_hash::{FieldPathHash, compute_field_path_hash},
};

/// Immutable metadata for a registered field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldNode {
  path: FieldPath,
  display: FieldPathDisplay,
  path_hash: FieldPathHash,
  type_id: TypeId,
  type_name: &'static str,
  envelope_mode: EnvelopeMode,
  external_serializer_allowed: bool,
}

impl FieldNode {
  /// Creates a new field node using the provided options.
  pub fn new<T: Any + 'static>(path: FieldPath, display: FieldPathDisplay, options: FieldOptions) -> Self {
    let path_hash = compute_field_path_hash(&path, &display);
    Self {
      path,
      display,
      path_hash,
      type_id: TypeId::of::<T>(),
      type_name: core::any::type_name::<T>(),
      envelope_mode: options.envelope_mode(),
      external_serializer_allowed: options.external_serializer_allowed(),
    }
  }

  /// Returns the numeric path.
  #[must_use]
  pub fn path(&self) -> &FieldPath {
    &self.path
  }

  /// Returns the display string.
  #[must_use]
  pub fn display(&self) -> &FieldPathDisplay {
    &self.display
  }

  /// Returns the hashed representation.
  #[must_use]
  pub const fn path_hash(&self) -> FieldPathHash {
    self.path_hash
  }

  /// Returns the field type identifier.
  #[must_use]
  pub const fn type_id(&self) -> TypeId {
    self.type_id
  }

  /// Returns the field type name.
  #[must_use]
  pub const fn type_name(&self) -> &'static str {
    self.type_name
  }

  /// Returns the envelope mode.
  #[must_use]
  pub const fn envelope_mode(&self) -> EnvelopeMode {
    self.envelope_mode
  }

  /// Indicates whether external serializers are allowed.
  #[must_use]
  pub const fn external_serializer_allowed(&self) -> bool {
    self.external_serializer_allowed
  }
}
