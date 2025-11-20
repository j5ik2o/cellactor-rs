use alloc::sync::Arc;

use fraktor_actor_rs::core::actor_prim::Pid;
use fraktor_utils_rs::core::runtime_toolbox::NoStdToolbox;
use std::sync::Mutex;

use crate::core::config::{RetryJitter, RetryPolicy};
use crate::core::identity::{ClusterIdentity, NodeId};
use crate::core::metrics::ClusterMetrics;
use crate::core::routing::{ClusterContext, ClusterError, PidCache, PidCacheEntry};
use crate::core::routing::cluster_context::ResolveBridge;

struct MockRuntime {
    results: Mutex<Vec<Result<PidCacheEntry, ClusterError>>>,
    calls: Mutex<u32>,
}

impl MockRuntime {
    fn new(results: Vec<Result<PidCacheEntry, ClusterError>>) -> Self {
        Self { results: Mutex::new(results), calls: Mutex::new(0) }
    }
}

impl ResolveBridge<NoStdToolbox> for MockRuntime {
    fn resolve(&self, _: &ClusterIdentity, _: &NodeId) -> Result<PidCacheEntry, ClusterError> {
        *self.calls.lock().unwrap() += 1;
        self.results.lock().unwrap().remove(0)
    }
}

#[derive(Default)]
struct MockMetrics {
    retries: Mutex<u32>,
}

impl MockMetrics {
    fn retry_count(&self) -> u32 {
        *self.retries.lock().unwrap()
    }
}

impl ClusterMetrics for MockMetrics {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn record_retry(&self, _identity: &ClusterIdentity) {
        *self.retries.lock().unwrap() += 1;
    }
}

fn policy() -> RetryPolicy {
    RetryPolicy::new(
        core::num::NonZeroU32::new(2).unwrap(),
        core::time::Duration::from_millis(10),
        core::time::Duration::from_millis(20),
        RetryJitter::None,
    )
}

fn entry(pid: u64) -> PidCacheEntry {
    PidCacheEntry::new(Pid::new(pid, 0), NodeId::new("node-a"), 99)
}

#[test]
fn hits_cache_without_runtime_call() {
    let cache = Arc::new(PidCache::new());
    let identity = ClusterIdentity::new("echo", "cached");
    cache.insert(identity.clone(), entry(1));
    let runtime = Arc::new(MockRuntime::new(vec![Ok(entry(2))]));
    let metrics = Arc::new(MockMetrics::default());
    let ctx = ClusterContext::new(runtime.clone(), cache.clone(), policy(), metrics);

    let result = ctx.request(&identity, &NodeId::new("req"));

    assert_eq!(result.unwrap().pid(), Pid::new(1, 0));
    assert_eq!(*runtime.calls.lock().unwrap(), 0);
}

#[test]
fn retries_until_success() {
    let cache = Arc::new(PidCache::new());
    let identity = ClusterIdentity::new("echo", "retry");
    let runtime = Arc::new(MockRuntime::new(vec![Err(ClusterError::Timeout), Ok(entry(3))]));
    let metrics = Arc::new(MockMetrics::default());
    let ctx = ClusterContext::new(runtime.clone(), cache.clone(), policy(), metrics);

    let result = ctx.request(&identity, &NodeId::new("req"));

    assert_eq!(result.unwrap().pid(), Pid::new(3, 0));
    assert_eq!(*runtime.calls.lock().unwrap(), 2);
}

#[test]
fn gives_up_after_exhaustion() {
    let cache = Arc::new(PidCache::new());
    let identity = ClusterIdentity::new("echo", "fail");
    let runtime = Arc::new(MockRuntime::new(vec![Err(ClusterError::Blocked), Err(ClusterError::Blocked)]));
    let metrics = Arc::new(MockMetrics::default());
    let ctx = ClusterContext::new(runtime.clone(), cache.clone(), policy(), metrics.clone());

    let result = ctx.request(&identity, &NodeId::new("req"));

    assert!(matches!(result, Err(ClusterError::Blocked)));
    assert_eq!(*runtime.calls.lock().unwrap(), 2);
    assert_eq!(metrics.retry_count(), 2);
}
