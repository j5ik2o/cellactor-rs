//! UTF-8 representation of a field path for telemetry purposes.

use alloc::string::String;

use heapless::Vec;

use super::{constants::MAX_FIELD_PATH_BYTES, error::SerializationError};

/// Human-readable field path kept for telemetry/logging.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldPathDisplay {
  bytes: Vec<u8, MAX_FIELD_PATH_BYTES>,
}

impl FieldPathDisplay {
  /// Creates a display string from UTF-8 input.
  pub fn from_str(value: &str) -> Result<Self, SerializationError> {
    if value.is_empty() {
      return Err(SerializationError::InvalidAggregateSchema("field path display must not be empty"));
    }
    let mut bytes = Vec::new();
    if bytes.extend_from_slice(value.as_bytes()).is_err() {
      return Err(SerializationError::InvalidAggregateSchema("field path display too long"));
    }
    Ok(Self { bytes })
  }

  /// Returns the stored bytes.
  #[must_use]
  pub fn as_bytes(&self) -> &[u8] {
    self.bytes.as_slice()
  }

  /// Returns the stored length.
  #[must_use]
  pub fn len(&self) -> usize {
    self.bytes.len()
  }

  /// Returns the stored string slice.
  #[must_use]
  pub fn as_str(&self) -> &str {
    core::str::from_utf8(self.bytes.as_slice()).expect("validated UTF-8")
  }

  /// Converts into a heap-allocated string.
  #[must_use]
  pub fn into_string(self) -> String {
    String::from(self.as_str())
  }
}

impl From<FieldPathDisplay> for String {
  fn from(value: FieldPathDisplay) -> Self {
    value.into_string()
  }
}
