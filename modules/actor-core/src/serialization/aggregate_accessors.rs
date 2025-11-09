//! Collection of field extractors associated with an aggregate schema.

use core::{any::TypeId, fmt};

use cellactor_utils_core_rs::sync::ArcShared;
use heapless::Vec;

use super::{
  aggregate_field_extractor::AggregateFieldExtractor,
  constants::MAX_FIELDS_PER_AGGREGATE,
  error::SerializationError,
  field_value_ref::FieldValueRef,
};

/// Holds extractor functions for each field declared in an aggregate schema.
#[derive(Clone)]
pub struct AggregateAccessors {
  root_type:  TypeId,
  extractors: Vec<ArcShared<dyn AggregateFieldExtractor>, MAX_FIELDS_PER_AGGREGATE>,
}

impl fmt::Debug for AggregateAccessors {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f
      .debug_struct("AggregateAccessors")
      .field("root_type", &self.root_type)
      .field("len", &self.extractors.len())
      .finish()
  }
}

impl AggregateAccessors {
  /// Creates an empty accessor table for the specified aggregate type.
  #[must_use]
  pub const fn new(root_type: TypeId) -> Self {
    Self { root_type, extractors: Vec::new() }
  }

  /// Registers a new extractor and returns its assigned index.
  pub fn push(&mut self, extractor: ArcShared<dyn AggregateFieldExtractor>) -> Result<usize, SerializationError> {
    let index = self.extractors.len();
    self
      .extractors
      .push(extractor)
      .map_err(|_| SerializationError::InvalidAggregateSchema("too many field accessors"))?;
    Ok(index)
  }

  /// Returns the total number of extractors registered.
  #[must_use]
  pub fn len(&self) -> usize {
    self.extractors.len()
  }

  /// Returns the root aggregate [`TypeId`].
  #[must_use]
  pub const fn root_type(&self) -> TypeId {
    self.root_type
  }

  /// Extracts the field value for the specified index.
  pub fn extract<'a>(&self, index: usize, root: &'a dyn core::any::Any) -> Result<FieldValueRef<'a>, SerializationError> {
    self
      .extractors
      .get(index)
      .ok_or(SerializationError::InvalidAggregateSchema("accessor index out of bounds"))?
      .extract(root)
  }
}
