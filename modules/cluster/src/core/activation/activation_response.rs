use fraktor_actor_rs::core::actor_prim::Pid;

use crate::core::identity::ClusterIdentity;

use super::activation_error::ActivationError;
use super::lease_id::LeaseId;

/// Message returned by the placement layer after processing an activation request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivationResponse {
    identity: ClusterIdentity,
    pid: Option<Pid>,
    lease_id: LeaseId,
    topology_hash: u64,
    error: Option<ActivationError>,
}

impl ActivationResponse {
    /// Creates a successful response containing the allocated PID.
    #[must_use]
    pub fn success(identity: ClusterIdentity, pid: Pid, lease_id: LeaseId, topology_hash: u64) -> Self {
        Self { identity, pid: Some(pid), lease_id, topology_hash, error: None }
    }

    /// Creates a failed response containing the activation error.
    #[must_use]
    pub fn failure(identity: ClusterIdentity, error: ActivationError, lease_id: LeaseId, topology_hash: u64) -> Self {
        Self { identity, pid: None, lease_id, topology_hash, error: Some(error) }
    }

    /// Returns the identity that originated the request.
    #[must_use]
    pub fn identity(&self) -> &ClusterIdentity {
        &self.identity
    }

    /// Returns the PID if activation succeeded.
    #[must_use]
    pub fn pid(&self) -> Option<Pid> {
        self.pid
    }

    /// Returns the lease identifier tied to the response.
    #[must_use]
    pub fn lease_id(&self) -> LeaseId {
        self.lease_id
    }

    /// Returns the topology hash observed by the placement actor.
    #[must_use]
    pub fn topology_hash(&self) -> u64 {
        self.topology_hash
    }

    /// Returns the activation error, if any.
    #[must_use]
    pub fn error(&self) -> Option<&ActivationError> {
        self.error.as_ref()
    }
}
