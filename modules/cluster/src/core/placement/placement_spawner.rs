use fraktor_actor_rs::core::actor_prim::Pid;
use fraktor_actor_rs::core::props::PropsGeneric;
use fraktor_utils_rs::core::runtime_toolbox::RuntimeToolbox;

use crate::core::identity::ClusterIdentity;

use super::placement_spawner_error::PlacementSpawnerError;

/// Abstraction over the component that spawns actors for placement.
pub trait PlacementSpawner<TB>
where
    TB: RuntimeToolbox + 'static,
{
    /// Attempts to spawn the actor described by the props.
    fn spawn(
        &self,
        identity: &ClusterIdentity,
        props: PropsGeneric<TB>,
    ) -> Result<Pid, PlacementSpawnerError>;
}
