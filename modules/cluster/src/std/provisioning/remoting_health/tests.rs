use crate::core::identity::NodeId;
use crate::core::provisioning::descriptor::ProviderId;
use crate::std::provisioning::provider_event::{RemoteTopologyEvent, RemoteTopologyKind};
use crate::std::provisioning::remoting_health::{RemotingHealthMetrics, RemotingNodeStatus};

fn evt(kind: RemoteTopologyKind) -> RemoteTopologyEvent {
  RemoteTopologyEvent {
    seq_no:        1,
    provider_id:   ProviderId::new("p1"),
    snapshot_hash: 5,
    node_id:       NodeId::new("n42"),
    kind,
  }
}

#[test]
fn records_join_and_block_and_unblock() {
  let metrics = RemotingHealthMetrics::new();

  metrics.record_event(&evt(RemoteTopologyKind::Join));
  let status = metrics.status_of("n42").unwrap();
  assert_eq!(RemotingNodeStatus::Up, status.status);

  metrics.record_event(&evt(RemoteTopologyKind::Blocked));
  let status = metrics.status_of("n42").unwrap();
  assert_eq!(RemotingNodeStatus::Degraded, status.status);

  metrics.record_event(&evt(RemoteTopologyKind::Unblocked));
  let status = metrics.status_of("n42").unwrap();
  assert_eq!(RemotingNodeStatus::Up, status.status);
}

#[test]
fn leave_marks_down() {
  let metrics = RemotingHealthMetrics::new();
  metrics.record_event(&evt(RemoteTopologyKind::Leave));
  let status = metrics.status_of("n42").unwrap();
  assert_eq!(RemotingNodeStatus::Down, status.status);
}
