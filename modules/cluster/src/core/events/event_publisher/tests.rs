use fraktor_utils_rs::core::runtime_toolbox::NoStdToolbox;

use crate::core::events::{ClusterEvent, ClusterEventPublisher};
use crate::core::identity::{ClusterIdentity, NodeId};

#[test]
fn enqueues_and_drains_events() {
    let publisher = ClusterEventPublisher::<NoStdToolbox>::new();
    publisher.enqueue(ClusterEvent::BlockListApplied { node: NodeId::new("node-a") });
    publisher.enqueue(ClusterEvent::ActivationStarted {
        identity: ClusterIdentity::new("echo", "a"),
        owner: NodeId::new("node-a"),
    });

    let events = publisher.drain();

    assert_eq!(events.len(), 2);
    assert!(publisher.drain().is_empty());
}
