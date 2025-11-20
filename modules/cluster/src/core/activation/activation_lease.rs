use super::lease_id::LeaseId;
use super::lease_status::LeaseStatus;
use crate::core::identity::node_id::NodeId;

/// Records ownership information for a cluster identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivationLease {
    lease_id: LeaseId,
    owner: NodeId,
    topology_hash: u64,
    status: LeaseStatus,
}

impl ActivationLease {
    pub(crate) fn new(lease_id: LeaseId, owner: NodeId, topology_hash: u64, status: LeaseStatus) -> Self {
        Self { lease_id, owner, topology_hash, status }
    }

    /// Returns the lease identifier.
    #[must_use]
    pub fn lease_id(&self) -> LeaseId {
        self.lease_id
    }

    /// Returns the owning node identifier.
    #[must_use]
    pub fn owner(&self) -> &NodeId {
        &self.owner
    }

    /// Returns the topology hash associated with the acquisition.
    #[must_use]
    pub fn topology_hash(&self) -> u64 {
        self.topology_hash
    }

    /// Returns the current lease status.
    #[must_use]
    pub fn status(&self) -> LeaseStatus {
        self.status
    }

}
