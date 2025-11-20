//! Placement domain components.

/// Actor responsible for driving activations.
mod placement_actor;
/// Trait abstracting spawner implementation.
mod placement_spawner;
/// Errors returned by placement spawners.
mod placement_spawner_error;

pub use placement_actor::PlacementActor;
pub use placement_spawner::PlacementSpawner;
pub use placement_spawner_error::PlacementSpawnerError;
