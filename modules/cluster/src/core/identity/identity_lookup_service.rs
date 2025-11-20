use fraktor_utils_rs::core::{
  runtime_toolbox::{RuntimeToolbox, SyncMutexFamily, ToolboxMutex},
  sync::sync_mutex_like::SyncMutexLike,
};

use super::{
  cluster_identity::ClusterIdentity, cluster_node::ClusterNode, hash_ring_provider::HashRingProvider, node_id::NodeId,
  topology_snapshot::TopologySnapshot,
};
use crate::core::config::HashStrategy;

/// Identity lookup service backed by a rendezvous hash ring.
pub struct IdentityLookupService<TB>
where
  TB: RuntimeToolbox, {
  ring: ToolboxMutex<HashRingProvider, TB>,
}

impl<TB> IdentityLookupService<TB>
where
  TB: RuntimeToolbox,
{
  /// Creates a new lookup service configured with the given strategy.
  #[must_use]
  pub fn new(strategy: HashStrategy, hash_seed: u64) -> Self {
    let provider = HashRingProvider::new(strategy, hash_seed);
    Self { ring: <TB::MutexFamily as SyncMutexFamily>::create(provider) }
  }

  /// Applies the latest topology snapshot to the ring.
  pub fn update_topology(&self, snapshot: &TopologySnapshot) {
    let mut guard = self.ring.lock();
    guard.rebuild(snapshot);
  }

  /// Selects an owner node for the provided identity.
  #[must_use]
  pub fn select_owner(&self, identity: &ClusterIdentity, requester: &NodeId) -> Option<ClusterNode> {
    let guard = self.ring.lock();
    guard.select(identity, requester)
  }

  /// Exposes the hash of the last applied topology snapshot.
  #[must_use]
  pub fn topology_hash(&self) -> u64 {
    let guard = self.ring.lock();
    guard.topology_hash()
  }
}
