use std::time::Duration;

use crate::{
  core::{
    config::ClusterMetricsConfig,
    identity::{ClusterIdentity, NodeId},
  },
  std::metrics::StdClusterMetrics,
};

fn config(enabled: bool) -> ClusterMetricsConfig {
  ClusterMetricsConfig::new("ns".to_string(), enabled)
}

#[test]
fn records_metrics_when_enabled() {
  let metrics = StdClusterMetrics::new(&config(true));
  let id = ClusterIdentity::new("echo", "a");

  metrics.record_resolve_duration(&id, Duration::from_millis(5));
  metrics.record_request_duration(&id, Duration::from_millis(7));
  metrics.record_retry_attempt(&id);
  metrics.record_timeout(&id);
  metrics.set_virtual_actor_gauge(3);
  metrics.increment_block_list(&NodeId::new("node-a"));

  assert_eq!(metrics.resolve_count(), 1);
  assert_eq!(metrics.request_count(), 1);
  assert_eq!(metrics.retry_count(), 1);
  assert_eq!(metrics.timeout_count(), 1);
  assert_eq!(metrics.virtual_actor_gauge(), 3);
  assert_eq!(metrics.block_list_count(), 1);
  assert_eq!(metrics.last_resolve_duration(), Some(Duration::from_millis(5)));
  assert_eq!(metrics.last_request_duration(), Some(Duration::from_millis(7)));
}

#[test]
fn disabled_metrics_are_noop() {
  let metrics = StdClusterMetrics::new(&config(false));
  let id = ClusterIdentity::new("echo", "a");

  metrics.record_resolve_duration(&id, Duration::from_millis(5));
  metrics.record_request_duration(&id, Duration::from_millis(7));
  metrics.record_retry_attempt(&id);
  metrics.record_timeout(&id);
  metrics.set_virtual_actor_gauge(3);
  metrics.increment_block_list(&NodeId::new("node-a"));

  assert_eq!(metrics.resolve_count(), 0);
  assert_eq!(metrics.request_count(), 0);
  assert_eq!(metrics.retry_count(), 0);
  assert_eq!(metrics.timeout_count(), 0);
  assert_eq!(metrics.virtual_actor_gauge(), 0);
  assert_eq!(metrics.block_list_count(), 0);
  assert_eq!(metrics.last_resolve_duration(), None);
  assert_eq!(metrics.last_request_duration(), None);
}
use crate::core::metrics::ClusterMetrics;
