use std::sync::{Arc, Mutex};

use crate::core::identity::{ClusterNode, NodeId};
use crate::core::provisioning::descriptor::ProviderId;
use crate::core::provisioning::snapshot::{ProviderHealth, ProviderSnapshot};
use crate::std::provisioning::partition_manager_bridge::{PartitionManagerBridge, PartitionManagerError, PartitionManagerPort};

fn snapshot(hash: u64) -> ProviderSnapshot {
  ProviderSnapshot {
    members: vec![ClusterNode::new(NodeId::new("n1"), "127.0.0.1", 1, false)],
    hash,
    blocked_nodes: vec![],
    health: ProviderHealth::Healthy,
  }
}

struct MockPM {
  latest:  Mutex<Vec<u64>>,
  changes: Mutex<Vec<(String, String)>>,
}

impl MockPM {
  fn new() -> Self {
    Self { latest: Mutex::new(Vec::new()), changes: Mutex::new(Vec::new()) }
  }
}

impl PartitionManagerPort for MockPM {
  fn apply_snapshot(&self, snapshot: &ProviderSnapshot) {
    self.latest.lock().unwrap().push(snapshot.hash);
  }

  fn provider_changed(&self, from: ProviderId, to: ProviderId) {
    self.changes.lock().unwrap().push((from.as_str().to_string(), to.as_str().to_string()));
  }
}

#[test]
fn caches_and_returns_latest_snapshot() {
  let port = Arc::new(MockPM::new());
  let bridge = PartitionManagerBridge::new(port.clone());

  bridge.apply_snapshot(1, snapshot(11)).unwrap();
  let snap = bridge.latest_snapshot().unwrap();

  assert_eq!(11, snap.hash);
  assert_eq!(vec![11], *port.latest.lock().unwrap());
}

#[test]
fn errors_when_snapshot_missing() {
  let port = Arc::new(MockPM::new());
  let bridge = PartitionManagerBridge::new(port);
  let err = bridge.latest_snapshot().unwrap_err();
  assert_eq!(PartitionManagerError::NoSnapshot, err);
}

#[test]
fn records_provider_change() {
  let port = Arc::new(MockPM::new());
  let bridge = PartitionManagerBridge::new(port.clone());

  bridge.provider_changed(1, ProviderId::new("old"), ProviderId::new("new")).unwrap();

  assert_eq!(vec![("old".to_string(), "new".to_string())], *port.changes.lock().unwrap());
}

#[test]
fn rejects_out_of_order_seq() {
  let port = Arc::new(MockPM::new());
  let bridge = PartitionManagerBridge::new(port.clone());

  bridge.apply_snapshot(5, snapshot(1)).unwrap();
  let err = bridge.apply_snapshot(4, snapshot(2)).unwrap_err();

  assert!(matches!(err, PartitionManagerError::OutOfOrder { seq_no: 4, last_seq: 5 }));
  assert_eq!(vec![1], *port.latest.lock().unwrap());
}

#[test]
fn shutdown_rejects_new_updates_but_keeps_latest() {
  let port = Arc::new(MockPM::new());
  let bridge = PartitionManagerBridge::new(port.clone());

  bridge.apply_snapshot(1, snapshot(3)).unwrap();
  bridge.begin_shutdown();
  let err = bridge.apply_snapshot(2, snapshot(4)).unwrap_err();

  assert!(matches!(err, PartitionManagerError::ShuttingDown));
  assert_eq!(3, bridge.latest_snapshot().unwrap().hash);
  assert_eq!(vec![3], *port.latest.lock().unwrap());
}
