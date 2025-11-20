use core::time::Duration;

use crate::core::config::{RetryJitter, RetryPolicy};
use crate::core::routing::retry::{RetryOutcome, RetryPolicyRunner};

fn policy() -> RetryPolicy {
    RetryPolicy::new(
        core::num::NonZeroU32::new(3).unwrap(),
        Duration::from_millis(50),
        Duration::from_millis(200),
        RetryJitter::None,
    )
}

#[test]
fn runner_retries_until_max_attempts() {
    let mut runner = RetryPolicyRunner::new(policy());
    assert!(matches!(runner.next(), RetryOutcome::RetryAfter(_)));
    assert!(matches!(runner.next(), RetryOutcome::RetryAfter(_)));
    assert!(matches!(runner.next(), RetryOutcome::RetryAfter(_)));
    assert!(matches!(runner.next(), RetryOutcome::GiveUp));
}
