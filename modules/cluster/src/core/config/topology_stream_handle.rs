use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(test)]
mod tests;

/// Detects termination of a topology stream.
#[derive(Debug, Default)]
pub struct TopologyStreamHandle {
  closed: AtomicBool,
}

impl TopologyStreamHandle {
  /// Creates a new open handle.
  #[must_use]
  pub const fn new() -> Self {
    Self { closed: AtomicBool::new(false) }
  }

  /// Marks the stream as closed. Returns previous closed state.
  pub fn mark_closed(&self) -> bool {
    self.closed.swap(true, Ordering::SeqCst)
  }

  /// Returns true if the stream has been marked closed.
  #[must_use]
  pub fn is_closed(&self) -> bool {
    self.closed.load(Ordering::SeqCst)
  }
}
