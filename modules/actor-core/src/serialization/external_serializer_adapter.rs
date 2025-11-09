//! Adapter that serializes pure value fields via an external serializer.

use alloc::string::ToString;

use super::{
  FieldNode, FieldPayload, FieldValueRef, SerializationError, bincode_serializer::BincodeSerializer,
  serializer::SerializerHandle,
};

/// Externally serializes pure value fields when no registry binding exists.
pub(super) struct ExternalSerializerAdapter {
  serializer: SerializerHandle,
}

impl ExternalSerializerAdapter {
  /// Creates a new adapter backed by the built-in bincode serializer.
  #[must_use]
  pub(super) fn new() -> Self {
    Self { serializer: SerializerHandle::new(BincodeSerializer::new()) }
  }

  /// Serializes the provided field value into a [`FieldPayload`].
  pub(super) fn serialize(&self, field: &FieldNode, value: &FieldValueRef) -> Result<FieldPayload, SerializationError> {
    let bytes = self.serializer.serialize_erased(value.as_erased())?;
    let manifest = field.type_name().to_string();
    Ok(FieldPayload::new(bytes, manifest, self.serializer.identifier(), field.path_hash()))
  }
}
