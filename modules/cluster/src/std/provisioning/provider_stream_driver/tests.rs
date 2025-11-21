use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::identity::{ClusterNode, NodeId};
use crate::core::provisioning::descriptor::ProviderId;
use crate::core::provisioning::snapshot::{ProviderHealth, ProviderSnapshot};
use crate::std::provisioning::partition_manager_bridge::{PartitionManagerBridge, PartitionManagerPort};
use crate::std::provisioning::placement_supervisor_bridge::{PlacementSupervisorBridge, PlacementSupervisorPort};
use crate::std::provisioning::provider_event::{ProviderEvent, ProviderTermination};
use crate::std::provisioning::provider_stream::ProviderStream;
use crate::std::provisioning::provider_stream_driver::{ProviderStreamDriver, StreamProgress};
use crate::std::provisioning::provider_watch_hub::ProviderWatchHub;
use crate::std::provisioning::provisioning_metrics::ProvisioningMetrics;

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
}

impl PlacementSupervisorPort for RecordingPlacement {
  fn apply_snapshot(&self, snapshot: &ProviderSnapshot) {
    self.snapshots.lock().unwrap().push(snapshot.hash);
  }

  fn provider_changed(&self, _: ProviderId, _: ProviderId) {}
}

#[derive(Default)]
struct RecordingPartition {
  snapshots: Mutex<Vec<u64>>,
}

impl PartitionManagerPort for RecordingPartition {
  fn apply_snapshot(&self, snapshot: &ProviderSnapshot) {
    self.snapshots.lock().unwrap().push(snapshot.hash);
  }

  fn provider_changed(&self, _: ProviderId, _: ProviderId) {}
}

#[test]
fn delivers_only_on_hash_change_and_records_metrics() {
  let stream = VecStream::new(vec![
    ProviderEvent::Snapshot(snapshot(1)),
    ProviderEvent::Snapshot(snapshot(1)), // same hash, should be skipped
    ProviderEvent::Snapshot(snapshot(2)), // new hash, should deliver
  ]);
  let hub = Arc::new(ProviderWatchHub::new());
  let placement_port = Arc::new(RecordingPlacement::default());
  let partition_port = Arc::new(RecordingPartition::default());
  let placement = Arc::new(PlacementSupervisorBridge::new(placement_port.clone()));
  let partition = Arc::new(PartitionManagerBridge::new(partition_port.clone()));
  let metrics = Arc::new(ProvisioningMetrics::new());

  let mut driver = ProviderStreamDriver::new(stream, hub.clone(), placement.clone(), partition.clone(), metrics.clone());

  assert!(matches!(driver.pump_once().unwrap(), StreamProgress::Advanced));
  assert!(matches!(driver.pump_once().unwrap(), StreamProgress::Advanced));
  assert!(matches!(driver.pump_once().unwrap(), StreamProgress::Advanced));

  // first and third snapshots delivered (hash 1 then 2)
  assert_eq!(vec![1, 2], *placement_port.snapshots.lock().unwrap());
  assert_eq!(vec![1, 2], *partition_port.snapshots.lock().unwrap());
  assert_eq!(2, metrics.snapshot_latencies().len());
}

#[test]
fn records_termination_and_interrupt_metric() {
  let stream = VecStream::new(vec![ProviderEvent::Terminated { reason: ProviderTermination::Ended }]);
  let hub = Arc::new(ProviderWatchHub::new());
  let placement = Arc::new(PlacementSupervisorBridge::new(Arc::new(RecordingPlacement::default())));
  let partition = Arc::new(PartitionManagerBridge::new(Arc::new(RecordingPartition::default())));
  let metrics = Arc::new(ProvisioningMetrics::new());

  let mut driver = ProviderStreamDriver::new(stream, hub.clone(), placement, partition, metrics.clone());

  assert!(matches!(driver.pump_once().unwrap(), StreamProgress::Terminated));

  assert!(hub.termination().is_some());
  assert_eq!(vec![0], metrics.interruptions());
}
