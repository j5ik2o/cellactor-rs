use alloc::{boxed::Box, string::String};

use crate::core::config::{
  ClusterConfig, ClusterConfigError, ClusterMetricsConfig, HashStrategy, RetryJitter, RetryPolicy, TopologyWatch,
  topology_stream::TopologyStream,
};

#[derive(Clone, Debug)]
struct MockTopologyStream {
  id: &'static str,
}

impl MockTopologyStream {
  fn new(id: &'static str) -> Self {
    Self { id }
  }
}

impl TopologyStream for MockTopologyStream {
  fn stream_id(&self) -> &'static str {
    self.id
  }
}

fn sample_retry_policy() -> RetryPolicy {
  use core::{num::NonZeroU32, time::Duration};

  RetryPolicy::new(
    NonZeroU32::new(5).expect("non-zero"),
    Duration::from_millis(30),
    Duration::from_millis(500),
    RetryJitter::Decorrelated,
  )
}

fn sample_watch(label: &'static str) -> TopologyWatch {
  TopologyWatch::new(Box::new(MockTopologyStream::new(label)))
}

#[test]
fn builder_populates_all_fields() {
  let metrics = ClusterMetricsConfig::new(String::from("telemetry"), false);
  let retry = sample_retry_policy();
  let watch = sample_watch("stream");

  let config = ClusterConfig::builder()
    .system_name("fraktor-dev")
    .hash_strategy(HashStrategy::WeightedRendezvous)
    .retry_policy(retry.clone())
    .topology_watch(watch.clone())
    .allow_weighted_nodes(false)
    .metrics_config(metrics.clone())
    .hash_seed(42)
    .build()
    .expect("builder success");

  assert_eq!(config.system_name(), "fraktor-dev");
  assert_eq!(config.hash_strategy(), HashStrategy::WeightedRendezvous);
  assert_eq!(config.retry_policy(), &retry);
  assert_eq!(config.topology_watch().stream_id(), "stream");
  assert!(!config.allow_weighted_nodes());
  assert_eq!(config.metrics_config(), &metrics);
  assert_eq!(config.hash_seed(), 42);
}

#[test]
fn builder_requires_topology_watch() {
  let err = ClusterConfig::builder().system_name("fraktor-dev").build().expect_err("missing watch should error");

  assert_eq!(err, ClusterConfigError::MissingTopologyWatch);
}
