//! Configuration domain for the cluster runtime.

/// User-facing configuration entry points.
mod cluster_config;
/// Builder for `ClusterConfig`.
mod cluster_config_builder;
/// Errors produced while composing a configuration.
mod cluster_config_error;
/// Metrics configuration helpers.
mod cluster_metrics_config;
/// Rendezvous hashing strategy definition.
mod hash_strategy;
/// Retry jitter enumeration.
mod retry_jitter;
/// Retry policy structure.
mod retry_policy;
/// Trait representing topology streams supplied by providers.
pub mod topology_stream;
/// Handle that owns a topology stream.
mod topology_watch;

pub use cluster_config::ClusterConfig;
pub use cluster_config_builder::ClusterConfigBuilder;
pub use cluster_config_error::ClusterConfigError;
pub use cluster_metrics_config::ClusterMetricsConfig;
pub use hash_strategy::HashStrategy;
pub use retry_jitter::RetryJitter;
pub use retry_policy::RetryPolicy;
pub use topology_watch::TopologyWatch;
