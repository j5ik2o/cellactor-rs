use alloc::{boxed::Box, string::String, vec::Vec};
use core::{any::TypeId, ptr};
use std::sync::Mutex;

use fraktor_actor_rs::core::{
  actor_prim::{Actor, ActorContextGeneric, Pid},
  error::ActorError,
  messaging::AnyMessageViewGeneric,
  props::PropsGeneric,
};
use fraktor_utils_rs::core::{runtime_toolbox::NoStdToolbox, sync::ArcShared};

use crate::core::{
  activation::{
    ActivationError, ActivationLease, ActivationLedger, ActivationRequest, ActivationResponse, LeaseId, LeaseStatus,
    PartitionBridge, PartitionBridgeError,
  },
  config::{
    ClusterConfig, ClusterMetricsConfig, HashStrategy, RetryPolicy, TopologyWatch, topology_stream::TopologyStream,
  },
  events::{ClusterEvent, ClusterEventPublisher},
  identity::{ClusterIdentity, ClusterNode, IdentityLookupService, NodeId, TopologySnapshot},
  metrics::ClusterMetrics,
  routing::{PidCache, PidCacheEntry},
  runtime::{ClusterRuntime, ResolveError},
};

#[derive(Clone)]
struct DummyStream;

impl TopologyStream for DummyStream {
  fn stream_id(&self) -> &'static str {
    "dummy"
  }
}

#[derive(Default)]
struct TestMetrics {
  resolves:    Mutex<u32>,
  retries:     Mutex<u32>,
  timeouts:    Mutex<u32>,
  block_lists: Mutex<u32>,
  gauge:       Mutex<usize>,
}

impl TestMetrics {
  fn gauge(&self) -> usize {
    *self.gauge.lock().unwrap()
  }

  fn block_list_count(&self) -> u32 {
    *self.block_lists.lock().unwrap()
  }

  fn resolve_count(&self) -> u32 {
    *self.resolves.lock().unwrap()
  }

  fn timeout_count(&self) -> u32 {
    *self.timeouts.lock().unwrap()
  }
}

impl ClusterMetrics for TestMetrics {
  fn as_any(&self) -> &dyn core::any::Any {
    self
  }

  fn record_resolve_duration(&self, _identity: &ClusterIdentity, _duration: core::time::Duration) {
    *self.resolves.lock().unwrap() += 1;
  }

  fn record_request_duration(&self, _identity: &ClusterIdentity, _duration: core::time::Duration) {
    // ignore for tests
  }

  fn record_retry_attempt(&self, _identity: &ClusterIdentity) {
    *self.retries.lock().unwrap() += 1;
  }

  fn record_timeout(&self, _identity: &ClusterIdentity) {
    *self.timeouts.lock().unwrap() += 1;
  }

  fn set_virtual_actor_gauge(&self, value: usize) {
    *self.gauge.lock().unwrap() = value;
  }

  fn increment_block_list(&self, _node: &NodeId) {
    *self.block_lists.lock().unwrap() += 1;
  }
}

fn sample_watch() -> TopologyWatch {
  TopologyWatch::new(Box::new(DummyStream))
}

fn sample_config() -> ClusterConfig {
  ClusterConfig::builder()
    .system_name("cluster")
    .hash_strategy(HashStrategy::Rendezvous)
    .retry_policy(RetryPolicy::default())
    .topology_watch(sample_watch())
    .metrics_config(ClusterMetricsConfig::new(String::from("ns"), true))
    .build()
    .expect("config build")
}

#[test]
fn runtime_retains_dependencies() {
  let config = sample_config();
  let identity = ArcShared::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 7));
  let activation = ArcShared::new(ActivationLedger::<NoStdToolbox>::new());
  let metrics_impl = ArcShared::new(TestMetrics::default());
  let metrics: ArcShared<dyn ClusterMetrics> = metrics_impl.clone();

  let bridge = ArcShared::new(TestBridge::default());
  let pid_cache = ArcShared::new(PidCache::new());
  let events = ArcShared::new(ClusterEventPublisher::new());
  let runtime = ClusterRuntime::new(
    config.clone(),
    identity.clone(),
    activation.clone(),
    metrics.clone(),
    bridge,
    pid_cache,
    events,
  );

  assert_eq!(runtime.config().system_name(), config.system_name());
  assert!(ptr::eq(&*identity as *const _, runtime.identity() as *const _));
  assert!(ptr::eq(&*activation as *const _, runtime.activation() as *const _));
  assert_eq!(runtime.metrics().as_any().type_id(), TypeId::of::<TestMetrics>());
}

#[test]
fn resolve_owner_acquires_lease() {
  let config = sample_config();
  let identity = ClusterIdentity::new("echo", "id-1");
  let requester = NodeId::new("req");

  let identity_service = ArcShared::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 17));
  identity_service.update_topology(&sample_snapshot());
  let ledger = ArcShared::new(ActivationLedger::<NoStdToolbox>::new());
  let metrics_impl = ArcShared::new(TestMetrics::default());
  let metrics: ArcShared<dyn ClusterMetrics> = metrics_impl.clone();

  let bridge = ArcShared::new(TestBridge::default());
  let pid_cache = ArcShared::new(PidCache::new());
  let events = ArcShared::new(ClusterEventPublisher::new());
  let runtime = ClusterRuntime::new(config, identity_service, ledger.clone(), metrics, bridge, pid_cache, events);

  let first = runtime.resolve_owner(&identity, &requester).expect("first resolve");
  assert_eq!(first.owner().id().as_str(), "node-b");
  assert_eq!(metrics_impl.resolve_count(), 1);
  assert_eq!(metrics_impl.gauge(), 1);

  let err = runtime.resolve_owner(&identity, &requester).expect_err("second resolve should fail");
  match err {
    | ResolveError::LeaseConflict { existing } => {
      assert_eq!(existing.owner().as_str(), "node-b");
    },
    | _ => panic!("unexpected error"),
  }
}

#[test]
fn handle_blocked_node_revokes_leases() {
  let config = sample_config();
  let identity_service = ArcShared::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 17));
  identity_service.update_topology(&sample_snapshot());
  let ledger = ArcShared::new(ActivationLedger::<NoStdToolbox>::new());
  let metrics_impl = ArcShared::new(TestMetrics::default());
  let metrics: ArcShared<dyn ClusterMetrics> = metrics_impl.clone();
  let bridge = ArcShared::new(TestBridge::default());
  let pid_cache = ArcShared::new(PidCache::new());
  let events = ArcShared::new(ClusterEventPublisher::new());
  let runtime =
    ClusterRuntime::new(config, identity_service, ledger.clone(), metrics, bridge, pid_cache.clone(), events);

  let identity = ClusterIdentity::new("echo", "id-1");
  let requester = NodeId::new("req");
  pid_cache.insert(identity.clone(), PidCacheEntry::new(Pid::new(9, 0), NodeId::new("node-b"), 99));
  let resolution = runtime.resolve_owner(&identity, &requester).expect("resolution");
  let owner_id = resolution.owner().id().clone();

  let revoked = runtime.handle_blocked_node(&owner_id);

  assert_eq!(revoked.len(), 1);
  assert_eq!(revoked[0].0, identity);
  assert!(matches!(revoked[0].1.status(), LeaseStatus::Revoked));
  assert!(ledger.get(&identity).is_none());
  assert_eq!(metrics_impl.block_list_count(), 1);
  assert_eq!(metrics_impl.gauge(), 0);
  assert!(pid_cache.get(&identity).is_none());
}

#[test]
fn blocked_node_enqueues_blocklist_event() {
  let config = sample_config();
  let identity_service = ArcShared::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 17));
  identity_service.update_topology(&sample_snapshot());
  let ledger = ArcShared::new(ActivationLedger::<NoStdToolbox>::new());
  let metrics_impl = ArcShared::new(TestMetrics::default());
  let metrics: ArcShared<dyn ClusterMetrics> = metrics_impl.clone();
  let bridge = ArcShared::new(TestBridge::default());
  let pid_cache = ArcShared::new(PidCache::new());
  let events = ArcShared::new(ClusterEventPublisher::new());
  let runtime = ClusterRuntime::new(config, identity_service, ledger, metrics, bridge, pid_cache, events.clone());

  let node = NodeId::new("node-a");

  let _ = runtime.handle_blocked_node(&node);

  let drained = events.drain();
  assert!(drained.iter().any(|event| matches!(event, ClusterEvent::BlockListApplied { node: n } if n == &node)));
}

#[test]
fn begin_shutdown_prevents_new_resolves() {
  let config = sample_config();
  let identity_service = ArcShared::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 17));
  identity_service.update_topology(&sample_snapshot());
  let ledger = ArcShared::new(ActivationLedger::<NoStdToolbox>::new());
  let metrics_impl = ArcShared::new(TestMetrics::default());
  let metrics: ArcShared<dyn ClusterMetrics> = metrics_impl.clone();
  let bridge = ArcShared::new(TestBridge::default());
  let pid_cache = ArcShared::new(PidCache::new());
  let events = ArcShared::new(ClusterEventPublisher::new());
  let runtime =
    ClusterRuntime::new(config, identity_service, ledger.clone(), metrics, bridge, pid_cache.clone(), events);

  let identity = ClusterIdentity::new("echo", "a");
  let requester = NodeId::new("req");
  pid_cache.insert(identity.clone(), PidCacheEntry::new(Pid::new(1, 0), NodeId::new("node-a"), 1));

  runtime.begin_shutdown();

  let err = runtime.resolve_owner(&identity, &requester).expect_err("shutdown should block resolves");
  assert!(matches!(err, ResolveError::ShuttingDown));
  assert!(ledger.release_all().is_empty());
  assert_eq!(metrics_impl.gauge(), 0);
  assert!(pid_cache.get(&identity).is_none());
}

#[test]
fn topology_changed_keeps_cache_when_hash_unchanged() {
  let config = sample_config();
  let identity_service = ArcShared::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 17));
  let snapshot = sample_snapshot();
  identity_service.update_topology(&snapshot);
  let ledger = ArcShared::new(ActivationLedger::<NoStdToolbox>::new());
  let metrics_impl = ArcShared::new(TestMetrics::default());
  let metrics: ArcShared<dyn ClusterMetrics> = metrics_impl.clone();
  let bridge = ArcShared::new(TestBridge::default());
  let pid_cache = ArcShared::new(PidCache::new());
  let events = ArcShared::new(ClusterEventPublisher::new());
  let runtime = ClusterRuntime::new(config, identity_service, ledger, metrics, bridge, pid_cache.clone(), events);

  let identity = ClusterIdentity::new("echo", "keep");
  pid_cache.insert(identity.clone(), PidCacheEntry::new(Pid::new(1, 0), NodeId::new("node-a"), snapshot.hash()));

  let removed = runtime.handle_topology_changed();

  assert!(removed.is_empty());
  assert!(pid_cache.get(&identity).is_some());
}

#[test]
fn topology_changed_invalidates_cache_on_hash_difference() {
  let config = sample_config();
  let identity_service = ArcShared::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 17));
  identity_service.update_topology(&sample_snapshot());
  let ledger = ArcShared::new(ActivationLedger::<NoStdToolbox>::new());
  let metrics_impl = ArcShared::new(TestMetrics::default());
  let metrics: ArcShared<dyn ClusterMetrics> = metrics_impl.clone();
  let bridge = ArcShared::new(TestBridge::default());
  let pid_cache = ArcShared::new(PidCache::new());
  let events = ArcShared::new(ClusterEventPublisher::new());
  let runtime = ClusterRuntime::new(config, identity_service, ledger, metrics, bridge, pid_cache.clone(), events);

  let identity = ClusterIdentity::new("echo", "drop");
  pid_cache.insert(identity.clone(), PidCacheEntry::new(Pid::new(1, 0), NodeId::new("node-a"), 1));

  // update topology with new hash
  runtime.identity().update_topology(&TopologySnapshot::new(2, vec![ClusterNode::new(
    NodeId::new("node-a"),
    "10.0.0.1",
    1,
    false,
  )]));

  let removed = runtime.handle_topology_changed();

  assert_eq!(removed, alloc::vec![identity.clone()]);
  assert!(pid_cache.get(&identity).is_none());
}

#[test]
fn dispatches_activation_request_via_bridge() {
  let config = sample_config();
  let identity_service = ArcShared::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 17));
  identity_service.update_topology(&sample_snapshot());
  let ledger = ArcShared::new(ActivationLedger::<NoStdToolbox>::new());
  let metrics_impl = ArcShared::new(TestMetrics::default());
  let metrics: ArcShared<dyn ClusterMetrics> = metrics_impl.clone();
  let bridge = ArcShared::new(TestBridge::default());
  let pid_cache = ArcShared::new(PidCache::new());
  let events = ArcShared::new(ClusterEventPublisher::new());
  let runtime =
    ClusterRuntime::new(config, identity_service, ledger, metrics, bridge.clone(), pid_cache, events.clone());

  let identity = ClusterIdentity::new("echo", "req");
  let lease = ActivationLease::new(LeaseId::new(1), NodeId::new("node-a"), 2, LeaseStatus::Active);
  let props = PropsGeneric::<NoStdToolbox>::from_fn(|| TestActor);
  let request = ActivationRequest::new(identity.clone(), lease, props);

  runtime.dispatch_activation_request(request).expect("bridge send");

  let recorded = bridge.requests.lock().unwrap();
  assert_eq!(recorded.len(), 1);
  assert_eq!(recorded[0], identity);

  let drained = events.drain();
  assert!(drained.iter().any(|event| matches!(event,
    ClusterEvent::ActivationStarted { identity: ev_id, owner }
    if ev_id == &ClusterIdentity::new("echo", "req") && owner == &NodeId::new("node-a")
  )));
}

#[test]
fn activation_failure_releases_lease_and_emits_event() {
  let identity_service = ArcShared::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 17));
  identity_service.update_topology(&sample_snapshot());
  let ledger = ArcShared::new(ActivationLedger::<NoStdToolbox>::new());
  let metrics_impl = ArcShared::new(TestMetrics::default());
  let metrics: ArcShared<dyn ClusterMetrics> = metrics_impl.clone();
  let bridge = ArcShared::new(TestBridge::default());
  let pid_cache = ArcShared::new(PidCache::new());
  let events = ArcShared::new(ClusterEventPublisher::new());
  let runtime = ClusterRuntime::new(
    sample_config(),
    identity_service.clone(),
    ledger.clone(),
    metrics,
    bridge,
    pid_cache,
    events.clone(),
  );

  let identity = ClusterIdentity::new("echo", "fail");
  let requester = NodeId::new("req");
  let resolution = runtime.resolve_owner(&identity, &requester).expect("acquire lease");

  let response = ActivationResponse::failure(
    identity.clone(),
    ActivationError::SpawnFailed,
    resolution.lease().lease_id(),
    resolution.lease().topology_hash(),
  );

  runtime.handle_activation_response(response).expect("handle failure");

  assert!(ledger.get(&identity).is_none());
  assert!(runtime.pid_cache().get(&identity).is_none());
  let drained = events.drain();
  assert!(
    drained
      .iter()
      .any(|event| matches!(event, ClusterEvent::ActivationFailed { identity: ev_id, .. } if ev_id == &identity))
  );
}

#[test]
fn activation_terminated_clears_cache_and_emits_event() {
  let identity_service = ArcShared::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 17));
  identity_service.update_topology(&sample_snapshot());
  let ledger = ArcShared::new(ActivationLedger::<NoStdToolbox>::new());
  let metrics_impl = ArcShared::new(TestMetrics::default());
  let metrics: ArcShared<dyn ClusterMetrics> = metrics_impl.clone();
  let bridge = ArcShared::new(TestBridge::default());
  let pid_cache = ArcShared::new(PidCache::new());
  let events = ArcShared::new(ClusterEventPublisher::new());
  let runtime = ClusterRuntime::new(
    sample_config(),
    identity_service.clone(),
    ledger.clone(),
    metrics,
    bridge,
    pid_cache.clone(),
    events.clone(),
  );

  let identity = ClusterIdentity::new("echo", "term");
  let requester = NodeId::new("req");
  let resolution = runtime.resolve_owner(&identity, &requester).expect("acquire lease");
  pid_cache.insert(
    identity.clone(),
    PidCacheEntry::new(Pid::new(99, 0), resolution.owner().id().clone(), resolution.lease().topology_hash()),
  );

  runtime.handle_activation_terminated(&identity, resolution.lease().lease_id()).expect("handle terminated");

  assert!(ledger.get(&identity).is_none());
  assert!(pid_cache.get(&identity).is_none());
  let drained = events.drain();
  assert!(
    drained
      .iter()
      .any(|event| matches!(event, ClusterEvent::ActivationTerminated { identity: ev_id, .. } if ev_id == &identity))
  );
}

fn sample_snapshot() -> TopologySnapshot {
  TopologySnapshot::new(99, vec![
    ClusterNode::new(NodeId::new("node-a"), "10.0.0.1", 1, false),
    ClusterNode::new(NodeId::new("node-b"), "10.0.0.2", 4, false),
  ])
}

#[derive(Default)]
struct TestBridge {
  requests:  Mutex<Vec<ClusterIdentity>>,
  responses: Mutex<Vec<ActivationResponse>>,
}

impl PartitionBridge<NoStdToolbox> for TestBridge {
  fn send_activation_request(&self, request: ActivationRequest<NoStdToolbox>) -> Result<(), PartitionBridgeError> {
    self.requests.lock().unwrap().push(request.identity().clone());
    Ok(())
  }

  fn handle_activation_response(&self, response: ActivationResponse) {
    self.responses.lock().unwrap().push(response);
  }
}

struct TestActor;

impl Actor<NoStdToolbox> for TestActor {
  fn receive(
    &mut self,
    _ctx: &mut ActorContextGeneric<'_, NoStdToolbox>,
    _message: AnyMessageViewGeneric<'_, NoStdToolbox>,
  ) -> Result<(), ActorError> {
    Ok(())
  }
}
