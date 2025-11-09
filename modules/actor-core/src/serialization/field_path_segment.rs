//! Numeric segment used inside a field path.

/// Numeric identifier representing a single segment in a field path.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FieldPathSegment {
  value: u16,
}

impl FieldPathSegment {
  /// Creates a new segment from a numeric index.
  #[must_use]
  pub const fn new(value: u16) -> Self {
    Self { value }
  }

  /// Returns the inner index value.
  #[must_use]
  pub const fn value(&self) -> u16 {
    self.value
  }
}
