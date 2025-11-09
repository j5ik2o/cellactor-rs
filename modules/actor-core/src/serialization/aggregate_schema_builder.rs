//! Builder for aggregate schemas.

use alloc::vec::Vec as AllocVec;
use core::any::{Any, TypeId};

use heapless::Vec;

use super::{
  aggregate_schema::AggregateSchema, constants::MAX_FIELDS_PER_AGGREGATE, envelope_mode::EnvelopeMode,
  error::SerializationError, field_node::FieldNode, field_options::FieldOptions, field_path::FieldPath,
  field_path_display::FieldPathDisplay, pure_value::is_pure_value, traversal_policy::TraversalPolicy,
};

/// Builder used to declare nested fields for an aggregate type.
#[derive(Debug)]
pub struct AggregateSchemaBuilder<T: Any + 'static> {
  root_display:     FieldPathDisplay,
  traversal_policy: TraversalPolicy,
  fields:           Vec<FieldNode, MAX_FIELDS_PER_AGGREGATE>,
  field_hashes:     AllocVec<u128>,
  _marker:          core::marker::PhantomData<T>,
}

impl<T: Any + 'static> AggregateSchemaBuilder<T> {
  /// Creates a new builder for the provided root display and traversal policy.
  #[must_use]
  pub fn new(traversal_policy: TraversalPolicy, root_display: FieldPathDisplay) -> Self {
    Self {
      root_display,
      traversal_policy,
      fields: Vec::new(),
      field_hashes: AllocVec::new(),
      _marker: core::marker::PhantomData,
    }
  }

  /// Adds a new field with the provided path and options.
  pub fn add_field<F: Any + 'static>(
    &mut self,
    path: FieldPath,
    display: FieldPathDisplay,
    options: FieldOptions,
  ) -> Result<&mut Self, SerializationError> {
    if options.external_serializer_allowed() && !is_pure_value::<F>() {
      return Err(SerializationError::InvalidAggregateSchema("external serializer requires pure value type"));
    }
    if self.fields.len() == MAX_FIELDS_PER_AGGREGATE {
      return Err(SerializationError::InvalidAggregateSchema("too many fields in aggregate"));
    }
    let node = FieldNode::new::<F>(path, display, options);
    if self.field_hashes.iter().any(|existing| *existing == node.path_hash()) {
      return Err(SerializationError::InvalidAggregateSchema("duplicate field path"));
    }
    self.field_hashes.push(node.path_hash());
    self.fields.push(node).map_err(|_| SerializationError::InvalidAggregateSchema("too many fields"))?;
    Ok(self)
  }

  /// Finalizes the builder into an aggregate schema.
  pub fn finish(self) -> Result<AggregateSchema, SerializationError> {
    if self.fields.is_empty() {
      return Err(SerializationError::InvalidAggregateSchema("aggregate must contain at least one field"));
    }
    Ok(AggregateSchema::new(
      TypeId::of::<T>(),
      core::any::type_name::<T>(),
      self.root_display,
      self.traversal_policy,
      self.fields,
    ))
  }
}

impl<T: Any + 'static> AggregateSchemaBuilder<T> {
  /// Convenience helper for adding a value field with optional external support.
  pub fn add_value_field<F: Any + 'static>(
    &mut self,
    path: FieldPath,
    display: FieldPathDisplay,
    external_allowed: bool,
  ) -> Result<&mut Self, SerializationError> {
    self.add_field::<F>(
      path,
      display,
      FieldOptions::new(EnvelopeMode::PreserveOrder).with_external_serializer_allowed(external_allowed),
    )
  }
}
