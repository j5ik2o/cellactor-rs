use alloc::vec::Vec;

use fraktor_actor_rs::core::actor_prim::Pid;
use fraktor_utils_rs::core::runtime_toolbox::{RuntimeToolbox, SyncMutexFamily, ToolboxMutex};
use fraktor_utils_rs::core::sync::sync_mutex_like::SyncMutexLike;

use crate::core::identity::{ClusterIdentity, NodeId};

/// Cached PID entry with owner metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PidCacheEntry {
    pid: Pid,
    owner: NodeId,
    topology_hash: u64,
}

impl PidCacheEntry {
    /// Builds a new cache entry.
    pub fn new(pid: Pid, owner: NodeId, topology_hash: u64) -> Self {
        Self { pid, owner, topology_hash }
    }

    /// Returns cached PID reference.
    pub fn pid(&self) -> Pid {
        self.pid
    }

    /// Returns owning node.
    pub fn owner(&self) -> &NodeId {
        &self.owner
    }

    /// Returns topology hash.
    pub fn topology_hash(&self) -> u64 {
        self.topology_hash
    }
}

type CacheMap = hashbrown::HashMap<ClusterIdentity, PidCacheEntry, rapidhash::RapidBuildHasher>;

/// PID cache keyed by cluster identity.
pub struct PidCache<TB>
where
    TB: RuntimeToolbox,
{
    entries: ToolboxMutex<CacheMap, TB>,
}

impl<TB> PidCache<TB>
where
    TB: RuntimeToolbox,
{
    /// Creates an empty cache.
    pub fn new() -> Self {
        let map = CacheMap::with_hasher(rapidhash::RapidBuildHasher::default());
        Self { entries: <TB::MutexFamily as SyncMutexFamily>::create(map) }
    }

    /// Stores or replaces an entry for the identity.
    pub fn insert(&self, identity: ClusterIdentity, entry: PidCacheEntry) {
        let mut guard = self.entries.lock();
        guard.insert(identity, entry);
    }

    /// Fetches an entry if cached.
    pub fn get(&self, identity: &ClusterIdentity) -> Option<PidCacheEntry> {
        let guard = self.entries.lock();
        guard.get(identity).cloned()
    }

    /// Invalidates a specific identity entry.
    pub fn invalidate(&self, identity: &ClusterIdentity) -> bool {
        let mut guard = self.entries.lock();
        guard.remove(identity).is_some()
    }

    /// Invalidates all entries belonging to the node.
    pub fn invalidate_node(&self, node: &NodeId) -> Vec<ClusterIdentity> {
        let mut guard = self.entries.lock();
        let mut removed = Vec::new();
        guard.retain(|identity, entry| {
            if entry.owner() == node {
                removed.push(identity.clone());
                false
            } else {
                true
            }
        });
        removed
    }

    /// Clears the cache.
    pub fn clear(&self) {
        let mut guard = self.entries.lock();
        guard.clear();
    }
}

#[cfg(test)]
mod tests;
