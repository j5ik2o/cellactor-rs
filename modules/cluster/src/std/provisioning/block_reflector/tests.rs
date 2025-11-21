use std::sync::Mutex;

use crate::{
  core::{
    identity::{ClusterNode, NodeId},
    provisioning::{
      descriptor::ProviderId,
      snapshot::{ProviderHealth, ProviderSnapshot},
    },
  },
  std::provisioning::{
    block_reflector::apply_block_event,
    provider_event::{RemoteTopologyEvent, RemoteTopologyKind},
  },
};

fn snapshot(health: ProviderHealth) -> ProviderSnapshot {
  ProviderSnapshot {
    members: vec![ClusterNode::new(NodeId::new("n1"), "127.0.0.1", 1, false)],
    hash: 5,
    blocked_nodes: vec![],
    health,
  }
}

fn event(kind: RemoteTopologyKind) -> RemoteTopologyEvent {
  RemoteTopologyEvent {
    seq_no: 1,
    provider_id: ProviderId::new("p1"),
    snapshot_hash: 5,
    node_id: NodeId::new("n2"),
    kind,
  }
}

#[test]
fn blocks_and_unblocks_nodes() {
  let mut snap = snapshot(ProviderHealth::Healthy);
  apply_block_event(&mut snap, &event(RemoteTopologyKind::Blocked), |_| {});
  assert_eq!(vec![NodeId::new("n2")], snap.blocked_nodes);

  apply_block_event(&mut snap, &event(RemoteTopologyKind::Unblocked), |_| {});
  assert!(snap.blocked_nodes.is_empty());
}

#[test]
fn emits_warning_when_degraded_and_blocked() {
  let mut snap = snapshot(ProviderHealth::Degraded);
  let warnings: Mutex<Vec<String>> = Mutex::new(Vec::new());

  apply_block_event(&mut snap, &event(RemoteTopologyKind::Blocked), |msg| {
    warnings.lock().unwrap().push(msg);
  });

  let stored = warnings.lock().unwrap().clone();
  assert_eq!(1, stored.len());
  assert!(stored[0].contains("n2"));
}
