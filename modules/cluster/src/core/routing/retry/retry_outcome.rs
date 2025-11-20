use core::time::Duration;

/// Result of evaluating a retry attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetryOutcome {
  /// Perform the retry after the provided backoff delay.
  RetryAfter(Duration),
  /// Exhausted retry attempts.
  GiveUp,
}
