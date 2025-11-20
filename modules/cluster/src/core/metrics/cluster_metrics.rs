use core::any::Any;
use core::time::Duration;

use crate::core::identity::{ClusterIdentity, NodeId};

/// Metrics sink for cluster-specific signals.
pub trait ClusterMetrics: Send + Sync + 'static {
    /// Returns the metrics instance as `Any` for downcasting in tests.
    fn as_any(&self) -> &dyn Any;

    /// Records the time spent resolving an identity.
    fn record_resolve_duration(&self, _identity: &ClusterIdentity, _duration: Duration) {}

    /// Records that a retry was attempted for the identity.
    fn record_retry(&self, _identity: &ClusterIdentity) {}

    /// Sets the current virtual actor gauge.
    fn set_virtual_actor_gauge(&self, _value: usize) {}

    /// Records that block list actions affected a node.
    fn increment_block_list(&self, _node: &NodeId) {}
}

/// No-op metrics implementation for environments without instrumentation.
pub struct NoopClusterMetrics;

impl ClusterMetrics for NoopClusterMetrics {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
