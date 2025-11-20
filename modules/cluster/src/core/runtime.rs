use alloc::{sync::Arc, vec::Vec};
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;

use fraktor_utils_rs::core::runtime_toolbox::RuntimeToolbox;

use crate::core::activation::{ActivationLease, ActivationLedger, ActivationRequest, PartitionBridge, PartitionBridgeError, LedgerError};
use crate::core::routing::PidCache;
use crate::core::config::ClusterConfig;
use crate::core::identity::{ClusterIdentity, IdentityLookupService, NodeId};
use crate::core::metrics::ClusterMetrics;

/// Resolution result helpers.
pub mod owner_resolution;
/// Errors raised during owner resolution.
pub mod resolve_error;

pub use owner_resolution::OwnerResolution;
pub use resolve_error::ResolveError;

/// Runtime bundle that exposes cluster services to extensions.
pub struct ClusterRuntime<TB>
where
    TB: RuntimeToolbox,
{
    config: ClusterConfig,
    identity: Arc<IdentityLookupService<TB>>,
    activation: Arc<ActivationLedger<TB>>,
    metrics: Arc<dyn ClusterMetrics>,
    bridge: Arc<dyn PartitionBridge<TB>>,
    pid_cache: Arc<PidCache<TB>>,
    shutting_down: AtomicBool,
}

impl<TB> ClusterRuntime<TB>
where
    TB: RuntimeToolbox,
{
    /// Creates a new runtime bundle.
    pub fn new(
        config: ClusterConfig,
        identity: Arc<IdentityLookupService<TB>>,
        activation: Arc<ActivationLedger<TB>>,
        metrics: Arc<dyn ClusterMetrics>,
        bridge: Arc<dyn PartitionBridge<TB>>,
        pid_cache: Arc<PidCache<TB>>,
    ) -> Self {
        Self {
            config,
            identity,
            activation,
            metrics,
            bridge,
            pid_cache,
            shutting_down: AtomicBool::new(false),
        }
    }

    /// Returns the runtime configuration.
    pub fn config(&self) -> &ClusterConfig {
        &self.config
    }

    /// Returns the identity lookup service handle.
    pub fn identity(&self) -> &IdentityLookupService<TB> {
        &self.identity
    }

    /// Returns the activation ledger handle.
    pub fn activation(&self) -> &ActivationLedger<TB> {
        &self.activation
    }

    /// Returns the metrics sink.
    pub fn metrics(&self) -> &dyn ClusterMetrics {
        self.metrics.as_ref()
    }

    /// Returns the PID cache handle.
    pub fn pid_cache(&self) -> &PidCache<TB> {
        &self.pid_cache
    }

    /// Resolves the owner for the provided cluster identity.
    pub fn resolve_owner(
        &self,
        identity: &ClusterIdentity,
        requester: &NodeId,
    ) -> Result<OwnerResolution, ResolveError> {
        if self.shutting_down.load(Ordering::SeqCst) {
            return Err(ResolveError::ShuttingDown);
        }
        let owner = self
            .identity
            .select_owner(identity, requester)
            .ok_or(ResolveError::RingEmpty)?;
        let topology_hash = self.identity.topology_hash();
        match self
            .activation
            .acquire(identity.clone(), owner.id().clone(), topology_hash)
        {
            Ok(lease) => {
                self.metrics.record_resolve_duration(identity, Duration::ZERO);
                self.metrics.set_virtual_actor_gauge(self.activation.len());
                Ok(OwnerResolution::new(owner, lease))
            }
            Err(LedgerError::AlreadyOwned { existing }) => Err(ResolveError::LeaseConflict { existing }),
        }
    }

    /// Handles block list notification for the provided node.
    pub fn handle_blocked_node(&self, node: &NodeId) -> Vec<(ClusterIdentity, ActivationLease)> {
        self.metrics.increment_block_list(node);
        self.pid_cache.invalidate_node(node);
        let revoked = self.activation.revoke_owner(node);
        self.metrics.set_virtual_actor_gauge(self.activation.len());
        revoked
    }

    /// Begins graceful shutdown by rejecting future resolves and releasing leases.
    pub fn begin_shutdown(&self) -> Vec<(ClusterIdentity, ActivationLease)> {
        self.shutting_down.store(true, Ordering::SeqCst);
        self.pid_cache.clear();
        let released = self.activation.release_all();
        self.metrics.set_virtual_actor_gauge(0);
        released
    }

    /// Dispatches the activation request through the partition bridge.
    pub fn dispatch_activation_request(
        &self,
        request: ActivationRequest<TB>,
    ) -> Result<(), PartitionBridgeError> {
        self.bridge.send_activation_request(request)
    }

    /// Returns the partition bridge handle.
    pub fn partition_bridge(&self) -> &dyn PartitionBridge<TB> {
        self.bridge.as_ref()
    }
}

#[cfg(test)]
mod tests;
