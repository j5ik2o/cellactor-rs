use alloc::string::String;
use core::hash::{Hash, Hasher};

/// Unique identifier for a cluster node/member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeId {
  value: String,
}

impl NodeId {
  /// Creates a new node identifier.
  #[must_use]
  pub fn new(value: impl Into<String>) -> Self {
    Self { value: value.into() }
  }

  /// Exposes the identifier as a string slice.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.value
  }
}

impl Hash for NodeId {
  fn hash<H>(&self, state: &mut H)
  where
    H: Hasher, {
    state.write(self.value.as_bytes());
  }
}
