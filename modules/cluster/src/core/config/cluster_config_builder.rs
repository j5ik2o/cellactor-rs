use alloc::string::String;

use super::{
  cluster_config::ClusterConfig, cluster_config_error::ClusterConfigError,
  cluster_metrics_config::ClusterMetricsConfig, hash_strategy::HashStrategy, retry_policy::RetryPolicy,
  topology_watch::TopologyWatch,
};

/// Builder used to compose `ClusterConfig` instances.
#[derive(Debug, Clone)]
pub struct ClusterConfigBuilder {
  system_name:          Option<String>,
  hash_strategy:        HashStrategy,
  retry_policy:         RetryPolicy,
  topology_watch:       Option<TopologyWatch>,
  allow_weighted_nodes: bool,
  metrics_config:       ClusterMetricsConfig,
  hash_seed:            u64,
}

impl ClusterConfigBuilder {
  /// Creates a new builder instance with defaults.
  #[must_use]
  pub fn new() -> Self {
    Self {
      system_name:          None,
      hash_strategy:        HashStrategy::default(),
      retry_policy:         RetryPolicy::default(),
      topology_watch:       None,
      allow_weighted_nodes: true,
      metrics_config:       ClusterMetricsConfig::default(),
      hash_seed:            0,
    }
  }

  /// Sets the system name used in actor paths.
  pub fn system_name(mut self, name: impl Into<String>) -> Self {
    self.system_name = Some(name.into());
    self
  }

  /// Selects the rendezvous hash strategy.
  #[must_use]
  pub const fn hash_strategy(mut self, strategy: HashStrategy) -> Self {
    self.hash_strategy = strategy;
    self
  }

  /// Overrides the retry policy.
  #[must_use]
  pub const fn retry_policy(mut self, policy: RetryPolicy) -> Self {
    self.retry_policy = policy;
    self
  }

  /// Sets the topology watch handle supplying membership updates.
  #[must_use]
  pub fn topology_watch(mut self, watch: TopologyWatch) -> Self {
    self.topology_watch = Some(watch);
    self
  }

  /// Controls whether weighted rendezvous is allowed.
  #[must_use]
  pub const fn allow_weighted_nodes(mut self, enabled: bool) -> Self {
    self.allow_weighted_nodes = enabled;
    self
  }

  /// Adjusts metrics export configuration.
  #[must_use]
  pub fn metrics_config(mut self, config: ClusterMetricsConfig) -> Self {
    self.metrics_config = config;
    self
  }

  /// Overrides the hash seed used for rendezvous hashing.
  #[must_use]
  pub const fn hash_seed(mut self, seed: u64) -> Self {
    self.hash_seed = seed;
    self
  }

  /// Finalizes the builder into a cluster configuration.
  ///
  /// # Errors
  ///
  /// Returns `ClusterConfigError::MissingTopologyWatch` if no topology watch was configured.
  pub fn build(self) -> Result<ClusterConfig, ClusterConfigError> {
    let system_name = self.system_name.unwrap_or_else(|| String::from("fraktor-cluster"));
    let topology_watch = self.topology_watch.ok_or(ClusterConfigError::MissingTopologyWatch)?;

    Ok(ClusterConfig::new(
      system_name,
      self.hash_strategy,
      self.retry_policy,
      topology_watch,
      self.allow_weighted_nodes,
      self.metrics_config,
      self.hash_seed,
    ))
  }
}

impl Default for ClusterConfigBuilder {
  fn default() -> Self {
    Self::new()
  }
}
