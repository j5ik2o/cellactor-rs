//! Builder for aggregate schemas.

use alloc::vec::Vec as AllocVec;
use core::any::{Any, TypeId};

use cellactor_utils_core_rs::sync::ArcShared;
use heapless::Vec;
use serde::Serialize;

use super::{
  aggregate_accessors::AggregateAccessors, aggregate_field_extractor::TypedFieldExtractor,
  aggregate_schema::AggregateSchema, aggregate_schema_registration::AggregateSchemaRegistration,
  constants::MAX_FIELDS_PER_AGGREGATE, envelope_mode::EnvelopeMode, error::SerializationError, field_node::FieldNode,
  field_options::FieldOptions, field_path::FieldPath, field_path_display::FieldPathDisplay, pure_value::is_pure_value,
  traversal_policy::TraversalPolicy,
};

#[cfg(test)]
mod tests;

/// Builder used to declare nested fields for an aggregate type.
#[derive(Debug)]
pub struct AggregateSchemaBuilder<T: Any + Send + Sync + 'static> {
  root_display:     FieldPathDisplay,
  traversal_policy: TraversalPolicy,
  fields:           Vec<FieldNode, MAX_FIELDS_PER_AGGREGATE>,
  field_hashes:     AllocVec<u128>,
  accessors:        AggregateAccessors,
  _marker:          core::marker::PhantomData<T>,
}

impl<T: Any + Send + Sync + 'static> AggregateSchemaBuilder<T> {
  /// Creates a new builder for the provided root display and traversal policy.
  #[must_use]
  pub fn new(traversal_policy: TraversalPolicy, root_display: FieldPathDisplay) -> Self {
    Self {
      root_display,
      traversal_policy,
      fields: Vec::new(),
      field_hashes: AllocVec::new(),
      accessors: AggregateAccessors::new(TypeId::of::<T>()),
      _marker: core::marker::PhantomData,
    }
  }

  /// Adds a new field with the provided path and options.
  pub fn add_field<F, Accessor>(
    &mut self,
    path: FieldPath,
    display: FieldPathDisplay,
    options: FieldOptions,
    accessor: Accessor,
  ) -> Result<&mut Self, SerializationError>
  where
    F: Serialize + Any + Send + Sync + 'static,
    Accessor: for<'a> Fn(&'a T) -> &'a F + Send + Sync + 'static, {
    if display.len() > super::constants::MAX_FIELD_PATH_BYTES {
      return Err(SerializationError::InvalidAggregateSchema("field path display exceeds maximum length"));
    }
    if options.external_serializer_allowed() && options.envelope_mode() != EnvelopeMode::PreserveOrder {
      return Err(SerializationError::InvalidAggregateSchema("external serializer requires preserve-order envelope"));
    }
    if options.external_serializer_allowed() && !is_pure_value::<F>() {
      return Err(SerializationError::InvalidAggregateSchema("external serializer requires pure value type"));
    }
    if self.fields.len() == MAX_FIELDS_PER_AGGREGATE {
      return Err(SerializationError::InvalidAggregateSchema("too many fields in aggregate"));
    }
    let extractor = ArcShared::new(TypedFieldExtractor::<T, F, Accessor>::new(accessor));
    let accessor_index = self.accessors.push(extractor)?;
    let node = FieldNode::new::<F>(path, display, options, accessor_index as u16);
    if self.field_hashes.iter().any(|existing| *existing == node.path_hash()) {
      return Err(SerializationError::InvalidAggregateSchema("duplicate field path"));
    }
    self.field_hashes.push(node.path_hash());
    self.fields.push(node).map_err(|_| SerializationError::InvalidAggregateSchema("too many fields"))?;
    Ok(self)
  }

  /// Finalizes the builder into an aggregate schema.
  pub fn finish(self) -> Result<AggregateSchemaRegistration, SerializationError> {
    if self.fields.is_empty() {
      return Err(SerializationError::InvalidAggregateSchema("aggregate must contain at least one field"));
    }
    if self.accessors.len() != self.fields.len() {
      return Err(SerializationError::InvalidAggregateSchema("accessor count mismatch"));
    }
    let schema = AggregateSchema::new(
      TypeId::of::<T>(),
      core::any::type_name::<T>(),
      self.root_display,
      self.traversal_policy,
      self.fields,
    );
    Ok(AggregateSchemaRegistration::new(schema, self.accessors))
  }
}

impl<T: Any + Send + Sync + 'static> AggregateSchemaBuilder<T> {
  /// Convenience helper for adding a value field with optional external support.
  pub fn add_value_field<F, Accessor>(
    &mut self,
    path: FieldPath,
    display: FieldPathDisplay,
    external_allowed: bool,
    accessor: Accessor,
  ) -> Result<&mut Self, SerializationError>
  where
    F: Serialize + Any + Send + Sync + 'static,
    Accessor: for<'a> Fn(&'a T) -> &'a F + Send + Sync + 'static, {
    self.add_field::<F, _>(
      path,
      display,
      FieldOptions::new(EnvelopeMode::PreserveOrder).with_external_serializer_allowed(external_allowed),
      accessor,
    )
  }
}
