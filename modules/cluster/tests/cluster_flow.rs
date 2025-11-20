use std::boxed::Box;
use std::string::String;
use std::sync::{Arc, Mutex};
use std::vec::Vec;

use fraktor_actor_rs::core::actor_prim::{Actor, ActorContextGeneric};
use fraktor_actor_rs::core::messaging::AnyMessageViewGeneric;
use fraktor_actor_rs::core::props::PropsGeneric;
use fraktor_actor_rs::core::error::ActorError;
use fraktor_utils_rs::core::runtime_toolbox::NoStdToolbox;

use fraktor_cluster_rs::core::activation::{ActivationLedger, ActivationRequest, ActivationResponse, PartitionBridge, PartitionBridgeError};
use fraktor_cluster_rs::core::config::{ClusterConfig, ClusterMetricsConfig, HashStrategy, RetryPolicy, TopologyStream, TopologyWatch};
use fraktor_cluster_rs::core::identity::{ClusterIdentity, ClusterNode, IdentityLookupService, NodeId, TopologySnapshot};
use fraktor_cluster_rs::core::metrics::ClusterMetrics;
use fraktor_cluster_rs::core::routing::PidCache;
use fraktor_cluster_rs::core::runtime::ClusterRuntime;

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

    let identity_service = Arc::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 17));
    identity_service.update_topology(sample_snapshot());
    let ledger = Arc::new(ActivationLedger::<NoStdToolbox>::new());
    let metrics_impl = Arc::new(MockMetrics::default());
    let metrics: Arc<dyn ClusterMetrics> = metrics_impl.clone();
    let bridge = Arc::new(MockBridge::default());
    let pid_cache = Arc::new(PidCache::new());

    let runtime = ClusterRuntime::new(config, identity_service, ledger.clone(), metrics, bridge.clone(), pid_cache.clone());

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

    runtime.begin_shutdown();
    assert_eq!(metrics_impl.gauge(), 0);
    assert!(pid_cache.get(&identity).is_none());
}
fn sample_watch() -> TopologyWatch {
    TopologyWatch::new(Box::new(DummyStream))
}

fn sample_snapshot() -> TopologySnapshot {
    TopologySnapshot::new(
        99,
        vec![
            ClusterNode::new(NodeId::new("node-a"), "10.0.0.1", 1, false),
            ClusterNode::new(NodeId::new("node-b"), "10.0.0.2", 4, false),
        ],
    )
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
    resolves: Mutex<u32>,
    block_lists: Mutex<u32>,
    gauge: Mutex<usize>,
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
    requests: Mutex<Vec<ActivationRequest<NoStdToolbox>>>,
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
