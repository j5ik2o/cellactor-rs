use core::time::Duration;

use crate::core::{
  config::{RetryJitter, RetryPolicy},
  routing::retry::RetryOutcome,
};

/// Backoff iterator derived from a retry policy.
pub struct RetryPolicyRunner {
  policy:  RetryPolicy,
  attempt: u32,
  backoff: Duration,
}

impl RetryPolicyRunner {
  /// Creates a new runner for the given policy.
  #[must_use]
  pub const fn new(policy: RetryPolicy) -> Self {
    Self { backoff: policy.initial_backoff(), attempt: 0, policy }
  }

  /// Evaluates the next retry decision.
  pub fn next_outcome(&mut self) -> RetryOutcome {
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

impl Iterator for RetryPolicyRunner {
  type Item = RetryOutcome;

  fn next(&mut self) -> Option<Self::Item> {
    Some(self.next_outcome())
  }
}
