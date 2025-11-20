use alloc::vec;

use crate::core::config::HashStrategy;
use crate::core::identity::{ClusterIdentity, ClusterNode, NodeId, TopologySnapshot};

use super::HashRingProvider;

#[test]
fn rebuild_ignores_blocked_nodes() {
    let node_a = ClusterNode::new(NodeId::new("node-a"), "10.0.0.1", 1, false);
    let node_b = ClusterNode::new(NodeId::new("node-b"), "10.0.0.2", 5, true);
    let snapshot = TopologySnapshot::new(42, vec![node_a.clone(), node_b]);

    let mut provider = HashRingProvider::new(HashStrategy::WeightedRendezvous, 11);
    provider.rebuild(&snapshot);

    let identity = ClusterIdentity::new("echo", "abc");
    let requester = NodeId::new("caller");
    let selected = provider.select(&identity, &requester).expect("owner");

    assert_eq!(selected.id(), node_a.id());
    assert_eq!(provider.topology_hash(), 42);
}

#[test]
fn returns_none_when_ring_is_empty() {
    let provider = HashRingProvider::new(HashStrategy::Rendezvous, 7);
    let identity = ClusterIdentity::new("echo", "abc");
    let requester = NodeId::new("caller");

    assert!(provider.select(&identity, &requester).is_none());
}
