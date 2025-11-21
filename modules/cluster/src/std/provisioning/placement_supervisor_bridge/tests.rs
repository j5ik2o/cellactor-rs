use std::sync::{Arc, Mutex};

use crate::{
  core::{
    identity::{ClusterNode, NodeId},
    provisioning::{
      descriptor::ProviderId,
      snapshot::{ProviderHealth, ProviderSnapshot},
    },
  },
  std::provisioning::placement_supervisor_bridge::{
    PlacementBridgeError, PlacementSupervisorBridge, PlacementSupervisorPort,
  },
};

struct RecordingPort {
  snapshots: Mutex<Vec<u64>>,
  changes:   Mutex<Vec<(String, String)>>,
}

impl RecordingPort {
  fn new() -> Self {
    Self { snapshots: Mutex::new(Vec::new()), changes: Mutex::new(Vec::new()) }
  }
}

impl PlacementSupervisorPort for RecordingPort {
  fn apply_snapshot(&self, snapshot: &ProviderSnapshot) {
    self.snapshots.lock().unwrap().push(snapshot.hash);
  }

  fn provider_changed(&self, from: ProviderId, to: ProviderId) {
    self.changes.lock().unwrap().push((from.as_str().to_string(), to.as_str().to_string()));
  }
}

fn snapshot(hash: u64) -> ProviderSnapshot {
  ProviderSnapshot {
    members: vec![ClusterNode::new(NodeId::new("n1"), "127.0.0.1", 1, false)],
    hash,
    blocked_nodes: vec![],
    health: ProviderHealth::Healthy,
  }
}

#[test]
fn forwards_in_seq_order() {
  let port = Arc::new(RecordingPort::new());
  let bridge = PlacementSupervisorBridge::new(port.clone());

  bridge.apply_snapshot(1, &snapshot(9)).unwrap();
  bridge.provider_changed(2, ProviderId::new("a"), ProviderId::new("b")).unwrap();
  bridge.apply_snapshot(3, &snapshot(10)).unwrap();

  assert_eq!(vec![9, 10], *port.snapshots.lock().unwrap());
  assert_eq!(vec![("a".to_string(), "b".to_string())], *port.changes.lock().unwrap());
}

#[test]
fn rejects_out_of_order_seq() {
  let port = Arc::new(RecordingPort::new());
  let bridge = PlacementSupervisorBridge::new(port.clone());

  bridge.apply_snapshot(5, &snapshot(1)).unwrap();
  let err = bridge.apply_snapshot(4, &snapshot(2)).unwrap_err();

  assert!(matches!(err, PlacementBridgeError::OutOfOrder { seq_no: 4, last_seq: 5 }));
  assert_eq!(vec![1], *port.snapshots.lock().unwrap());
}

#[test]
fn rejects_after_shutdown() {
  let port = Arc::new(RecordingPort::new());
  let bridge = PlacementSupervisorBridge::new(port.clone());

  bridge.apply_snapshot(1, &snapshot(1)).unwrap();
  bridge.begin_shutdown();
  let err = bridge.provider_changed(2, ProviderId::new("x"), ProviderId::new("y")).unwrap_err();

  assert!(matches!(err, PlacementBridgeError::ShuttingDown));
  assert_eq!(vec![1], *port.snapshots.lock().unwrap());
  assert!(port.changes.lock().unwrap().is_empty());
}
