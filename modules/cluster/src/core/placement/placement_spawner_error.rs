use super::super::activation::activation_error::ActivationError;

/// Errors returned when placement cannot spawn an actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementSpawnerError {
    /// Encountered an unknown cluster kind.
    UnknownKind,
    /// Underlying runtime failed to spawn the actor.
    SpawnFailed,
}

impl From<PlacementSpawnerError> for ActivationError {
    fn from(value: PlacementSpawnerError) -> Self {
        match value {
            PlacementSpawnerError::UnknownKind => ActivationError::UnknownKind,
            PlacementSpawnerError::SpawnFailed => ActivationError::SpawnFailed,
        }
    }
}
