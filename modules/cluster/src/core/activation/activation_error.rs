/// Errors that can occur during activation processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationError {
    /// Requested kind is not registered.
    UnknownKind,
    /// Actor spawning failed.
    SpawnFailed,
    /// Actor terminated and lease should be discarded.
    Terminated,
}
