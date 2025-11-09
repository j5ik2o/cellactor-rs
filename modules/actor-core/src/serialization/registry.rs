//! Serializer registry.

use alloc::{
  format,
  string::{String, ToString},
  vec::Vec,
};
use core::any::{Any, TypeId};

use cellactor_utils_core_rs::{
  runtime_toolbox::SyncMutexFamily,
  sync::{ArcShared, sync_mutex_like::SyncMutexLike},
};
use hashbrown::{HashMap, hash_map::Entry};
use portable_atomic::{AtomicU64, Ordering};

use super::{
  aggregate_accessors::AggregateAccessors,
  aggregate_schema::AggregateSchema,
  aggregate_schema_registration::AggregateSchemaRegistration,
  constants::MAX_FIELD_PATH_BYTES,
  error::SerializationError,
  external_serializer_policy::ExternalSerializerPolicyEntry,
  field_path_hash::FieldPathHash,
  pekko_assignment_metrics::PekkoAssignmentMetrics,
  pekko_serializable::PekkoSerializable,
  registry_audit::{RegistryAuditIssue, RegistryAuditReport},
  serializer::SerializerHandle,
  type_binding::TypeBinding,
};
use crate::{RuntimeToolbox, ToolboxMutex};

#[cfg(test)]
mod tests;

/// Stores serializers and type bindings for a given actor system.
pub struct SerializerRegistry<TB: RuntimeToolbox + 'static> {
  serializers:              ToolboxMutex<HashMap<u32, SerializerHandle>, TB>,
  type_bindings:            ToolboxMutex<HashMap<TypeId, ArcShared<TypeBinding>>, TB>,
  manifest_bindings:        ToolboxMutex<HashMap<ManifestKey, ArcShared<TypeBinding>>, TB>,
  aggregate_schemas:        ToolboxMutex<HashMap<TypeId, ArcShared<AggregateSchema>>, TB>,
  aggregate_accessors:      ToolboxMutex<HashMap<TypeId, ArcShared<AggregateAccessors>>, TB>,
  field_policies:           ToolboxMutex<HashMap<FieldPathHash, ExternalSerializerPolicyEntry>, TB>,
  pekko_assignment_success: AtomicU64,
  pekko_assignment_failure: AtomicU64,
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
      serializers:              <TB::MutexFamily as SyncMutexFamily>::create(HashMap::new()),
      type_bindings:            <TB::MutexFamily as SyncMutexFamily>::create(HashMap::new()),
      manifest_bindings:        <TB::MutexFamily as SyncMutexFamily>::create(HashMap::new()),
      aggregate_schemas:        <TB::MutexFamily as SyncMutexFamily>::create(HashMap::new()),
      aggregate_accessors:      <TB::MutexFamily as SyncMutexFamily>::create(HashMap::new()),
      field_policies:           <TB::MutexFamily as SyncMutexFamily>::create(HashMap::new()),
      pekko_assignment_success: AtomicU64::new(0),
      pekko_assignment_failure: AtomicU64::new(0),
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

  /// Finds a type binding by [`TypeId`].
  pub(super) fn find_binding_by_id(
    &self,
    type_id: TypeId,
    type_name: &'static str,
  ) -> Result<ArcShared<TypeBinding>, SerializationError> {
    self.type_bindings.lock().get(&type_id).cloned().ok_or(SerializationError::NoSerializerForType(type_name))
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

  /// Ensures that a Pekko-compatible type is bound to its default serializer.
  pub fn assign_default_serializer<T>(&self) -> Result<(), SerializationError>
  where
    T: PekkoSerializable, {
    if self.has_binding_for::<T>() {
      return Ok(());
    }

    let serializer_id = T::pekko_serializer_id();
    let serializer = self.find_serializer_by_id(serializer_id)?;
    let manifest_value =
      T::pekko_manifest().map(|value| value.to_string()).unwrap_or_else(|| core::any::type_name::<T>().to_string());

    match self.bind_type::<T, _>(&serializer, Some(manifest_value), T::pekko_decode) {
      | Ok(()) => {
        self.pekko_assignment_success.fetch_add(1, Ordering::Relaxed);
        Ok(())
      },
      | Err(error) => {
        self.pekko_assignment_failure.fetch_add(1, Ordering::Relaxed);
        Err(error)
      },
    }
  }

  /// Returns Pekko assignment counters for observability.
  #[must_use]
  pub fn pekko_assignment_metrics(&self) -> PekkoAssignmentMetrics {
    PekkoAssignmentMetrics {
      success_total: self.pekko_assignment_success.load(Ordering::Relaxed),
      failure_total: self.pekko_assignment_failure.load(Ordering::Relaxed),
    }
  }

  /// Returns whether an external serializer is allowed for the provided path, if known.
  #[must_use]
  pub fn field_policy(&self, hash: FieldPathHash) -> Option<bool> {
    self.field_policies.lock().get(&hash).map(|entry| entry.external_allowed())
  }

  /// Runs an audit across all registered schemas and reports structural issues.
  pub fn audit(&self) -> RegistryAuditReport {
    let schemas_guard = self.aggregate_schemas.lock();
    let type_bindings_guard = self.type_bindings.lock();
    let mut issues = Vec::new();

    Self::detect_schema_field_issues(&schemas_guard, &type_bindings_guard, &mut issues);
    Self::detect_schema_cycles(&schemas_guard, &mut issues);
    Self::detect_manifest_collisions(&type_bindings_guard, &mut issues);

    RegistryAuditReport::new(schemas_guard.len(), issues)
  }

  fn detect_schema_field_issues(
    schemas: &HashMap<TypeId, ArcShared<AggregateSchema>>,
    bindings: &HashMap<TypeId, ArcShared<TypeBinding>>,
    issues: &mut Vec<RegistryAuditIssue>,
  ) {
    for schema in schemas.values() {
      Self::check_display_length(schema.root_display().as_str(), schema.root_type_name(), issues);
      for field in schema.fields() {
        Self::check_display_length(field.display().as_str(), field.type_name(), issues);
        if !field.external_serializer_allowed() && !bindings.contains_key(&field.type_id()) {
          issues.push(RegistryAuditIssue {
            field_path: field.display().as_str().to_string(),
            type_name:  field.type_name(),
            reason:     "serializer not registered".to_string(),
          });
        }
      }
    }
  }

  fn detect_manifest_collisions(
    bindings: &HashMap<TypeId, ArcShared<TypeBinding>>,
    issues: &mut Vec<RegistryAuditIssue>,
  ) {
    let mut manifest_index: HashMap<ManifestKey, &'static str> = HashMap::new();
    for binding in bindings.values() {
      let key = ManifestKey::new(binding.serializer_id(), binding.manifest().to_string());
      if let Some(existing) = manifest_index.insert(key.clone(), binding.type_name()) {
        issues.push(RegistryAuditIssue {
          field_path: format!("serializer={} manifest={}", key.serializer_id(), key.manifest()),
          type_name:  binding.type_name(),
          reason:     format!("manifest collision with {}", existing),
        });
      }
    }
  }

  fn detect_schema_cycles(schemas: &HashMap<TypeId, ArcShared<AggregateSchema>>, issues: &mut Vec<RegistryAuditIssue>) {
    let mut adjacency: HashMap<TypeId, Vec<(TypeId, String)>> = HashMap::new();
    for (type_id, schema) in schemas.iter() {
      let entry = adjacency.entry(*type_id).or_insert_with(Vec::new);
      for field in schema.fields() {
        if schemas.contains_key(&field.type_id()) {
          entry.push((field.type_id(), field.display().as_str().to_string()));
        }
      }
    }

    let mut visit_state: HashMap<TypeId, AuditVisitState> = HashMap::new();
    let mut path: Vec<(TypeId, Option<String>)> = Vec::new();

    for type_id in adjacency.keys().copied() {
      if visit_state.get(&type_id).is_none() {
        path.push((type_id, None));
        Self::explore_cycles(type_id, &adjacency, &mut visit_state, &mut path, schemas, issues);
        path.pop();
      }
    }
  }

  fn explore_cycles(
    current: TypeId,
    adjacency: &HashMap<TypeId, Vec<(TypeId, String)>>,
    visit_state: &mut HashMap<TypeId, AuditVisitState>,
    path: &mut Vec<(TypeId, Option<String>)>,
    schemas: &HashMap<TypeId, ArcShared<AggregateSchema>>,
    issues: &mut Vec<RegistryAuditIssue>,
  ) {
    visit_state.insert(current, AuditVisitState::Visiting);
    if let Some(edges) = adjacency.get(&current) {
      for (next, display) in edges {
        match visit_state.get(next) {
          | Some(AuditVisitState::Visiting) => {
            let mut segments = Vec::new();
            if let Some(position) = path.iter().position(|entry| entry.0 == *next) {
              for entry in path.iter().skip(position + 1) {
                if let Some(via) = &entry.1 {
                  segments.push(via.clone());
                }
              }
            }
            segments.push(display.clone());
            let field_path = segments.join(" -> ");
            let type_name = schemas.get(next).map(|schema| schema.root_type_name()).unwrap_or("unknown");
            issues.push(RegistryAuditIssue { field_path, type_name, reason: "cycle detected".to_string() });
          },
          | Some(AuditVisitState::Done) => continue,
          | None => {
            path.push((*next, Some(display.clone())));
            Self::explore_cycles(*next, adjacency, visit_state, path, schemas, issues);
            path.pop();
          },
        }
      }
    }
    visit_state.insert(current, AuditVisitState::Done);
  }

  fn check_display_length(path: &str, type_name: &'static str, issues: &mut Vec<RegistryAuditIssue>) {
    if path.as_bytes().len() > MAX_FIELD_PATH_BYTES {
      issues.push(RegistryAuditIssue {
        field_path: path.to_string(),
        type_name,
        reason: "FieldPathDisplay exceeds maximum length".to_string(),
      });
    }
  }

  /// Registers an aggregate schema for later lookups.
  pub fn register_aggregate_schema(&self, registration: AggregateSchemaRegistration) -> Result<(), SerializationError> {
    let (schema, accessors) = registration.into_parts();
    let type_id = schema.root_type();
    let type_name = schema.root_type_name();
    if accessors.root_type() != type_id {
      return Err(SerializationError::InvalidAggregateSchema("accessor root mismatch"));
    }
    if accessors.len() != schema.fields().len() {
      return Err(SerializationError::InvalidAggregateSchema("accessor count mismatch"));
    }

    let schema_arc = ArcShared::new(schema);
    let access_arc = ArcShared::new(accessors);

    let mut schemas_guard = self.aggregate_schemas.lock();
    if schemas_guard.contains_key(&type_id) {
      return Err(SerializationError::AggregateSchemaAlreadyRegistered(type_name));
    }

    let mut access_guard = self.aggregate_accessors.lock();
    access_guard.insert(type_id, access_arc);

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

  /// Loads a schema by [`TypeId`].
  pub(super) fn load_schema_by_id(&self, type_id: TypeId) -> Option<ArcShared<AggregateSchema>> {
    self.aggregate_schemas.lock().get(&type_id).cloned()
  }

  /// Loads the field accessors registered for the specified aggregate type.
  pub fn load_accessors<T>(&self) -> Result<ArcShared<AggregateAccessors>, SerializationError>
  where
    T: Any + 'static, {
    self
      .aggregate_accessors
      .lock()
      .get(&TypeId::of::<T>())
      .cloned()
      .ok_or(SerializationError::AggregateSchemaNotFound(core::any::type_name::<T>()))
  }

  /// Loads field accessors by [`TypeId`].
  pub(super) fn load_accessors_by_id(&self, type_id: TypeId) -> Option<ArcShared<AggregateAccessors>> {
    self.aggregate_accessors.lock().get(&type_id).cloned()
  }
}

/// Tracks DFS visitation for schema audit graph traversal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AuditVisitState {
  Visiting,
  Done,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ManifestKey {
  serializer_id: u32,
  manifest:      String,
}

impl ManifestKey {
  fn new(serializer_id: u32, manifest: String) -> Self {
    Self { serializer_id, manifest }
  }

  const fn serializer_id(&self) -> u32 {
    self.serializer_id
  }

  fn manifest(&self) -> &str {
    &self.manifest
  }
}
