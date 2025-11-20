use alloc::sync::Arc;

use fraktor_utils_rs::core::runtime_toolbox::RuntimeToolbox;

use crate::core::activation::{ActivationLedger, LedgerError};
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
    ) -> Self {
        Self {
            config,
            identity,
            activation,
            metrics,
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

    /// Resolves the owner for the provided cluster identity.
    pub fn resolve_owner(
        &self,
        identity: &ClusterIdentity,
        requester: &NodeId,
    ) -> Result<OwnerResolution, ResolveError> {
        let owner = self
            .identity
            .select_owner(identity, requester)
            .ok_or(ResolveError::RingEmpty)?;
        let topology_hash = self.identity.topology_hash();
        match self
            .activation
            .acquire(identity.clone(), owner.id().clone(), topology_hash)
        {
            Ok(lease) => Ok(OwnerResolution::new(owner, lease)),
            Err(LedgerError::AlreadyOwned { existing }) => Err(ResolveError::LeaseConflict { existing }),
        }
    }
}

#[cfg(test)]
mod tests;
