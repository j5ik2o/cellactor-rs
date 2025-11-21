use std::{
  sync::{Arc, Mutex},
  time::Duration,
};

use crate::{
  core::{
    identity::{ClusterNode, NodeId},
    provisioning::{
      descriptor::{ProviderDescriptor, ProviderId, ProviderKind},
      snapshot::{ProviderHealth, ProviderSnapshot},
    },
  },
  std::provisioning::{
    failover_controller::{FailoverConfig, FailoverController},
    partition_manager_bridge::{PartitionManagerBridge, PartitionManagerPort},
    placement_supervisor_bridge::{PlacementSupervisorBridge, PlacementSupervisorPort},
    provider_event::{ProviderEvent, ProviderTermination},
    provider_stream::ProviderStream,
    provider_stream_runner::{ProviderStreamRunner, RunnerProgress},
    provider_watch_hub::ProviderWatchHub,
    provisioning_metrics::ProvisioningMetrics,
  },
};

fn snapshot(hash: u64) -> ProviderSnapshot {
  ProviderSnapshot {
    members: vec![ClusterNode::new(NodeId::new("n1"), "127.0.0.1", 1, false)],
    hash,
    blocked_nodes: vec![],
    health: ProviderHealth::Healthy,
  }
}

struct VecStream {
  events: Vec<ProviderEvent>,
  pos:    usize,
}

impl VecStream {
  fn new(events: Vec<ProviderEvent>) -> Self {
    Self { events, pos: 0 }
  }
}

impl ProviderStream for VecStream {
  fn next_event(&mut self) -> Option<ProviderEvent> {
    if self.pos >= self.events.len() {
      return None;
    }
    let ev = self.events[self.pos].clone();
    self.pos += 1;
    Some(ev)
  }
}

#[derive(Default)]
struct RecordingPlacement {
  snapshots: Mutex<Vec<u64>>,
  changes:   Mutex<Vec<(String, String)>>,
}

impl PlacementSupervisorPort for RecordingPlacement {
  fn apply_snapshot(&self, snapshot: &ProviderSnapshot) {
    self.snapshots.lock().unwrap().push(snapshot.hash);
  }

  fn provider_changed(&self, from: ProviderId, to: ProviderId) {
    self.changes.lock().unwrap().push((from.as_str().to_string(), to.as_str().to_string()));
  }
}

#[derive(Default)]
struct RecordingPartition {
  snapshots: Mutex<Vec<u64>>,
  changes:   Mutex<Vec<(String, String)>>,
}

impl PartitionManagerPort for RecordingPartition {
  fn apply_snapshot(&self, snapshot: &ProviderSnapshot) {
    self.snapshots.lock().unwrap().push(snapshot.hash);
  }

  fn provider_changed(&self, from: ProviderId, to: ProviderId) {
    self.changes.lock().unwrap().push((from.as_str().to_string(), to.as_str().to_string()));
  }
}

fn desc(id: &str, prio: u8) -> ProviderDescriptor {
  ProviderDescriptor::new(ProviderId::new(id), ProviderKind::InMemory, prio)
}

#[test]
fn failover_switches_to_backup_and_continues_delivery() {
  let primary_stream = Box::new(VecStream::new(vec![
    ProviderEvent::Snapshot(snapshot(1)),
    ProviderEvent::Snapshot(snapshot(1)), // same hash skip
    ProviderEvent::Terminated { reason: ProviderTermination::Errored { reason: "gone".to_string() } },
  ]));
  let backup_stream = Box::new(VecStream::new(vec![ProviderEvent::Snapshot(snapshot(2))]));

  let mut cfg = FailoverConfig::default();
  cfg.max_errors = 1;
  let failover = FailoverController::new(vec![desc("primary", 10), desc("backup", 5)], cfg);

  let hub = Arc::new(ProviderWatchHub::new());
  let placement_port = Arc::new(RecordingPlacement::default());
  let partition_port = Arc::new(RecordingPartition::default());
  let placement = Arc::new(PlacementSupervisorBridge::new(placement_port.clone()));
  let partition = Arc::new(PartitionManagerBridge::new(partition_port.clone()));
  let metrics = Arc::new(ProvisioningMetrics::new());

  let mut runner = ProviderStreamRunner::new(
    failover,
    vec![(desc("primary", 10), primary_stream), (desc("backup", 5), backup_stream)],
    hub.clone(),
    placement.clone(),
    partition.clone(),
    metrics.clone(),
    Duration::from_secs(5),
  );

  // primary delivers hash 1 then terminates -> switch to backup -> deliver hash 2
  assert!(matches!(runner.pump_once(), RunnerProgress::Advanced));
  assert!(matches!(runner.pump_once(), RunnerProgress::Advanced));
  assert!(matches!(runner.pump_once(), RunnerProgress::Switched));
  assert!(matches!(runner.pump_once(), RunnerProgress::Advanced));

  assert_eq!(vec![1, 2], *placement_port.snapshots.lock().unwrap());
  assert_eq!(vec![1, 2], *partition_port.snapshots.lock().unwrap());
  assert_eq!(vec![("primary".to_string(), "backup".to_string())], *placement_port.changes.lock().unwrap());
  assert_eq!(vec![("primary".to_string(), "backup".to_string())], *partition_port.changes.lock().unwrap());

  // termination retained for primary
  assert!(hub.termination().is_some());
  assert_eq!(1, metrics.failovers().len());
}
