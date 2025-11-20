use alloc::string::String;

/// Configuration for metrics export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterMetricsConfig {
    namespace: String,
    enabled: bool,
}

impl ClusterMetricsConfig {
    /// Creates a new metrics configuration entry.
    pub fn new(namespace: String, enabled: bool) -> Self {
        Self { namespace, enabled }
    }

    /// Returns the configured namespace for metrics instruments.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Whether metrics publishing is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for ClusterMetricsConfig {
    fn default() -> Self {
        Self {
            namespace: String::from("fraktor.cluster"),
            enabled: true,
        }
    }
}
