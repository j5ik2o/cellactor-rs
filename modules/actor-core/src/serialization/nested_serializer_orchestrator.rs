//! Nested serializer orchestrator that walks aggregate schemas and produces field envelopes.

use alloc::string::ToString;
use core::{any::Any, time::Duration};

use cellactor_utils_core_rs::sync::ArcShared;
use erased_serde::Serialize as ErasedSerialize;

use super::{
  AggregateAccessors,
  AggregateSchema,
  FieldEnvelopeBuilder,
  FieldPayload,
  FieldTraversalEngine,
  FieldValueRef,
  SerializerRegistry,
  SerializedPayload,
  SerializationError,
  field_node::FieldNode,
  serialization_telemetry::SerializationTelemetry,
  type_binding::TypeBinding,
};
use crate::RuntimeToolbox;

#[cfg(test)]
mod tests;

/// Coordinates schema traversal and payload assembly for aggregates.
pub(super) struct NestedSerializerOrchestrator<TB: RuntimeToolbox + 'static> {
  registry: ArcShared<SerializerRegistry<TB>>,
  telemetry: ArcShared<dyn SerializationTelemetry>,
}

impl<TB: RuntimeToolbox + 'static> NestedSerializerOrchestrator<TB> {
  /// Creates a new orchestrator backed by the provided registry.
  pub(super) fn new(
    registry: ArcShared<SerializerRegistry<TB>>,
    telemetry: ArcShared<dyn SerializationTelemetry>,
  ) -> Self {
    Self { registry, telemetry }
  }

  /// Serializes the provided value, falling back to direct bindings when no aggregate schema exists.
  pub(super) fn serialize<T>(&self, value: &T) -> Result<SerializedPayload, SerializationError>
  where
    T: ErasedSerialize + Any + Send + Sync + 'static,
  {
    match (
      self.registry.load_schema::<T>(),
      self.registry.load_accessors::<T>(),
      self.registry.find_binding_by_type::<T>(),
    ) {
      (Ok(schema), Ok(accessors), Ok(binding)) => self.serialize_with_schema(value, &schema, &accessors, &binding),
      _ => self.serialize_direct::<T>(value),
    }
  }

  fn serialize_direct<T>(&self, value: &T) -> Result<SerializedPayload, SerializationError>
  where
    T: ErasedSerialize + Send + Sync + 'static,
  {
    let binding = self.registry.find_binding_by_type::<T>()?;
    let erased: &dyn ErasedSerialize = value;
    let bytes = binding.serializer().serialize_erased(erased)?;
    Ok(SerializedPayload::new(binding.serializer_id(), binding.manifest().to_string(), bytes))
  }

  fn serialize_with_schema(
    &self,
    value: &dyn Any,
    schema: &AggregateSchema,
    accessors: &AggregateAccessors,
    binding: &ArcShared<TypeBinding>,
  ) -> Result<SerializedPayload, SerializationError> {
    let telemetry = &*self.telemetry;
    let _scope = AggregateTelemetryScope::new(telemetry);
    let plan = FieldTraversalEngine::build(schema)?;
    let mut builder = FieldEnvelopeBuilder::new(binding.serializer_id(), binding.manifest().to_string());
    for index in plan.iter() {
      let node = schema
        .fields()
        .get(index)
        .ok_or(SerializationError::InvalidAggregateSchema("field traversal index out of bounds"))?;
      let field_hash = node.path_hash();
      let field_value = match accessors.extract(index, value) {
        Ok(field_value) => field_value,
        Err(error) => {
          telemetry.record_failure(field_hash, &error);
          telemetry.record_latency(field_hash, Duration::ZERO);
          return Err(error);
        },
      };
      let payload = match self.serialize_field(node, field_value) {
        Ok(payload) => payload,
        Err(error) => {
          telemetry.record_failure(field_hash, &error);
          telemetry.record_latency(field_hash, Duration::ZERO);
          return Err(error);
        },
      };
      if let Err(error) = builder.append_child(&payload) {
        telemetry.record_failure(field_hash, &error);
        telemetry.record_latency(field_hash, Duration::ZERO);
        return Err(error);
      }
      telemetry.record_success(field_hash);
      telemetry.record_latency(field_hash, Duration::ZERO);
    }
    builder.finalize()
  }

  fn serialize_field(&self, node: &FieldNode, field_value: FieldValueRef) -> Result<FieldPayload, SerializationError> {
    if let Some(schema) = self.registry.load_schema_by_id(node.type_id()) {
      let accessors = self
        .registry
        .load_accessors_by_id(node.type_id())
        .ok_or(SerializationError::AggregateSchemaNotFound(node.type_name()))?;
      let binding = self.registry.find_binding_by_id(node.type_id(), node.type_name())?;
      let nested = self.serialize_with_schema(field_value.as_any(), &schema, &accessors, &binding)?;
      let manifest = nested.manifest().to_string();
      let serializer_id = nested.serializer_id();
      let bytes = nested.into_bytes();
      Ok(FieldPayload::new(bytes, manifest, serializer_id, node.path_hash()))
    } else {
      let binding = self.registry.find_binding_by_id(node.type_id(), node.type_name())?;
      let bytes = binding.serializer().serialize_erased(field_value.as_erased())?;
      Ok(FieldPayload::new(bytes, binding.manifest().to_string(), binding.serializer_id(), node.path_hash()))
    }
  }
}

struct AggregateTelemetryScope<'a> {
  telemetry: &'a dyn SerializationTelemetry,
}

impl<'a> AggregateTelemetryScope<'a> {
  fn new(telemetry: &'a dyn SerializationTelemetry) -> Self {
    telemetry.on_aggregate_start();
    Self { telemetry }
  }
}

impl<'a> Drop for AggregateTelemetryScope<'a> {
  fn drop(&mut self) {
    self.telemetry.on_aggregate_finish();
  }
}
