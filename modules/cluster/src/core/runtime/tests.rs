use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::string::String;
use core::any::TypeId;
use core::ptr;

use fraktor_utils_rs::core::runtime_toolbox::NoStdToolbox;

use crate::core::activation::{ActivationLedger, LeaseStatus};
use crate::core::config::{
    ClusterConfig, ClusterMetricsConfig, HashStrategy, RetryPolicy, TopologyStream, TopologyWatch,
};
use crate::core::identity::{ClusterIdentity, ClusterNode, IdentityLookupService, NodeId, TopologySnapshot};
use crate::core::metrics::ClusterMetrics;
use crate::core::runtime::{ClusterRuntime, ResolveError};

#[derive(Clone)]
struct DummyStream;

impl TopologyStream for DummyStream {
    fn stream_id(&self) -> &'static str {
        "dummy"
    }
}

struct TestMetrics;

impl ClusterMetrics for TestMetrics {
    fn as_any(&self) -> &dyn core::any::Any {
        self
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

fn sample_snapshot() -> TopologySnapshot {
    TopologySnapshot::new(
        99,
        vec![
            ClusterNode::new(NodeId::new("node-a"), "10.0.0.1", 1, false),
            ClusterNode::new(NodeId::new("node-b"), "10.0.0.2", 4, false),
        ],
    )
}

#[test]
fn runtime_retains_dependencies() {
    let config = sample_config();
    let identity = Arc::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 7));
    let activation = Arc::new(ActivationLedger::<NoStdToolbox>::new());
    let metrics: Arc<dyn ClusterMetrics> = Arc::new(TestMetrics);

    let runtime = ClusterRuntime::new(config.clone(), identity.clone(), activation.clone(), metrics.clone());

    assert_eq!(runtime.config().system_name(), config.system_name());
    assert!(ptr::eq(Arc::as_ptr(&identity), runtime.identity() as *const _));
    assert!(ptr::eq(Arc::as_ptr(&activation), runtime.activation() as *const _));
    assert_eq!(
        runtime.metrics().as_any().type_id(),
        TypeId::of::<TestMetrics>()
    );
}

#[test]
fn resolve_owner_acquires_lease() {
    let config = sample_config();
    let identity = ClusterIdentity::new("echo", "id-1");
    let requester = NodeId::new("req");

    let identity_service = Arc::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 17));
    identity_service.update_topology(sample_snapshot());
    let ledger = Arc::new(ActivationLedger::<NoStdToolbox>::new());
    let metrics: Arc<dyn ClusterMetrics> = Arc::new(TestMetrics);

    let runtime = ClusterRuntime::new(config, identity_service, ledger.clone(), metrics);

    let first = runtime
        .resolve_owner(&identity, &requester)
        .expect("first resolve");
    assert_eq!(first.owner().id().as_str(), "node-b");

    let err = runtime
        .resolve_owner(&identity, &requester)
        .expect_err("second resolve should fail");
    match err {
        ResolveError::LeaseConflict { existing } => {
            assert_eq!(existing.owner().as_str(), "node-b");
        }
        _ => panic!("unexpected error"),
    }
}

#[test]
fn handle_blocked_node_revokes_leases() {
    let config = sample_config();
    let identity_service = Arc::new(IdentityLookupService::<NoStdToolbox>::new(HashStrategy::Rendezvous, 17));
    identity_service.update_topology(sample_snapshot());
    let ledger = Arc::new(ActivationLedger::<NoStdToolbox>::new());
    let metrics: Arc<dyn ClusterMetrics> = Arc::new(TestMetrics);
    let runtime = ClusterRuntime::new(config, identity_service, ledger.clone(), metrics);

    let identity = ClusterIdentity::new("echo", "id-1");
    let requester = NodeId::new("req");
    let resolution = runtime
        .resolve_owner(&identity, &requester)
        .expect("resolution");
    let owner_id = resolution.owner().id().clone();

    let revoked = runtime.handle_blocked_node(&owner_id);

    assert_eq!(revoked.len(), 1);
    assert_eq!(revoked[0].0, identity);
    assert!(matches!(revoked[0].1.status(), LeaseStatus::Revoked));
    assert!(ledger.get(&identity).is_none());
}
