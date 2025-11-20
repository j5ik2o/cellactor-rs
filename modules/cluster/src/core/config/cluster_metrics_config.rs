use alloc::string::String;

/// Configuration for metrics export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterMetricsConfig {
  namespace: String,
  enabled:   bool,
}

impl ClusterMetricsConfig {
  /// Creates a new metrics configuration entry.
  #[must_use]
  pub const fn new(namespace: String, enabled: bool) -> Self {
    Self { namespace, enabled }
  }

  /// Returns the configured namespace for metrics instruments.
  #[must_use]
  pub fn namespace(&self) -> &str {
    &self.namespace
  }

  /// Whether metrics publishing is enabled.
  #[must_use]
  pub const fn enabled(&self) -> bool {
    self.enabled
  }
}

impl Default for ClusterMetricsConfig {
  fn default() -> Self {
    Self { namespace: String::from("fraktor.cluster"), enabled: true }
  }
}
