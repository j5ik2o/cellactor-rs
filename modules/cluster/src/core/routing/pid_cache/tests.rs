use fraktor_actor_rs::core::actor_prim::Pid;
use fraktor_utils_rs::core::runtime_toolbox::NoStdToolbox;

use crate::core::{
  identity::{ClusterIdentity, NodeId},
  routing::pid_cache::{PidCache, PidCacheEntry},
};

fn cache() -> PidCache<NoStdToolbox> {
  PidCache::new()
}

#[test]
fn insert_and_get_entry() {
  let cache = cache();
  let identity = ClusterIdentity::new("echo", "a");
  let entry = PidCacheEntry::new(Pid::new(1, 0), NodeId::new("node-a"), 42);
  cache.insert(identity.clone(), entry.clone());

  assert_eq!(cache.get(&identity), Some(entry));
}

#[test]
fn invalidate_identity_removes_entry() {
  let cache = cache();
  let identity = ClusterIdentity::new("echo", "a");
  cache.insert(identity.clone(), PidCacheEntry::new(Pid::new(1, 0), NodeId::new("node-a"), 42));

  assert!(cache.invalidate(&identity));
  assert!(cache.get(&identity).is_none());
}

#[test]
fn invalidate_node_clears_related_entries() {
  let cache = cache();
  let id_a = ClusterIdentity::new("echo", "a");
  let id_b = ClusterIdentity::new("echo", "b");
  cache.insert(id_a.clone(), PidCacheEntry::new(Pid::new(1, 0), NodeId::new("node-a"), 42));
  cache.insert(id_b.clone(), PidCacheEntry::new(Pid::new(2, 0), NodeId::new("node-b"), 42));

  let removed = cache.invalidate_node(&NodeId::new("node-a"));

  assert_eq!(removed, vec![id_a.clone()]);
  assert!(cache.get(&id_a).is_none());
  assert!(cache.get(&id_b).is_some());
}

#[test]
fn keys_returns_snapshot() {
  let cache = cache();
  let id_a = ClusterIdentity::new("echo", "a");
  cache.insert(id_a.clone(), PidCacheEntry::new(Pid::new(1, 0), NodeId::new("node-a"), 1));

  let keys = cache.keys();

  assert_eq!(keys, vec![id_a]);
}

#[test]
fn clear_flushes_cache() {
  let cache = cache();
  let identity = ClusterIdentity::new("echo", "a");
  cache.insert(identity.clone(), PidCacheEntry::new(Pid::new(1, 0), NodeId::new("node-a"), 42));

  cache.clear();

  assert!(cache.get(&identity).is_none());
}
