//! Marker trait for Pekko-compatible default serialization bindings.

use alloc::string::ToString;
use serde::{Serialize, de::DeserializeOwned};

use super::error::SerializationError;

fn decode_from_bincode<T>(bytes: &[u8]) -> Result<T, SerializationError>
where
  T: DeserializeOwned, {
  bincode::serde::decode_from_slice(bytes, bincode::config::standard().with_fixed_int_encoding())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
}

/// Marker trait describing types that should automatically receive Pekko-compatible bindings.
pub trait PekkoSerializable: Serialize + DeserializeOwned + Send + Sync + 'static {
  /// Identifier of the serializer to bind this type to (defaults to built-in bincode: `1`).
  fn pekko_serializer_id() -> u32 {
    1
  }

  /// Optional manifest override (defaults to the Rust type name).
  fn pekko_manifest() -> Option<&'static str> {
    None
  }

  /// Deserializes bytes into `Self` when automatic bindings are created.
  fn pekko_decode(bytes: &[u8]) -> Result<Self, SerializationError>
  where
    Self: Sized, {
    decode_from_bincode(bytes)
  }
}
