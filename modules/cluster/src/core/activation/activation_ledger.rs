use alloc::vec::Vec;

use fraktor_utils_rs::core::{
  runtime_toolbox::{RuntimeToolbox, SyncMutexFamily, ToolboxMutex},
  sync::sync_mutex_like::SyncMutexLike,
};
use hashbrown::HashMap;
use rapidhash::RapidBuildHasher;

use super::{
  activation_lease::ActivationLease, lease_id::LeaseId, lease_status::LeaseStatus, ledger_error::LedgerError,
};
use crate::core::identity::{ClusterIdentity, NodeId};

#[cfg(test)]
mod tests;

/// Tracks activation leases per cluster identity.
pub struct ActivationLedger<TB>
where
  TB: RuntimeToolbox, {
  state: ToolboxMutex<LedgerState, TB>,
}

struct LedgerState {
  sequence: u64,
  entries:  LeaseMap,
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
    Self { state: <TB::MutexFamily as SyncMutexFamily>::create(state) }
  }

  /// Returns true when no leases are tracked.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.state.lock().entries.is_empty()
  }

  /// Attempts to acquire a lease for the provided identity.
  ///
  /// # Errors
  ///
  /// Returns `LedgerError::AlreadyOwned` if a lease already exists for the given identity.
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
      | Some(entry) if entry.lease_id() == lease_id => {
        guard.entries.remove(identity);
        true
      },
      | _ => false,
    }
  }

  /// Marks a lease as releasing in preparation for graceful shutdown.
  pub fn mark_releasing(&self, identity: &ClusterIdentity, lease_id: LeaseId) -> Option<ActivationLease> {
    let mut guard = self.state.lock();
    let lease = guard.entries.get_mut(identity)?;
    if lease.lease_id() != lease_id {
      return None;
    }
    lease.set_status(LeaseStatus::Releasing);
    Some(lease.clone())
  }

  /// Completes a release and removes the lease.
  pub fn complete_release(&self, identity: &ClusterIdentity, lease_id: LeaseId) -> Option<ActivationLease> {
    let mut guard = self.state.lock();
    match guard.entries.get(identity) {
      | Some(entry) if entry.lease_id() == lease_id => {
        let mut lease = entry.clone();
        lease.set_status(LeaseStatus::Released);
        guard.entries.remove(identity);
        Some(lease)
      },
      | _ => None,
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

  /// Returns the number of tracked leases.
  pub fn len(&self) -> usize {
    self.state.lock().entries.len()
  }
}

impl<TB> Default for ActivationLedger<TB>
where
  TB: RuntimeToolbox,
{
  fn default() -> Self {
    Self::new()
  }
}
impl Default for LedgerState {
  fn default() -> Self {
    Self { sequence: 0, entries: LeaseMap::with_hasher(RapidBuildHasher::default()) }
  }
}
