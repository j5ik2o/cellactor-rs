//! Provider health status.

/// Provider health status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderHealth {
  /// Healthy provider delivering snapshots.
  Healthy,
  /// Degraded but still delivering snapshots.
  Degraded,
  /// Unreachable provider.
  Unreachable,
}
