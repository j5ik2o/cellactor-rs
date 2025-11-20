use core::any::Any;

use crate::core::metrics::cluster_metrics::ClusterMetrics;

/// No-op metrics implementation for environments without instrumentation.
struct NoopClusterMetrics;

impl ClusterMetrics for NoopClusterMetrics {
  fn as_any(&self) -> &dyn Any {
    self
  }
}

#[test]
fn test_noop_cluster_metrics() {
  let metrics = NoopClusterMetrics;
  let _any = metrics.as_any();
}
