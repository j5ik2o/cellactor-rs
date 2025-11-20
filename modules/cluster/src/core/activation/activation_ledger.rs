use alloc::vec::Vec;

use hashbrown::HashMap;
use rapidhash::RapidBuildHasher;

use fraktor_utils_rs::core::runtime_toolbox::{RuntimeToolbox, SyncMutexFamily, ToolboxMutex};
use fraktor_utils_rs::core::sync::sync_mutex_like::SyncMutexLike;

use crate::core::identity::{ClusterIdentity, NodeId};

use super::activation_lease::ActivationLease;
use super::lease_id::LeaseId;
use super::lease_status::LeaseStatus;
use super::ledger_error::LedgerError;

/// Tracks activation leases per cluster identity.
pub struct ActivationLedger<TB>
where
    TB: RuntimeToolbox,
{
    state: ToolboxMutex<LedgerState, TB>,
}

struct LedgerState {
    sequence: u64,
    entries: LeaseMap,
}

type LeaseMap = HashMap<ClusterIdentity, ActivationLease, RapidBuildHasher>;

impl<TB> ActivationLedger<TB>
where
    TB: RuntimeToolbox,
{
    /// Creates an empty activation ledger.
    #[must_use]
    pub fn new() -> Self {
        let state = LedgerState::default();
        Self {
            state: <TB::MutexFamily as SyncMutexFamily>::create(state),
        }
    }

    /// Attempts to acquire a lease for the provided identity.
    pub fn acquire(
        &self,
        identity: ClusterIdentity,
        owner: NodeId,
        topology_hash: u64,
    ) -> Result<ActivationLease, LedgerError> {
        let mut guard = self.state.lock();
        if let Some(existing) = guard.entries.get(&identity) {
            return Err(LedgerError::AlreadyOwned { existing: existing.clone() });
        }

        guard.sequence = guard.sequence.wrapping_add(1);
        let lease_id = LeaseId::new(guard.sequence);
        let lease = ActivationLease::new(lease_id, owner, topology_hash, LeaseStatus::Active);
        guard.entries.insert(identity, lease.clone());
        Ok(lease)
    }

    /// Releases a previously acquired lease.
    pub fn release(&self, identity: &ClusterIdentity, lease_id: LeaseId) -> bool {
        let mut guard = self.state.lock();
        match guard.entries.get(identity) {
            Some(entry) if entry.lease_id() == lease_id => {
                guard.entries.remove(identity);
                true
            }
            _ => false,
        }
    }

    /// Fetches the lease associated with the identity, if any.
    #[must_use]
    pub fn get(&self, identity: &ClusterIdentity) -> Option<ActivationLease> {
        let guard = self.state.lock();
        guard.entries.get(identity).cloned()
    }

    /// Revokes all leases owned by the provided node.
    pub fn revoke_owner(&self, owner: &NodeId) -> Vec<(ClusterIdentity, ActivationLease)> {
        let mut guard = self.state.lock();
        let mut revoked = Vec::new();
        guard.entries.retain(|identity, lease| {
            if lease.owner() == owner {
                let mut revoked_lease = lease.clone();
                revoked_lease.set_status(LeaseStatus::Revoked);
                revoked.push((identity.clone(), revoked_lease));
                false
            } else {
                true
            }
        });
        revoked
    }

    /// Releases all tracked leases, returning their identities and leases.
    pub fn release_all(&self) -> Vec<(ClusterIdentity, ActivationLease)> {
        let mut guard = self.state.lock();
        guard.entries.drain().collect()
    }
}

impl Default for LedgerState {
    fn default() -> Self {
        Self { sequence: 0, entries: LeaseMap::with_hasher(RapidBuildHasher::default()) }
    }
}

#[cfg(test)]
mod tests;
