//! Configuration domain for the cluster runtime.

/// User-facing configuration entry points.
pub mod cluster_config;
/// Builder for `ClusterConfig`.
pub mod cluster_config_builder;
/// Errors produced while composing a configuration.
pub mod cluster_config_error;
/// Metrics configuration helpers.
pub mod cluster_metrics_config;
/// Rendezvous hashing strategy definition.
pub mod hash_strategy;
/// Retry jitter enumeration.
pub mod retry_jitter;
/// Retry policy structure.
pub mod retry_policy;
/// Trait representing topology streams supplied by providers.
pub mod topology_stream;
/// Handle that owns a topology stream.
pub mod topology_watch;

pub use cluster_config::ClusterConfig;
pub use cluster_config_builder::ClusterConfigBuilder;
pub use cluster_config_error::ClusterConfigError;
pub use cluster_metrics_config::ClusterMetricsConfig;
pub use hash_strategy::HashStrategy;
pub use retry_jitter::RetryJitter;
pub use retry_policy::RetryPolicy;
pub use topology_stream::TopologyStream;
pub use topology_watch::TopologyWatch;
