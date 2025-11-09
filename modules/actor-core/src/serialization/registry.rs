//! Serializer registry.

use alloc::string::{String, ToString};
use core::any::{Any, TypeId};

use cellactor_utils_core_rs::{
  runtime_toolbox::SyncMutexFamily,
  sync::{ArcShared, sync_mutex_like::SyncMutexLike},
};
use hashbrown::{HashMap, hash_map::Entry};

use super::{
  aggregate_schema::AggregateSchema, error::SerializationError,
  external_serializer_policy::ExternalSerializerPolicyEntry, field_path_hash::FieldPathHash,
  serializer::SerializerHandle, type_binding::TypeBinding,
};
use crate::{RuntimeToolbox, ToolboxMutex};

#[cfg(test)]
mod tests;

/// Stores serializers and type bindings for a given actor system.
pub struct SerializerRegistry<TB: RuntimeToolbox + 'static> {
  serializers:       ToolboxMutex<HashMap<u32, SerializerHandle>, TB>,
  type_bindings:     ToolboxMutex<HashMap<TypeId, ArcShared<TypeBinding>>, TB>,
  manifest_bindings: ToolboxMutex<HashMap<ManifestKey, ArcShared<TypeBinding>>, TB>,
  aggregate_schemas: ToolboxMutex<HashMap<TypeId, ArcShared<AggregateSchema>>, TB>,
  field_policies:    ToolboxMutex<HashMap<FieldPathHash, ExternalSerializerPolicyEntry>, TB>,
}

impl<TB: RuntimeToolbox + 'static> Default for SerializerRegistry<TB> {
  fn default() -> Self {
    Self::new()
  }
}

impl<TB: RuntimeToolbox + 'static> SerializerRegistry<TB> {
  /// Creates an empty registry.
  #[must_use]
  pub fn new() -> Self {
    Self {
      serializers:       <TB::MutexFamily as SyncMutexFamily>::create(HashMap::new()),
      type_bindings:     <TB::MutexFamily as SyncMutexFamily>::create(HashMap::new()),
      manifest_bindings: <TB::MutexFamily as SyncMutexFamily>::create(HashMap::new()),
      aggregate_schemas: <TB::MutexFamily as SyncMutexFamily>::create(HashMap::new()),
      field_policies:    <TB::MutexFamily as SyncMutexFamily>::create(HashMap::new()),
    }
  }

  /// Registers a serializer handle.
  ///
  /// # Errors
  ///
  /// Returns an error if the serializer ID is already registered.
  pub fn register_serializer(&self, handle: SerializerHandle) -> Result<(), SerializationError> {
    let mut serializers = self.serializers.lock();
    match serializers.entry(handle.identifier()) {
      | Entry::Occupied(_) => Err(SerializationError::DuplicateSerializerId(handle.identifier())),
      | Entry::Vacant(slot) => {
        slot.insert(handle);
        Ok(())
      },
    }
  }

  /// Binds a concrete type to the provided serializer.
  ///
  /// # Errors
  ///
  /// Returns an error if the manifest is invalid or the type is already bound.
  pub fn bind_type<T, F>(
    &self,
    serializer: &SerializerHandle,
    manifest: Option<String>,
    deserializer: F,
  ) -> Result<(), SerializationError>
  where
    T: Any + Send + Sync + 'static,
    F: Fn(&[u8]) -> Result<T, SerializationError> + Send + Sync + 'static, {
    let manifest_value = manifest.unwrap_or_else(|| core::any::type_name::<T>().to_string());
    if manifest_value.trim().is_empty() {
      return Err(SerializationError::InvalidManifest(manifest_value));
    }
    let key = ManifestKey::new(serializer.identifier(), manifest_value.clone());

    {
      let manifest_guard = self.manifest_bindings.lock();
      if manifest_guard.contains_key(&key) {
        return Err(SerializationError::InvalidManifest(manifest_value));
      }
    }

    let mut type_guard = self.type_bindings.lock();
    if type_guard.contains_key(&TypeId::of::<T>()) {
      return Err(SerializationError::InvalidManifest(manifest_value));
    }

    let serializer_id = serializer.identifier();
    let binding =
      ArcShared::new(TypeBinding::new(TypeId::of::<T>(), manifest_value, serializer_id, serializer, deserializer));

    self.manifest_bindings.lock().insert(key, binding.clone());
    type_guard.insert(TypeId::of::<T>(), binding);
    Ok(())
  }

  /// Removes a binding by [`TypeId`].
  pub fn unbind_type(&self, type_id: TypeId) {
    if let Some(binding) = self.type_bindings.lock().remove(&type_id) {
      let manifest_key = ManifestKey::new(binding.serializer_id(), binding.manifest().to_string());
      self.manifest_bindings.lock().remove(&manifest_key);
    }
  }

  /// Removes a binding by `(serializer_id, manifest)`.
  pub fn unbind_manifest(&self, serializer_id: u32, manifest: &str) {
    let key = ManifestKey::new(serializer_id, manifest.to_string());
    if let Some(binding) = self.manifest_bindings.lock().remove(&key) {
      self.type_bindings.lock().remove(&binding.type_id());
    }
  }

  /// Finds a type binding by [`TypeId`].
  pub(super) fn find_binding_by_type<T>(&self) -> Result<ArcShared<TypeBinding>, SerializationError>
  where
    T: Any + 'static, {
    self
      .type_bindings
      .lock()
      .get(&TypeId::of::<T>())
      .cloned()
      .ok_or(SerializationError::NoSerializerForType(core::any::type_name::<T>()))
  }

  /// Finds a binding by manifest.
  pub(super) fn find_binding_by_manifest(
    &self,
    serializer_id: u32,
    manifest: &str,
  ) -> Result<ArcShared<TypeBinding>, SerializationError> {
    let key = ManifestKey::new(serializer_id, manifest.to_string());
    self
      .manifest_bindings
      .lock()
      .get(&key)
      .cloned()
      .ok_or_else(|| SerializationError::InvalidManifest(manifest.to_string()))
  }

  /// Finds a serializer handle by identifier.
  pub(super) fn find_serializer_by_id(&self, identifier: u32) -> Result<SerializerHandle, SerializationError> {
    self.serializers.lock().get(&identifier).cloned().ok_or(SerializationError::SerializerNotFound(identifier))
  }

  /// Returns `true` when a binding for the provided type exists.
  #[must_use]
  pub fn has_binding_for<T>(&self) -> bool
  where
    T: Any + 'static, {
    self.type_bindings.lock().contains_key(&TypeId::of::<T>())
  }

  /// Returns whether an external serializer is allowed for the provided path, if known.
  #[must_use]
  pub fn field_policy(&self, hash: FieldPathHash) -> Option<bool> {
    self.field_policies.lock().get(&hash).map(|entry| entry.external_allowed())
  }

  /// Registers an aggregate schema for later lookups.
  pub fn register_aggregate_schema(&self, schema: AggregateSchema) -> Result<(), SerializationError> {
    let type_id = schema.root_type();
    let type_name = schema.root_type_name();
    let schema_arc = ArcShared::new(schema);

    let mut schemas_guard = self.aggregate_schemas.lock();
    if schemas_guard.contains_key(&type_id) {
      return Err(SerializationError::AggregateSchemaAlreadyRegistered(type_name));
    }

    let mut policies_guard = self.field_policies.lock();
    for node in schema_arc.fields() {
      policies_guard.insert(node.path_hash(), ExternalSerializerPolicyEntry::from_field_node(node));
    }

    schemas_guard.insert(type_id, schema_arc);
    Ok(())
  }

  /// Loads the schema registered for the specified aggregate type.
  pub fn load_schema<T>(&self) -> Result<ArcShared<AggregateSchema>, SerializationError>
  where
    T: Any + 'static, {
    self
      .aggregate_schemas
      .lock()
      .get(&TypeId::of::<T>())
      .cloned()
      .ok_or(SerializationError::AggregateSchemaNotFound(core::any::type_name::<T>()))
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ManifestKey {
  serializer_id: u32,
  manifest:      String,
}

impl ManifestKey {
  const fn new(serializer_id: u32, manifest: String) -> Self {
    Self { serializer_id, manifest }
  }
}
