//! Descriptor containing provider configuration metadata.

use alloc::string::String;

use serde::{Deserialize, Serialize};

use super::{ProviderId, ProviderKind};

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
  pub const fn new(id: ProviderId, kind: ProviderKind, priority: u8) -> Self {
    Self { id, kind, priority, endpoint: None }
  }

  /// Provider identifier.
  #[must_use]
  pub const fn id(&self) -> &ProviderId {
    &self.id
  }

  /// Provider kind.
  #[must_use]
  pub const fn kind(&self) -> &ProviderKind {
    &self.kind
  }

  /// Priority value (higher value means higher priority).
  #[must_use]
  pub const fn priority(&self) -> u8 {
    self.priority
  }

  /// Optional endpoint (used by Consul/K8s/custom providers).
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
