//! Borrowed reference to a field value captured during aggregate traversal.

use core::any::Any;

use erased_serde::Serialize as ErasedSerialize;
use serde::Serialize;

/// Wrapper holding erased references to an aggregate field value.
pub struct FieldValueRef<'a> {
  any:    &'a dyn Any,
  erased: &'a dyn ErasedSerialize,
}

impl<'a> FieldValueRef<'a> {
  /// Creates a new wrapper from the provided reference.
  #[must_use]
  pub fn new<T>(value: &'a T) -> Self
  where
    T: Serialize + Any, {
    Self { any: value, erased: value }
  }

  /// Returns the value as an [`Any`] reference for downcasting.
  #[must_use]
  pub fn as_any(&self) -> &'a dyn Any {
    self.any
  }

  /// Returns the erased serialization trait object.
  #[must_use]
  pub fn as_erased(&self) -> &'a dyn ErasedSerialize {
    self.erased
  }
}
