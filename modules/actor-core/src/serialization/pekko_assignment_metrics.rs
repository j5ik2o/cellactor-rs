//! Metrics describing Pekko serializer assignments.

/// Aggregated assignment counters for Pekko-compatible serializers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PekkoAssignmentMetrics {
  /// Number of successful automatic assignments.
  pub success_total: u64,
  /// Number of failed assignments (e.g., manifest collisions).
  pub failure_total: u64,
}
