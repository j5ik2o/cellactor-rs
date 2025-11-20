use alloc::sync::Arc;

use fraktor_utils_rs::core::runtime_toolbox::RuntimeToolbox;

use crate::core::config::RetryPolicy;
use crate::core::identity::{ClusterIdentity, NodeId};

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
}

impl<TB> ClusterContext<TB>
where
    TB: RuntimeToolbox,
{
    /// Creates a new cluster context.
    #[must_use]
    pub fn new(runtime: Arc<dyn ResolveBridge<TB>>, cache: Arc<PidCache<TB>>, policy: RetryPolicy) -> Self {
        Self { runtime, cache, policy }
    }

    /// Resolves a PID, consulting the cache first and applying retries when needed.
    pub fn request(&self, identity: &ClusterIdentity, requester: &NodeId) -> Result<PidCacheEntry, ClusterError> {
        if let Some(entry) = self.cache.get(identity) {
            return Ok(entry);
        }

        let mut runner = RetryPolicyRunner::new(self.policy.clone());
        loop {
            match self.runtime.resolve(identity, requester) {
                Ok(entry) => {
                    self.cache.insert(identity.clone(), entry.clone());
                    return Ok(entry);
                }
                Err(err) => match runner.next() {
                    RetryOutcome::RetryAfter(_) => {
                        self.cache.invalidate(identity);
                        continue;
                    }
                    RetryOutcome::GiveUp => return Err(err),
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
