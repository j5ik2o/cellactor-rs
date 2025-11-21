use std::sync::{Arc, Mutex};

use crate::{
  core::{identity::NodeId, provisioning::descriptor::ProviderId},
  std::provisioning::{
    provider_event::{RemoteTopologyEvent, RemoteTopologyKind},
    remoting_bridge::{RemotingBridge, RemotingBridgeError},
    remoting_health::{RemotingHealthMetrics, RemotingNodeStatus},
    remoting_port::RemotingPort,
  },
};

struct RecordingPort {
  events: Mutex<Vec<(String, u64)>>,
}

impl RecordingPort {
  fn new() -> Self {
    Self { events: Mutex::new(Vec::new()) }
  }
}

impl RemotingPort for RecordingPort {
  fn publish_remote_topology(&self, event: &RemoteTopologyEvent) {
    self.events.lock().unwrap().push((event.provider_id.as_str().to_string(), event.seq_no));
  }
}

fn event(seq_no: u64) -> RemoteTopologyEvent {
  RemoteTopologyEvent {
    seq_no,
    provider_id: ProviderId::new("p1"),
    snapshot_hash: 42,
    node_id: NodeId::new("n1"),
    kind: RemoteTopologyKind::Join,
  }
}

#[test]
fn publishes_and_deduplicates_by_idempotency_key() {
  let port = Arc::new(RecordingPort::new());
  let bridge = RemotingBridge::new(port.clone());

  bridge.publish(&event(1)).unwrap();
  let err = bridge.publish(&event(1)).unwrap_err();

  assert!(matches!(err, RemotingBridgeError::Duplicate { provider, seq_no } if provider == "p1" && seq_no == 1));
  assert_eq!(vec![("p1".to_string(), 1)], *port.events.lock().unwrap());
}

#[test]
fn allows_distinct_seq_for_same_provider() {
  let port = Arc::new(RecordingPort::new());
  let bridge = RemotingBridge::new(port.clone());

  bridge.publish(&event(1)).unwrap();
  bridge.publish(&event(2)).unwrap();

  assert_eq!(vec![("p1".to_string(), 1), ("p1".to_string(), 2)], *port.events.lock().unwrap());
}

#[test]
fn updates_health_metrics_on_publish() {
  let port = Arc::new(RecordingPort::new());
  let health = Arc::new(RemotingHealthMetrics::new());
  let bridge = RemotingBridge::with_health(port.clone(), health.clone());

  bridge.publish(&event(1)).unwrap();

  let entry = health.status_of("n1").unwrap();
  assert_eq!(RemotingNodeStatus::Up, entry.status);
}
