use crate::core::identity::{ClusterNode, NodeId};
use crate::core::provisioning::snapshot::{ProviderHealth, ProviderSnapshot};
use crate::std::provisioning::provider_event::{ProviderEvent, ProviderTermination};
use crate::std::provisioning::provider_watch_hub::ProviderWatchHub;

fn sample_snapshot(hash: u64) -> ProviderSnapshot {
  ProviderSnapshot {
    members: vec![ClusterNode::new(NodeId::new("n1"), "127.0.0.1", 1, false)],
    hash,
    blocked_nodes: vec![],
    health: ProviderHealth::Healthy,
  }
}

#[test]
fn stores_latest_snapshot() {
  let hub = ProviderWatchHub::new();

  hub.apply_event(ProviderEvent::Snapshot(sample_snapshot(1)));
  hub.apply_event(ProviderEvent::Snapshot(sample_snapshot(2)));

  let latest = hub.latest_snapshot().unwrap();
  assert_eq!(latest.hash, 2);
}

#[test]
fn records_termination_reason() {
  let hub = ProviderWatchHub::new();

  hub.apply_event(ProviderEvent::Terminated { reason: ProviderTermination::Errored { reason: "failed".to_string() } });

  let reason = hub.termination().unwrap();
  assert!(matches!(reason, ProviderTermination::Errored { .. }));
}
