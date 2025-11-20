//! Metrics adapters for the cluster runtime.

/// Trait describing cluster metrics sinks.
mod cluster_metrics;

pub use cluster_metrics::ClusterMetrics;
