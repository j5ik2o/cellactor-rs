use fraktor_actor_rs::core::actor_prim::Pid;

use crate::core::identity::NodeId;

/// Cached PID entry with owner metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PidCacheEntry {
  pid:           Pid,
  owner:         NodeId,
  topology_hash: u64,
}

impl PidCacheEntry {
  /// Builds a new cache entry.
  #[must_use]
  pub const fn new(pid: Pid, owner: NodeId, topology_hash: u64) -> Self {
    Self { pid, owner, topology_hash }
  }

  /// Returns cached PID reference.
  #[must_use]
  pub const fn pid(&self) -> Pid {
    self.pid
  }

  /// Returns owning node.
  #[must_use]
  pub const fn owner(&self) -> &NodeId {
    &self.owner
  }

  /// Returns topology hash.
  #[must_use]
  pub const fn topology_hash(&self) -> u64 {
    self.topology_hash
  }
}
