//! Placement domain components.

/// Actor responsible for driving activations.
pub mod placement_actor;
/// Trait abstracting spawner implementation.
pub mod placement_spawner;
/// Errors returned by placement spawners.
pub mod placement_spawner_error;

pub use placement_actor::PlacementActor;
pub use placement_spawner::PlacementSpawner;
pub use placement_spawner_error::PlacementSpawnerError;
