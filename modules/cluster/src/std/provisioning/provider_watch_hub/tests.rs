use crate::{
  core::{
    identity::{ClusterNode, NodeId},
    provisioning::snapshot::{ProviderHealth, ProviderSnapshot},
  },
  std::provisioning::{
    provider_event::{ProviderEvent, ProviderTermination},
    provider_watch_hub::ProviderWatchHub,
  },
};

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

  hub.apply_event(ProviderEvent::Snapshot(sample_snapshot(1))).unwrap();
  hub.apply_event(ProviderEvent::Snapshot(sample_snapshot(2))).unwrap();

  let latest = hub.latest_snapshot().unwrap();
  assert_eq!(latest.hash, 2);
}

#[test]
fn reports_invalidation_on_hash_change() {
  let hub = ProviderWatchHub::new();

  hub.apply_event(ProviderEvent::Snapshot(sample_snapshot(10))).unwrap();
  let (_, invalid) = hub.latest_snapshot_with_invalidation().unwrap();
  assert!(!invalid, "initial snapshot should not mark invalidation");

  hub.apply_event(ProviderEvent::Snapshot(sample_snapshot(11))).unwrap();
  let (_, invalid2) = hub.latest_snapshot_with_invalidation().unwrap();
  assert!(invalid2, "hash change should mark invalidation");

  // same hash again resets invalidation
  hub.apply_event(ProviderEvent::Snapshot(sample_snapshot(11))).unwrap();
  let (_, invalid3) = hub.latest_snapshot_with_invalidation().unwrap();
  assert!(!invalid3, "same hash should be treated as cache hit");
}

#[test]
fn records_termination_reason() {
  let hub = ProviderWatchHub::new();

  hub
    .apply_event(ProviderEvent::Terminated { reason: ProviderTermination::Errored { reason: "failed".to_string() } })
    .unwrap();

  let reason = hub.termination().unwrap();
  assert!(matches!(reason, ProviderTermination::Errored { .. }));
}

#[test]
fn rejects_after_shutdown() {
  let hub = ProviderWatchHub::new();
  hub.begin_shutdown();
  let err = hub.apply_event(ProviderEvent::Snapshot(sample_snapshot(3))).unwrap_err();
  assert!(matches!(err, crate::std::provisioning::provider_watch_hub::WatchError::ShuttingDown));
}
