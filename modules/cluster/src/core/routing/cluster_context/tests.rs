use alloc::sync::Arc;

use fraktor_actor_rs::core::actor_prim::Pid;
use fraktor_utils_rs::core::runtime_toolbox::NoStdToolbox;

use crate::core::config::{RetryJitter, RetryPolicy};
use crate::core::identity::{ClusterIdentity, NodeId};
use crate::core::routing::{ClusterContext, ClusterError, PidCache, PidCacheEntry};
use crate::core::routing::cluster_context::ResolveBridge;

struct MockRuntime {
    results: std::sync::Mutex<Vec<Result<PidCacheEntry, ClusterError>>>,
    calls: std::sync::Mutex<u32>,
}

impl MockRuntime {
    fn new(results: Vec<Result<PidCacheEntry, ClusterError>>) -> Self {
        Self { results: std::sync::Mutex::new(results), calls: std::sync::Mutex::new(0) }
    }
}

impl ResolveBridge<NoStdToolbox> for MockRuntime {
    fn resolve(&self, _: &ClusterIdentity, _: &NodeId) -> Result<PidCacheEntry, ClusterError> {
        *self.calls.lock().unwrap() += 1;
        self.results.lock().unwrap().remove(0)
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
    let ctx = ClusterContext::new(runtime.clone(), cache.clone(), policy());

    let result = ctx.request(&identity, &NodeId::new("req"));

    assert_eq!(result.unwrap().pid(), Pid::new(1, 0));
    assert_eq!(*runtime.calls.lock().unwrap(), 0);
}

#[test]
fn retries_until_success() {
    let cache = Arc::new(PidCache::new());
    let identity = ClusterIdentity::new("echo", "retry");
    let runtime = Arc::new(MockRuntime::new(vec![Err(ClusterError::Timeout), Ok(entry(3))]));
    let ctx = ClusterContext::new(runtime.clone(), cache.clone(), policy());

    let result = ctx.request(&identity, &NodeId::new("req"));

    assert_eq!(result.unwrap().pid(), Pid::new(3, 0));
    assert_eq!(*runtime.calls.lock().unwrap(), 2);
}

#[test]
fn gives_up_after_exhaustion() {
    let cache = Arc::new(PidCache::new());
    let identity = ClusterIdentity::new("echo", "fail");
    let runtime = Arc::new(MockRuntime::new(vec![Err(ClusterError::Blocked), Err(ClusterError::Blocked)]));
    let ctx = ClusterContext::new(runtime.clone(), cache.clone(), policy());

    let result = ctx.request(&identity, &NodeId::new("req"));

    assert!(matches!(result, Err(ClusterError::Blocked)));
    assert_eq!(*runtime.calls.lock().unwrap(), 2);
}
