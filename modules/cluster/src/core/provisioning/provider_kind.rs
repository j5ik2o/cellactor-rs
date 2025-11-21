//! Supported provider kinds.

use alloc::string::String;

use serde::{Deserialize, Serialize};

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
