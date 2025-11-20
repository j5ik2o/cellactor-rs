use alloc::string::String;
use core::hash::{Hash, Hasher};

/// Uniquely identifies a virtual actor within the cluster.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClusterIdentity {
  kind:     String,
  identity: String,
}

impl ClusterIdentity {
  /// Creates a new cluster identity from the provided kind and logical identity.
  #[must_use]
  pub fn new(kind: impl Into<String>, identity: impl Into<String>) -> Self {
    Self { kind: kind.into(), identity: identity.into() }
  }

  /// Returns the registered kind name.
  #[must_use]
  pub fn kind(&self) -> &str {
    &self.kind
  }

  /// Returns the logical identity component.
  #[must_use]
  pub fn identity(&self) -> &str {
    &self.identity
  }
}

impl Hash for ClusterIdentity {
  fn hash<H>(&self, state: &mut H)
  where
    H: Hasher, {
    state.write(self.kind.as_bytes());
    state.write_u8(0xff);
    state.write(self.identity.as_bytes());
  }
}
