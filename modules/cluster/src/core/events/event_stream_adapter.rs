use alloc::{format, string::String};
use core::time::Duration;

use fraktor_actor_rs::core::{
  event_stream::{EventStreamEvent, EventStreamGeneric},
  logging::{LogEvent, LogLevel},
};
use fraktor_utils_rs::core::{runtime_toolbox::RuntimeToolbox, sync::ArcShared};

use crate::core::{
  events::{ClusterEvent, ClusterEventPublisher},
  identity::ClusterIdentity,
};

#[cfg(test)]
mod tests;

/// Bridges clustered ClusterEvent to actor event stream.
pub struct ClusterEventStreamAdapter<TB>
where
  TB: RuntimeToolbox + 'static, {
  event_stream: ArcShared<EventStreamGeneric<TB>>,
}

impl<TB> ClusterEventStreamAdapter<TB>
where
  TB: RuntimeToolbox + 'static,
{
  /// Creates a new adapter backed by the provided event stream.
  #[must_use]
  pub const fn new(event_stream: ArcShared<EventStreamGeneric<TB>>) -> Self {
    Self { event_stream }
  }

  /// Drains publisher queue and publishes each event to the event stream.
  pub fn flush(&self, publisher: &ClusterEventPublisher<TB>) {
    for event in publisher.drain() {
      let stream_event = Self::to_log_event(&event);
      self.event_stream.publish(&stream_event);
    }
  }

  fn to_log_event(event: &ClusterEvent) -> EventStreamEvent<TB> {
    let (level, message) = match event {
      | ClusterEvent::ActivationStarted { identity, owner } => (
        LogLevel::Info,
        format!("cluster.activation_started identity={} owner={}", fmt_identity(identity), owner.as_str()),
      ),
      | ClusterEvent::ActivationTerminated { identity, lease } => (
        LogLevel::Info,
        format!(
          "cluster.activation_terminated identity={} lease={} owner={} hash={}",
          fmt_identity(identity),
          lease.lease_id().get(),
          lease.owner().as_str(),
          lease.topology_hash()
        ),
      ),
      | ClusterEvent::BlockListApplied { node } => {
        (LogLevel::Warn, format!("cluster.block_list_applied node={}", node.as_str()))
      },
      | ClusterEvent::RetryThrottled { identity } => {
        (LogLevel::Warn, format!("cluster.retry_throttled identity={}", fmt_identity(identity)))
      },
    };

    let log = LogEvent::new(level, message, Duration::ZERO, None);
    EventStreamEvent::Log(log)
  }
}

fn fmt_identity(identity: &ClusterIdentity) -> String {
  format!("{}/{}", identity.kind(), identity.identity())
}
