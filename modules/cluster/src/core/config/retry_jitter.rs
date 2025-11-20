/// Type of jitter applied to retry delays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RetryJitter {
  /// No jitter, deterministic exponential backoff.
  None,
  /// Full jitter per AWS Architecture best practice.
  #[default]
  Full,
  /// Decorrelated jitter variant.
  Decorrelated,
}
