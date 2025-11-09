//! Serialized child payload produced by field-level serializers.

use alloc::string::String;

use super::{bytes::Bytes, field_path_hash::FieldPathHash};

/// Encapsulates raw bytes and metadata for a nested field payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldPayload {
  raw_bytes:       Bytes,
  manifest:        String,
  serializer_id:   u32,
  field_path_hash: FieldPathHash,
}

impl FieldPayload {
  /// Creates a new field payload from its components.
  #[must_use]
  pub fn new(raw_bytes: Bytes, manifest: String, serializer_id: u32, field_path_hash: FieldPathHash) -> Self {
    Self { raw_bytes, manifest, serializer_id, field_path_hash }
  }

  /// Returns the serialized bytes for this field.
  #[must_use]
  pub fn raw_bytes(&self) -> &Bytes {
    &self.raw_bytes
  }

  /// Returns the manifest attached to this field.
  #[must_use]
  pub fn manifest(&self) -> &str {
    &self.manifest
  }

  /// Returns the serializer identifier used for this field.
  #[must_use]
  pub const fn serializer_id(&self) -> u32 {
    self.serializer_id
  }

  /// Returns the hash identifying the field path.
  #[must_use]
  pub const fn field_path_hash(&self) -> FieldPathHash {
    self.field_path_hash
  }
}
