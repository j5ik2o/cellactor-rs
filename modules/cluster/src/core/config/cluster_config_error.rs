/// Errors that can occur while building a cluster configuration.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ClusterConfigError {
  /// Missing topology watcher handle while finalizing the configuration.
  #[error("topology watch must be configured before building the cluster")]
  MissingTopologyWatch,
}
