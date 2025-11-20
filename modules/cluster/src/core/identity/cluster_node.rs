use alloc::string::String;

use super::node_id::NodeId;

/// Represents a concrete cluster node taking ownership of virtual actors.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ClusterNode {
  id:      NodeId,
  address: String,
  weight:  u32,
  blocked: bool,
}

impl ClusterNode {
  /// Creates a new cluster node description.
  #[must_use]
  pub fn new(id: NodeId, address: impl Into<String>, weight: u32, blocked: bool) -> Self {
    Self { id, address: address.into(), weight, blocked }
  }

  /// Returns the node identifier.
  #[must_use]
  pub fn id(&self) -> &NodeId {
    &self.id
  }

  /// Returns the node address string.
  #[must_use]
  pub fn address(&self) -> &str {
    &self.address
  }

  /// Returns the configured rendezvous weight.
  #[must_use]
  pub fn weight(&self) -> u32 {
    self.weight
  }

  /// Indicates whether the node is currently blocked.
  #[must_use]
  pub fn is_blocked(&self) -> bool {
    self.blocked
  }
}
