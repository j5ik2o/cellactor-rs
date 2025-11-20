/// Errors returned by cluster routing APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterError {
    /// Cluster runtime is shutting down.
    ShuttingDown,
    /// Target was blocked via BlockList.
    Blocked,
    /// Request timed out after retries.
    Timeout,
    /// Unknown cluster kind.
    NoSuchKind,
    /// Generic runtime failure.
    RuntimeFailure,
}
