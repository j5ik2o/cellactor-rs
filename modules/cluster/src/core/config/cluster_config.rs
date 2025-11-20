use alloc::string::String;

use super::{
  cluster_config_builder::ClusterConfigBuilder, cluster_metrics_config::ClusterMetricsConfig,
  hash_strategy::HashStrategy, retry_policy::RetryPolicy, topology_watch::TopologyWatch,
};

#[cfg(test)]
mod tests;

/// User-facing configuration for the cluster runtime.
#[derive(Debug, Clone)]
pub struct ClusterConfig {
  system_name:          String,
  hash_strategy:        HashStrategy,
  retry_policy:         RetryPolicy,
  topology_watch:       TopologyWatch,
  allow_weighted_nodes: bool,
  metrics_config:       ClusterMetricsConfig,
  hash_seed:            u64,
}

impl ClusterConfig {
  pub(crate) const fn new(
    system_name: String,
    hash_strategy: HashStrategy,
    retry_policy: RetryPolicy,
    topology_watch: TopologyWatch,
    allow_weighted_nodes: bool,
    metrics_config: ClusterMetricsConfig,
    hash_seed: u64,
  ) -> Self {
    Self { system_name, hash_strategy, retry_policy, topology_watch, allow_weighted_nodes, metrics_config, hash_seed }
  }

  /// Creates a new builder instance.
  #[must_use]
  pub fn builder() -> ClusterConfigBuilder {
    ClusterConfigBuilder::new()
  }

  /// Returns the configured system name.
  #[must_use]
  pub fn system_name(&self) -> &str {
    &self.system_name
  }

  /// Returns the hashing strategy.
  #[must_use]
  pub const fn hash_strategy(&self) -> HashStrategy {
    self.hash_strategy
  }

  /// Returns the retry policy.
  #[must_use]
  pub const fn retry_policy(&self) -> &RetryPolicy {
    &self.retry_policy
  }

  /// Returns the topology watch handle.
  #[must_use]
  pub const fn topology_watch(&self) -> &TopologyWatch {
    &self.topology_watch
  }

  /// Whether weighted nodes are allowed.
  #[must_use]
  pub const fn allow_weighted_nodes(&self) -> bool {
    self.allow_weighted_nodes
  }

  /// Metrics configuration accessor.
  #[must_use]
  pub const fn metrics_config(&self) -> &ClusterMetricsConfig {
    &self.metrics_config
  }

  /// Hash seed used for rendezvous hashing.
  #[must_use]
  pub const fn hash_seed(&self) -> u64 {
    self.hash_seed
  }
}
