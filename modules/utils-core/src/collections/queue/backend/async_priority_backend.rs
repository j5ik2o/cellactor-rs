use super::AsyncQueueBackend;
use crate::collections::PriorityMessage;

/// Extension trait for async backends supporting priority semantics.
pub trait AsyncPriorityBackend<T: PriorityMessage>: AsyncQueueBackend<T> {
  /// Returns a reference to the smallest element without removing it.
  fn peek_min(&self) -> Option<&T>;
}
