/// Type of jitter applied to retry delays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryJitter {
    /// No jitter, deterministic exponential backoff.
    None,
    /// Full jitter per AWS Architecture best practice.
    Full,
    /// Decorrelated jitter variant.
    Decorrelated,
}

impl Default for RetryJitter {
    fn default() -> Self {
        Self::Full
    }
}
