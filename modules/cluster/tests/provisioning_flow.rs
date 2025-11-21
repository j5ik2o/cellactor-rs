use std::sync::{Arc, Mutex};
use std::time::Duration;

use fraktor_cluster_rs::core::identity::{ClusterNode, NodeId};
use fraktor_cluster_rs::core::provisioning::descriptor::{ProviderDescriptor, ProviderId, ProviderKind};
use fraktor_cluster_rs::core::provisioning::snapshot::{ProviderHealth, ProviderSnapshot};
use fraktor_cluster_rs::std::provisioning::block_reflector::apply_block_event;
use fraktor_cluster_rs::std::provisioning::failover_controller::{FailoverConfig, FailoverController};
use fraktor_cluster_rs::std::provisioning::placement_supervisor_bridge::PlacementSupervisorBridge;
use fraktor_cluster_rs::std::provisioning::partition_manager_bridge::PartitionManagerBridge;
use fraktor_cluster_rs::std::provisioning::provider_event::{ProviderEvent, ProviderTermination, RemoteTopologyEvent, RemoteTopologyKind};
use fraktor_cluster_rs::std::provisioning::provider_watch_hub::ProviderWatchHub;
use fraktor_cluster_rs::std::provisioning::provisioning_metrics::ProvisioningMetrics;
use fraktor_cluster_rs::std::provisioning::remoting_bridge::RemotingBridge;
use fraktor_cluster_rs::std::provisioning::remoting_port::RemotingPort;

fn snapshot(hash: u64, health: ProviderHealth) -> ProviderSnapshot {
  ProviderSnapshot {
    members: vec![ClusterNode::new(NodeId::new("n1"), "127.0.0.1", 1, false)],
    hash,
    blocked_nodes: vec![],
    health,
  }
}

struct RecordingPS {
  snapshots: Mutex<Vec<u64>>,
  changes:   Mutex<Vec<(String, String)>>,
}

impl RecordingPS {
  fn new() -> Self {
    Self { snapshots: Mutex::new(Vec::new()), changes: Mutex::new(Vec::new()) }
  }
}

impl fraktor_cluster_rs::std::provisioning::placement_supervisor_bridge::PlacementSupervisorPort for RecordingPS {
  fn apply_snapshot(&self, snapshot: &ProviderSnapshot) {
    self.snapshots.lock().unwrap().push(snapshot.hash);
  }

  fn provider_changed(&self, from: ProviderId, to: ProviderId) {
    self
      .changes
      .lock()
      .unwrap()
      .push((from.as_str().to_string(), to.as_str().to_string()));
  }
}

struct RecordingPM {
  snapshots: Mutex<Vec<u64>>,
  changes:   Mutex<Vec<(String, String)>>,
}

impl RecordingPM {
  fn new() -> Self {
    Self { snapshots: Mutex::new(Vec::new()), changes: Mutex::new(Vec::new()) }
  }
}

impl fraktor_cluster_rs::std::provisioning::partition_manager_bridge::PartitionManagerPort for RecordingPM {
  fn apply_snapshot(&self, snapshot: &ProviderSnapshot) {
    self.snapshots.lock().unwrap().push(snapshot.hash);
  }

  fn provider_changed(&self, from: ProviderId, to: ProviderId) {
    self
      .changes
      .lock()
      .unwrap()
      .push((from.as_str().to_string(), to.as_str().to_string()));
  }
}

struct RecordingRemotingPort {
  events: Mutex<Vec<(String, u64)>>,
}

impl RecordingRemotingPort {
  fn new() -> Self {
    Self { events: Mutex::new(Vec::new()) }
  }
}

impl RemotingPort for RecordingRemotingPort {
  fn publish_remote_topology(&self, event: &RemoteTopologyEvent) {
    self
      .events
      .lock()
      .unwrap()
      .push((event.provider_id.as_str().to_string(), event.seq_no));
  }
}

#[test]
fn end_to_end_failover_and_notifications() {
  // setup descriptors and failover controller
  let primary = ProviderDescriptor::new(ProviderId::new("p1"), ProviderKind::InMemory, 10);
  let backup = ProviderDescriptor::new(ProviderId::new("p2"), ProviderKind::Consul, 5).with_endpoint("http://c");
  let mut failover = FailoverController::new(vec![primary.clone(), backup.clone()], FailoverConfig::default());

  let metrics = ProvisioningMetrics::new();
  let hub = ProviderWatchHub::new();

  let ps_port = Arc::new(RecordingPS::new());
  let pm_port = Arc::new(RecordingPM::new());
  let rem_port = Arc::new(RecordingRemotingPort::new());

  let ps_bridge = PlacementSupervisorBridge::new(ps_port.clone());
  let pm_bridge = PartitionManagerBridge::new(pm_port.clone());
  let rem_bridge = RemotingBridge::new(rem_port.clone());

  // initial snapshot from primary
  let seq1 = 1;
  let snap1 = snapshot(100, ProviderHealth::Healthy);
  hub.apply_event(ProviderEvent::Snapshot(snap1.clone())).unwrap();
  ps_bridge.apply_snapshot(seq1, &snap1).unwrap();
  pm_bridge.apply_snapshot(seq1, snap1.clone()).unwrap();
  metrics.record_snapshot_latency(seq1, Duration::from_millis(8));

  // fail primary to trigger failover to backup
  failover.record_failure(primary.id().as_str(), "timeout");
  failover.record_failure(primary.id().as_str(), "timeout");
  failover.record_failure(primary.id().as_str(), "timeout");
  let active = failover.select_active().unwrap();
  assert_eq!(backup.id(), active.id());
  let seq2 = 2;
  metrics.record_failover(seq2);
  ps_bridge.provider_changed(seq2, primary.id().clone(), backup.id().clone()).unwrap();
  pm_bridge.provider_changed(seq2, primary.id().clone(), backup.id().clone()).unwrap();

  // remoting event and block/unblock reflection
  let mut snap2 = snapshot(101, ProviderHealth::Degraded);
  let block_evt = RemoteTopologyEvent {
    seq_no:        seq2,
    provider_id:   backup.id().clone(),
    snapshot_hash: snap2.hash,
    node_id:       NodeId::new("n2"),
    kind:          RemoteTopologyKind::Blocked,
  };
  rem_bridge.publish(&block_evt).unwrap();
  let mut warnings: Vec<String> = Vec::new();
  apply_block_event(&mut snap2, &block_evt, |msg| warnings.push(msg));
  assert_eq!(vec![NodeId::new("n2")], snap2.blocked_nodes);
  assert_eq!(1, warnings.len());

  // stream interruption metric
  metrics.record_stream_interrupt(seq2);

  // termination signal retained in hub
  hub.apply_event(ProviderEvent::Terminated { reason: ProviderTermination::Errored { reason: "stream closed".to_string() } }).unwrap();
  assert!(hub.termination().is_some());

  // assertions
  assert_eq!(vec![(seq1, Duration::from_millis(8))], metrics.snapshot_latencies());
  assert_eq!(vec![seq2], metrics.failovers());
  assert_eq!(vec![seq2], metrics.interruptions());

  assert_eq!(vec![100], *ps_port.snapshots.lock().unwrap());
  assert_eq!(vec![100], *pm_port.snapshots.lock().unwrap());
  assert_eq!(vec![("p1".to_string(), "p2".to_string())], *ps_port.changes.lock().unwrap());
  assert_eq!(vec![("p1".to_string(), "p2".to_string())], *pm_port.changes.lock().unwrap());
  assert_eq!(vec![("p2".to_string(), seq2)], *rem_port.events.lock().unwrap());
}
