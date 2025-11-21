//! Provider descriptor used by provisioning registry.

use alloc::string::String;

/// Identifier for a provider instance.
use serde::{Deserialize, Serialize};

/// Logical identifier for a provider entry.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProviderId(String);

impl ProviderId {
  /// Creates a new provider id.
  #[must_use]
  pub fn new(id: impl Into<String>) -> Self {
    Self(id.into())
  }

  /// Returns the underlying id as str.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

/// Supported provider kinds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
  /// In-memory static provider intended for local/dev.
  InMemory,
  /// Consul-based membership provider.
  Consul,
  /// Kubernetes endpoints informer-based provider.
  Kubernetes,
  /// Custom user-provided provider identified by name.
  Custom(String),
}

/// Descriptor containing provider configuration metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderDescriptor {
  id:       ProviderId,
  kind:     ProviderKind,
  priority: u8,
  endpoint: Option<String>,
}

impl ProviderDescriptor {
  /// Builds a new descriptor.
  #[must_use]
  pub fn new(id: ProviderId, kind: ProviderKind, priority: u8) -> Self {
    Self { id, kind, priority, endpoint: None }
  }

  /// Provider identifier.
  #[must_use]
  pub fn id(&self) -> &ProviderId {
    &self.id
  }

  /// Provider kind.
  #[must_use]
  pub fn kind(&self) -> &ProviderKind {
    &self.kind
  }

  /// Priority (高いほど優先)。
  #[must_use]
  pub fn priority(&self) -> u8 {
    self.priority
  }

  /// Optional endpoint (Consul/K8s/custom が利用)。
  #[must_use]
  pub fn endpoint(&self) -> Option<&str> {
    self.endpoint.as_deref()
  }

  /// Sets endpoint and returns self for chaining.
  #[must_use]
  pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
    self.endpoint = Some(endpoint.into());
    self
  }
}
