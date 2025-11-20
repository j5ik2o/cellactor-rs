use core::any::Any;

/// Metrics sink for cluster-specific signals.
pub trait ClusterMetrics: Send + Sync + 'static {
    /// Returns the metrics instance as `Any` for downcasting in tests.
    fn as_any(&self) -> &dyn Any;
}
