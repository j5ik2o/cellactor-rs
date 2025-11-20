use alloc::sync::Arc;
use core::time::Duration;

use fraktor_utils_rs::core::runtime_toolbox::RuntimeToolbox;

use crate::core::config::RetryPolicy;
use crate::core::identity::{ClusterIdentity, NodeId};
use crate::core::metrics::ClusterMetrics;

use super::cluster_error::ClusterError;
use super::pid_cache::{PidCache, PidCacheEntry};
use super::retry::{RetryOutcome, RetryPolicyRunner};

/// Trait abstracting runtime resolve/sender operations for tests.
pub trait ResolveBridge<TB>: Send + Sync + 'static
where
    TB: RuntimeToolbox,
{
    /// Resolves the identity into a PID cache entry.
    fn resolve(&self, identity: &ClusterIdentity, requester: &NodeId) -> Result<PidCacheEntry, ClusterError>;
}

/// Cluster routing context that caches PIDs and applies retry policy.
pub struct ClusterContext<TB>
where
    TB: RuntimeToolbox,
{
    runtime: Arc<dyn ResolveBridge<TB>>,
    cache: Arc<PidCache<TB>>,
    policy: RetryPolicy,
    metrics: Arc<dyn ClusterMetrics>,
}

impl<TB> ClusterContext<TB>
where
    TB: RuntimeToolbox,
{
    /// Creates a new cluster context.
    #[must_use]
    pub fn new(
        runtime: Arc<dyn ResolveBridge<TB>>,
        cache: Arc<PidCache<TB>>,
        policy: RetryPolicy,
        metrics: Arc<dyn ClusterMetrics>,
    ) -> Self {
        Self { runtime, cache, policy, metrics }
    }

    /// Resolves a PID, consulting the cache first and applying retries when needed.
    pub fn request(&self, identity: &ClusterIdentity, requester: &NodeId) -> Result<PidCacheEntry, ClusterError> {
        if let Some(entry) = self.cache.get(identity) {
            self.metrics.record_resolve_duration(identity, Duration::ZERO);
            self.metrics.record_request_duration(identity, Duration::ZERO);
            return Ok(entry);
        }

        let mut runner = RetryPolicyRunner::new(self.policy.clone());
        loop {
            match self.runtime.resolve(identity, requester) {
                Ok(entry) => {
                    self.cache.insert(identity.clone(), entry.clone());
                    self.metrics.record_resolve_duration(identity, Duration::ZERO);
                    self.metrics.record_request_duration(identity, Duration::ZERO);
                    return Ok(entry);
                }
                Err(err) => match runner.next_outcome() {
                    RetryOutcome::RetryAfter(_) => {
                        self.cache.invalidate(identity);
                        self.metrics.record_retry_attempt(identity);
                        continue;
                    }
                    RetryOutcome::GiveUp => {
                        self.metrics.record_timeout(identity);
                        return Err(ClusterError::Timeout);
                    }
                },
            }
        }
    }

    /// Explicitly invalidates a cached entry.
    pub fn invalidate(&self, identity: &ClusterIdentity) {
        self.cache.invalidate(identity);
    }
}

#[cfg(test)]
mod tests;
