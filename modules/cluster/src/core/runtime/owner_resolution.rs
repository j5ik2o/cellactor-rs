use crate::core::{activation::ActivationLease, identity::cluster_node::ClusterNode};

/// Captures the result of resolving a cluster identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnerResolution {
  owner: ClusterNode,
  lease: ActivationLease,
}

impl OwnerResolution {
  /// Creates a new resolution result.
  #[must_use]
  pub fn new(owner: ClusterNode, lease: ActivationLease) -> Self {
    Self { owner, lease }
  }

  /// Returns the selected owner node.
  #[must_use]
  pub fn owner(&self) -> &ClusterNode {
    &self.owner
  }

  /// Returns the lease granted for the identity.
  #[must_use]
  pub fn lease(&self) -> &ActivationLease {
    &self.lease
  }
}
