#![allow(clippy::disallowed_types)]
use std::{boxed::Box, string::String, sync::Mutex, vec::Vec};

use fraktor_actor_rs::core::{
  actor_prim::{Actor, ActorContextGeneric, Pid},
  error::ActorError,
  messaging::AnyMessageViewGeneric,
  props::PropsGeneric,
};
use fraktor_cluster_rs::core::{
  activation::{ActivationLedger, ActivationRequest, ActivationResponse, PartitionBridge, PartitionBridgeError},
  config::{
    ClusterConfig, ClusterMetricsConfig, HashStrategy, RetryPolicy, TopologyWatch, topology_stream::TopologyStream,
  },
  events::ClusterEventPublisher,
  identity::{ClusterIdentity, ClusterNode, IdentityLookupService, NodeId, TopologySnapshot},
  metrics::ClusterMetrics,
  routing::{PidCache, PidCacheEntry},
  runtime::ClusterRuntime,
};
use fraktor_utils_rs::core::{runtime_toolbox::NoStdToolbox, sync::ArcShared};

#[test]
fn activation_request_shutdown_flow() {
  let config = ClusterConfig::builder()
    .system_name("cluster")
    .hash_strategy(HashStrategy::Rendezvous)
    .retry_policy(RetryPolicy::default())
    .topology_watch(sample_watch())
    .metrics_config(ClusterMetricsConfig::new(String::from("ns"), true))
    .build()
    .expect("config");

  let identity_service = ArcShared::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 17));
  identity_service.update_topology(&sample_snapshot());
  let ledger = ArcShared::new(ActivationLedger::<NoStdToolbox>::new());
  let metrics_impl = ArcShared::new(MockMetrics::default());
  let metrics: ArcShared<dyn ClusterMetrics> = metrics_impl.clone();
  let bridge = ArcShared::new(MockBridge::default());
  let pid_cache = ArcShared::new(PidCache::new());
  let events = ArcShared::new(ClusterEventPublisher::new());

  let runtime = ClusterRuntime::new(
    config,
    identity_service.clone(),
    ledger.clone(),
    metrics,
    bridge.clone(),
    pid_cache.clone(),
    events,
  );

  let identity = ClusterIdentity::new("echo", "integration");
  let requester = NodeId::new("req");
  let resolution = runtime.resolve_owner(&identity, &requester).expect("resolution");
  assert_eq!(metrics_impl.resolve_count(), 1);

  let request = ActivationRequest::new(
    identity.clone(),
    resolution.lease().clone(),
    PropsGeneric::<NoStdToolbox>::from_fn(|| DummyActor),
  );
  runtime.dispatch_activation_request(request).expect("dispatch");
  assert_eq!(bridge.requests.lock().unwrap().len(), 1);

  let owner_id = resolution.lease().owner().clone();
  runtime.handle_blocked_node(&owner_id);
  assert_eq!(metrics_impl.block_list_count(), 1);

  // BlockList 後に新しい PID が再解決されたと仮定し、古いトポロジハッシュでキャッシュを保持
  pid_cache.insert(
    identity.clone(),
    PidCacheEntry::new(Pid::new(9, 0), NodeId::new("node-b"), resolution.lease().topology_hash()),
  );

  // トポロジ変更でキャッシュが無効化されることを検証
  identity_service.update_topology(&TopologySnapshot::new(100, vec![ClusterNode::new(
    NodeId::new("node-c"),
    "10.0.0.3",
    1,
    false,
  )]));
  let removed = runtime.handle_topology_changed();
  assert!(!removed.is_empty());

  runtime.begin_shutdown();
  assert_eq!(metrics_impl.gauge(), 0);
  assert!(pid_cache.get(&identity).is_none());
}
fn sample_watch() -> TopologyWatch {
  TopologyWatch::new(Box::new(DummyStream))
}

fn sample_snapshot() -> TopologySnapshot {
  TopologySnapshot::new(99, vec![
    ClusterNode::new(NodeId::new("node-a"), "10.0.0.1", 1, false),
    ClusterNode::new(NodeId::new("node-b"), "10.0.0.2", 4, false),
  ])
}

#[derive(Clone)]
struct DummyStream;

impl TopologyStream for DummyStream {
  fn stream_id(&self) -> &'static str {
    "dummy"
  }
}

#[derive(Default)]
struct MockMetrics {
  resolves:    Mutex<u32>,
  block_lists: Mutex<u32>,
  gauge:       Mutex<usize>,
}

impl MockMetrics {
  fn resolve_count(&self) -> u32 {
    *self.resolves.lock().unwrap()
  }

  fn block_list_count(&self) -> u32 {
    *self.block_lists.lock().unwrap()
  }

  fn gauge(&self) -> usize {
    *self.gauge.lock().unwrap()
  }
}

impl ClusterMetrics for MockMetrics {
  fn as_any(&self) -> &dyn core::any::Any {
    self
  }

  fn record_resolve_duration(&self, _identity: &ClusterIdentity, _duration: core::time::Duration) {
    *self.resolves.lock().unwrap() += 1;
  }

  fn increment_block_list(&self, _node: &NodeId) {
    *self.block_lists.lock().unwrap() += 1;
  }

  fn set_virtual_actor_gauge(&self, value: usize) {
    *self.gauge.lock().unwrap() = value;
  }
}

#[derive(Default)]
struct MockBridge {
  requests:  Mutex<Vec<ActivationRequest<NoStdToolbox>>>,
  responses: Mutex<Vec<ActivationResponse>>,
}

impl PartitionBridge<NoStdToolbox> for MockBridge {
  fn send_activation_request(&self, request: ActivationRequest<NoStdToolbox>) -> Result<(), PartitionBridgeError> {
    self.requests.lock().unwrap().push(request);
    Ok(())
  }

  fn handle_activation_response(&self, response: ActivationResponse) {
    self.responses.lock().unwrap().push(response);
  }
}

struct DummyActor;

impl Actor<NoStdToolbox> for DummyActor {
  fn receive(
    &mut self,
    _ctx: &mut ActorContextGeneric<'_, NoStdToolbox>,
    _message: AnyMessageViewGeneric<'_, NoStdToolbox>,
  ) -> Result<(), ActorError> {
    Ok(())
  }
}
