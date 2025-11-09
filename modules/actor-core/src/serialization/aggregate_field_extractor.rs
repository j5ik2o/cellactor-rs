//! Trait objects to extract field references from aggregate roots.

use core::{any::Any, marker::PhantomData};

use serde::Serialize;

use super::{error::SerializationError, field_value_ref::FieldValueRef};

/// Extracts a single field value from an aggregate instance.
pub trait AggregateFieldExtractor: Send + Sync {
  /// Returns the field value reference for the given aggregate.
  fn extract<'a>(&self, root: &'a dyn Any) -> Result<FieldValueRef<'a>, SerializationError>;
}

/// Typed field extractor that downcasts the aggregate before invoking the accessor.
pub(super) struct TypedFieldExtractor<T, F, Accessor>
where
  T: Any + Send + Sync + 'static,
  F: Serialize + Any + Send + Sync + 'static,
  Accessor: for<'a> Fn(&'a T) -> &'a F + Send + Sync + 'static, {
  accessor: Accessor,
  _marker:  PhantomData<(T, F)>,
}

impl<T, F, Accessor> TypedFieldExtractor<T, F, Accessor>
where
  T: Any + Send + Sync + 'static,
  F: Serialize + Any + Send + Sync + 'static,
  Accessor: for<'a> Fn(&'a T) -> &'a F + Send + Sync + 'static,
{
  /// Creates a new extractor wrapper.
  #[must_use]
  pub(super) const fn new(accessor: Accessor) -> Self {
    Self { accessor, _marker: PhantomData }
  }
}

impl<T, F, Accessor> AggregateFieldExtractor for TypedFieldExtractor<T, F, Accessor>
where
  T: Any + Send + Sync + 'static,
  F: Serialize + Any + Send + Sync + 'static,
  Accessor: for<'a> Fn(&'a T) -> &'a F + Send + Sync + 'static,
{
  fn extract<'a>(&self, root: &'a dyn Any) -> Result<FieldValueRef<'a>, SerializationError> {
    let typed =
      root.downcast_ref::<T>().ok_or(SerializationError::InvalidAggregateValue(core::any::type_name::<T>()))?;
    Ok(FieldValueRef::new((self.accessor)(typed)))
  }
}
