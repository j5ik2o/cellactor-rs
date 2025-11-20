use alloc::vec::Vec;

use fraktor_utils_rs::core::{
  runtime_toolbox::{RuntimeToolbox, SyncMutexFamily, ToolboxMutex},
  sync::sync_mutex_like::SyncMutexLike,
};

use crate::core::events::cluster_event::ClusterEvent;

#[cfg(test)]
mod tests;

/// Lightweight publisher that queues cluster events before flushing to EventStream.
pub struct ClusterEventPublisher<TB>
where
  TB: RuntimeToolbox, {
  queue: ToolboxMutex<Vec<ClusterEvent>, TB>,
}

impl<TB> ClusterEventPublisher<TB>
where
  TB: RuntimeToolbox,
{
  /// Creates a new event publisher with an empty queue.
  #[must_use]
  pub fn new() -> Self {
    Self { queue: <TB::MutexFamily as SyncMutexFamily>::create(Vec::new()) }
  }

  /// Enqueues an event for later flushing.
  pub fn enqueue(&self, event: ClusterEvent) {
    self.queue.lock().push(event);
  }

  /// Drains all pending events.
  pub fn drain(&self) -> Vec<ClusterEvent> {
    self.queue.lock().drain(..).collect()
  }
}

impl<TB> Default for ClusterEventPublisher<TB>
where
  TB: RuntimeToolbox,
{
  fn default() -> Self {
    Self::new()
  }
}
