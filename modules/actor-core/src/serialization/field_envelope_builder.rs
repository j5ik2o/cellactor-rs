//! Builder responsible for assembling parent serialized payloads from child field payloads.

use alloc::{string::String, vec::Vec as AllocVec};

use heapless::Vec;

use super::{
  bytes::Bytes,
  constants::MAX_FIELDS_PER_AGGREGATE,
  error::SerializationError,
  field_payload::FieldPayload,
  payload::SerializedPayload,
};

#[cfg(test)]
mod tests;

/// Aggregates child field payloads into a single serialized representation.
pub struct FieldEnvelopeBuilder {
  serializer_id: u32,
  manifest:      String,
  fields:        Vec<FieldPayload, MAX_FIELDS_PER_AGGREGATE>,
}

const ENVELOPE_MAGIC: [u8; 4] = *b"AGGR";

impl FieldEnvelopeBuilder {
  /// Creates a new builder for the specified root manifest and serializer.
  #[must_use]
  pub fn new(serializer_id: u32, manifest: impl Into<String>) -> Self {
    Self { serializer_id, manifest: manifest.into(), fields: Vec::new() }
  }

  /// Appends a child field payload in traversal order.
  pub fn append_child(&mut self, payload: &FieldPayload) -> Result<(), SerializationError> {
    self
      .fields
      .push(payload.clone())
      .map_err(|_| SerializationError::SerializationFailed("too many field payloads".into()))
  }

  /// Consumes the builder and returns the final serialized payload.
  pub fn finalize(self) -> Result<SerializedPayload, SerializationError> {
    let mut buffer = AllocVec::new();
    buffer.extend_from_slice(&ENVELOPE_MAGIC);
    let count = u16::try_from(self.fields.len()).map_err(|_| SerializationError::SerializationFailed("field count overflow".into()))?;
    buffer.extend_from_slice(&count.to_le_bytes());

    for field in self.fields.iter() {
      buffer.extend_from_slice(&field.field_path_hash().to_le_bytes());
      buffer.extend_from_slice(&field.serializer_id().to_le_bytes());

      let manifest_bytes = field.manifest().as_bytes();
      let manifest_len = u16::try_from(manifest_bytes.len())
        .map_err(|_| SerializationError::SerializationFailed("manifest too long".into()))?;
      buffer.extend_from_slice(&manifest_len.to_le_bytes());
      buffer.extend_from_slice(manifest_bytes);

      let field_bytes = field.raw_bytes();
      let payload_len = u32::try_from(field_bytes.len()).map_err(|_| SerializationError::SerializationFailed("payload too large".into()))?;
      buffer.extend_from_slice(&payload_len.to_le_bytes());
      buffer.extend_from_slice(field_bytes.as_ref());
    }

    Ok(SerializedPayload::new(self.serializer_id, self.manifest, Bytes::from_vec(buffer)))
  }
}
