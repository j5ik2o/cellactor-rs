use alloc::vec::Vec;
use std::sync::Mutex;

use fraktor_actor_rs::core::event_stream::{EventStreamEvent, EventStreamGeneric, EventStreamSubscriber};
use fraktor_utils_rs::core::{runtime_toolbox::NoStdToolbox, sync::ArcShared};

use crate::core::{
  activation::{ActivationLease, LeaseId, LeaseStatus},
  events::{ClusterEvent, ClusterEventPublisher, ClusterEventStreamAdapter},
  identity::{ClusterIdentity, NodeId},
};

struct RecordingSubscriber {
  events: ArcShared<Mutex<Vec<EventStreamEvent<NoStdToolbox>>>>,
}

impl RecordingSubscriber {
  fn new(events: ArcShared<Mutex<Vec<EventStreamEvent<NoStdToolbox>>>>) -> Self {
    Self { events }
  }
}

impl EventStreamSubscriber<NoStdToolbox> for RecordingSubscriber {
  fn on_event(&self, event: &EventStreamEvent<NoStdToolbox>) {
    self.events.lock().unwrap().push(event.clone());
  }
}

#[test]
fn publishes_cluster_events_as_logs() {
  let event_stream = ArcShared::new(EventStreamGeneric::<NoStdToolbox>::default());
  let adapter = ClusterEventStreamAdapter::new(event_stream.clone());
  let publisher = ClusterEventPublisher::<NoStdToolbox>::new();

  publisher.enqueue(ClusterEvent::ActivationStarted {
    identity: ClusterIdentity::new("echo", "a"),
    owner:    NodeId::new("node-a"),
  });
  let lease = ActivationLease::new(LeaseId::new(1), NodeId::new("node-b"), 7, LeaseStatus::Active);
  publisher.enqueue(ClusterEvent::ActivationTerminated { identity: ClusterIdentity::new("echo", "b"), lease });

  let events = ArcShared::new(Mutex::new(Vec::new()));
  let subscriber: ArcShared<dyn EventStreamSubscriber<NoStdToolbox>> =
    ArcShared::new(RecordingSubscriber::new(events.clone()));
  let _subscription = EventStreamGeneric::subscribe_arc(&event_stream, &subscriber);

  adapter.flush(&publisher);

  let recorded = events.lock().unwrap().clone();
  assert_eq!(recorded.len(), 2);
  assert!(
    recorded
      .iter()
      .any(|event| matches!(event, EventStreamEvent::Log(log) if log.message().contains("activation_started")))
  );
  assert!(
    recorded
      .iter()
      .any(|event| matches!(event, EventStreamEvent::Log(log) if log.message().contains("activation_terminated")))
  );
}
