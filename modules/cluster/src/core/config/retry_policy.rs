use core::{num::NonZeroU32, time::Duration};

use super::retry_jitter::RetryJitter;

/// Retry policy used by `ClusterContext` when requests fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
  max_attempts:    NonZeroU32,
  initial_backoff: Duration,
  max_backoff:     Duration,
  jitter:          RetryJitter,
}

impl RetryPolicy {
  /// Creates a new retry policy description.
  #[must_use]
  pub const fn new(
    max_attempts: NonZeroU32,
    initial_backoff: Duration,
    max_backoff: Duration,
    jitter: RetryJitter,
  ) -> Self {
    Self { max_attempts, initial_backoff, max_backoff, jitter }
  }

  /// Maximum attempts allowed for a single request.
  #[must_use]
  pub const fn max_attempts(&self) -> NonZeroU32 {
    self.max_attempts
  }

  /// Initial delay before the first retry.
  #[must_use]
  pub const fn initial_backoff(&self) -> Duration {
    self.initial_backoff
  }

  /// Maximum backoff allowed.
  #[must_use]
  pub const fn max_backoff(&self) -> Duration {
    self.max_backoff
  }

  /// Jitter strategy applied to backoff durations.
  #[must_use]
  pub const fn jitter(&self) -> RetryJitter {
    self.jitter
  }
}

impl Default for RetryPolicy {
  fn default() -> Self {
    // SAFETY: 3 is non-zero
    let max_attempts = unsafe { NonZeroU32::new_unchecked(3) };
    Self {
      max_attempts,
      initial_backoff: Duration::from_millis(50),
      max_backoff: Duration::from_secs(2),
      jitter: RetryJitter::Full,
    }
  }
}
