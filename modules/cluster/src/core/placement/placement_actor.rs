use fraktor_actor_rs::core::actor_prim::Pid;
use fraktor_utils_rs::core::runtime_toolbox::RuntimeToolbox;

use super::{placement_spawner::PlacementSpawner, placement_spawner_error::PlacementSpawnerError};
use crate::core::{
  activation::{ActivationError, ActivationRequest, ActivationResponse, LeaseId},
  identity::ClusterIdentity,
};

#[cfg(test)]
mod tests;

/// Drives activation requests and emits responses based on spawner outcomes.
pub struct PlacementActor<TB, S>
where
  TB: RuntimeToolbox + 'static,
  S: PlacementSpawner<TB>, {
  spawner: S,
  _marker: core::marker::PhantomData<TB>,
}

impl<TB, S> PlacementActor<TB, S>
where
  TB: RuntimeToolbox + 'static,
  S: PlacementSpawner<TB>,
{
  /// Creates a placement actor backed by the provided spawner implementation.
  #[must_use]
  pub const fn new(spawner: S) -> Self {
    Self { spawner, _marker: core::marker::PhantomData }
  }

  /// Handles a freshly routed activation request and produces a response.
  pub fn handle_activation(&self, request: ActivationRequest<TB>) -> ActivationResponse {
    let (identity, lease, props) = request.into_parts();
    let lease_id = lease.lease_id();
    let topology_hash = lease.topology_hash();
    match self.spawner.spawn(&identity, props) {
      | Ok(pid) => ActivationResponse::success(identity, pid, lease_id, topology_hash),
      | Err(err) => ActivationResponse::failure(identity, map_spawner_error(err), lease_id, topology_hash),
    }
  }

  /// Handles a termination notification for the provided identity.
  #[must_use]
  pub const fn handle_terminated(
    &self,
    identity: ClusterIdentity,
    lease_id: LeaseId,
    topology_hash: u64,
  ) -> ActivationResponse {
    ActivationResponse::failure(identity, ActivationError::Terminated, lease_id, topology_hash)
  }

  /// Handles a graceful release completion signal.
  #[must_use]
  pub const fn handle_released(
    &self,
    identity: ClusterIdentity,
    lease_id: LeaseId,
    topology_hash: u64,
  ) -> ActivationResponse {
    ActivationResponse::success(identity, Pid::new(0, 0), lease_id, topology_hash)
  }
}

const fn map_spawner_error(err: PlacementSpawnerError) -> ActivationError {
  match err {
    | PlacementSpawnerError::UnknownKind => ActivationError::UnknownKind,
    | PlacementSpawnerError::SpawnFailed => ActivationError::SpawnFailed,
  }
}
