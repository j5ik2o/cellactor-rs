use fraktor_actor_rs::core::props::PropsGeneric;
use fraktor_utils_rs::core::runtime_toolbox::RuntimeToolbox;

use crate::core::identity::ClusterIdentity;

use super::activation_lease::ActivationLease;

/// Request message instructing the placement subsystem to spawn an activation.
pub struct ActivationRequest<TB>
where
    TB: RuntimeToolbox + 'static,
{
    identity: ClusterIdentity,
    lease: ActivationLease,
    props: PropsGeneric<TB>,
}

impl<TB> ActivationRequest<TB>
where
    TB: RuntimeToolbox + 'static,
{
    /// Creates a new activation request from the identity, lease, and props.
    #[must_use]
    pub fn new(identity: ClusterIdentity, lease: ActivationLease, props: PropsGeneric<TB>) -> Self {
        Self { identity, lease, props }
    }

    /// Returns the cluster identity slated for activation.
    #[must_use]
    pub fn identity(&self) -> &ClusterIdentity {
        &self.identity
    }

    /// Consumes the request and returns the identity.
    #[must_use]
    pub fn into_identity(self) -> ClusterIdentity {
        self.identity
    }

    /// Returns the activation lease granted by the runtime.
    #[must_use]
    pub fn lease(&self) -> &ActivationLease {
        &self.lease
    }

    /// Consumes the request and returns the lease.
    #[must_use]
    pub fn into_lease(self) -> ActivationLease {
        self.lease
    }

    /// Returns the props used to spawn the actor.
    #[must_use]
    pub fn props(&self) -> &PropsGeneric<TB> {
        &self.props
    }

    /// Consumes the request and returns the props.
    #[must_use]
    pub fn into_props(self) -> PropsGeneric<TB> {
        self.props
    }

    /// Decomposes the request into its components.
    #[must_use]
    pub fn into_parts(self) -> (ClusterIdentity, ActivationLease, PropsGeneric<TB>) {
        (self.identity, self.lease, self.props)
    }
}
