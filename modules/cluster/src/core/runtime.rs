use alloc::{sync::Arc, vec::Vec};
use core::{
  sync::atomic::{AtomicBool, Ordering},
  time::Duration,
};

use fraktor_utils_rs::core::runtime_toolbox::RuntimeToolbox;

use crate::core::{
  activation::{
    ActivationLease, ActivationLedger, ActivationRequest, LeaseId, LedgerError, PartitionBridge, PartitionBridgeError,
  },
  config::ClusterConfig,
  identity::{ClusterIdentity, IdentityLookupService, NodeId},
  metrics::ClusterMetrics,
  routing::{PidCache, cluster_error::ClusterError},
};

/// Resolution result helpers.
pub mod owner_resolution;
/// Errors raised during owner resolution.
pub mod resolve_error;

pub use owner_resolution::OwnerResolution;
pub use resolve_error::ResolveError;

/// Runtime bundle that exposes cluster services to extensions.
pub struct ClusterRuntime<TB>
where
  TB: RuntimeToolbox, {
  config:        ClusterConfig,
  identity:      Arc<IdentityLookupService<TB>>,
  activation:    Arc<ActivationLedger<TB>>,
  metrics:       Arc<dyn ClusterMetrics>,
  bridge:        Arc<dyn PartitionBridge<TB>>,
  pid_cache:     Arc<PidCache<TB>>,
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
    Self { config, identity, activation, metrics, bridge, pid_cache, shutting_down: AtomicBool::new(false) }
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
  pub fn resolve_owner(&self, identity: &ClusterIdentity, requester: &NodeId) -> Result<OwnerResolution, ResolveError> {
    if self.shutting_down.load(Ordering::SeqCst) {
      return Err(ResolveError::ShuttingDown);
    }
    let owner = self.identity.select_owner(identity, requester).ok_or(ResolveError::RingEmpty)?;
    let topology_hash = self.identity.topology_hash();
    match self.activation.acquire(identity.clone(), owner.id().clone(), topology_hash) {
      | Ok(lease) => {
        self.metrics.record_resolve_duration(identity, Duration::ZERO);
        self.metrics.set_virtual_actor_gauge(self.activation.len());
        Ok(OwnerResolution::new(owner, lease))
      },
      | Err(LedgerError::AlreadyOwned { existing }) => Err(ResolveError::LeaseConflict { existing }),
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

  /// Invalidates cache entries whose topology hash no longer matches the latest ring.
  pub fn handle_topology_changed(&self) -> Vec<ClusterIdentity> {
    let current_hash = self.identity.topology_hash();
    let mut removed = Vec::new();
    // Linear scan of cache to drop stale entries.
    let identities = self.pid_cache.keys();
    for identity in identities {
      if let Some(entry) = self.pid_cache.get(&identity) {
        if entry.topology_hash() != current_hash && self.pid_cache.invalidate(&identity) {
          removed.push(identity);
        }
      }
    }
    removed
  }

  /// Begins graceful shutdown by rejecting future resolves and releasing leases.
  pub fn begin_shutdown(&self) -> Vec<(ClusterIdentity, ActivationLease)> {
    self.shutting_down.store(true, Ordering::SeqCst);
    self.pid_cache.clear();
    let released = self.activation.release_all();
    self.metrics.set_virtual_actor_gauge(0);
    released
  }

  /// Marks a specific lease as releasing, allowing placement to poison actors.
  pub fn mark_releasing(&self, identity: &ClusterIdentity, lease_id: LeaseId) -> Option<ActivationLease> {
    if self.shutting_down.load(Ordering::SeqCst) {
      return None;
    }
    let lease = self.activation.mark_releasing(identity, lease_id)?;
    self.metrics.set_virtual_actor_gauge(self.activation.len());
    Some(lease)
  }

  /// Completes a lease release after termination is observed.
  pub fn complete_release(&self, identity: &ClusterIdentity, lease_id: LeaseId) -> Option<ActivationLease> {
    let lease = self.activation.complete_release(identity, lease_id)?;
    self.metrics.set_virtual_actor_gauge(self.activation.len());
    Some(lease)
  }

  /// Releases and clears cache when ownership changed.
  pub fn surrender_ownership(
    &self,
    identity: &ClusterIdentity,
    lease_id: LeaseId,
    _new_owner: &NodeId,
  ) -> Result<(), ClusterError> {
    match self.activation.complete_release(identity, lease_id) {
      | Some(_) => {
        self.pid_cache.invalidate(identity);
        self.metrics.set_virtual_actor_gauge(self.activation.len());
        Ok(())
      },
      | None => Err(ClusterError::OwnershipChanged),
    }
  }

  /// Dispatches the activation request through the partition bridge.
  pub fn dispatch_activation_request(&self, request: ActivationRequest<TB>) -> Result<(), PartitionBridgeError> {
    self.bridge.send_activation_request(request)
  }

  /// Returns the partition bridge handle.
  pub fn partition_bridge(&self) -> &dyn PartitionBridge<TB> {
    self.bridge.as_ref()
  }
}

#[cfg(test)]
mod tests;
