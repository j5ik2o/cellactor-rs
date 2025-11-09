//! Compact numeric representation of a field path.

use heapless::Vec;

use super::{constants::MAX_FIELD_PATH_DEPTH, error::SerializationError, field_path_segment::FieldPathSegment};

/// Numeric field path used internally for hashing and comparisons.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldPath {
  segments: Vec<FieldPathSegment, MAX_FIELD_PATH_DEPTH>,
}

impl FieldPath {
  /// Builds a path from the provided segments.
  pub fn from_segments(segments: &[FieldPathSegment]) -> Result<Self, SerializationError> {
    if segments.is_empty() {
      return Err(SerializationError::InvalidAggregateSchema("field path requires at least one segment"));
    }
    let mut buffer = Vec::new();
    if buffer.extend_from_slice(segments).is_err() {
      return Err(SerializationError::InvalidAggregateSchema("field path too deep"));
    }
    Ok(Self { segments: buffer })
  }

  /// Returns the stored segments.
  #[must_use]
  pub fn segments(&self) -> &[FieldPathSegment] {
    self.segments.as_slice()
  }
}
