use alloc::vec::Vec;

use super::cluster_node::ClusterNode;

/// Immutable snapshot of the current cluster membership state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopologySnapshot {
  hash:    u64,
  members: Vec<ClusterNode>,
}

impl TopologySnapshot {
  /// Creates a new snapshot from the provided nodes.
  #[must_use]
  pub const fn new(hash: u64, members: Vec<ClusterNode>) -> Self {
    Self { hash, members }
  }

  /// Returns the rendezvous hash of the snapshot contents.
  #[must_use]
  pub const fn hash(&self) -> u64 {
    self.hash
  }

  /// Returns the cluster nodes contained within the snapshot.
  #[must_use]
  pub fn members(&self) -> &[ClusterNode] {
    &self.members
  }
}
