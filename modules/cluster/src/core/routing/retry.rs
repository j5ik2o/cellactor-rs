use core::time::Duration;

use crate::core::config::{RetryJitter, RetryPolicy};

/// Result of evaluating a retry attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetryOutcome {
  /// Perform the retry after the provided backoff delay.
  RetryAfter(Duration),
  /// Exhausted retry attempts.
  GiveUp,
}

/// Backoff iterator derived from a retry policy.
pub struct RetryPolicyRunner {
  policy:  RetryPolicy,
  attempt: u32,
  backoff: Duration,
}

impl RetryPolicyRunner {
  /// Creates a new runner for the given policy.
  pub fn new(policy: RetryPolicy) -> Self {
    Self { backoff: policy.initial_backoff(), attempt: 0, policy }
  }

  /// Evaluates the next retry decision.
  pub fn next(&mut self) -> RetryOutcome {
    self.attempt += 1;
    if self.attempt > self.policy.max_attempts().get() {
      return RetryOutcome::GiveUp;
    }
    let delay = self.apply_jitter(self.backoff);
    self.backoff = core::cmp::min(self.backoff * 2, self.policy.max_backoff());
    RetryOutcome::RetryAfter(delay)
  }

  fn apply_jitter(&self, backoff: Duration) -> Duration {
    match self.policy.jitter() {
      | RetryJitter::None => backoff,
      | RetryJitter::Full => backoff / 2,
      | RetryJitter::Decorrelated => backoff + Duration::from_millis(10),
    }
  }
}
