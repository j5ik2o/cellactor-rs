//! Cluster-specific events flowing through EventStream.

/// Event payload shared with EventStream subscribers.
pub mod cluster_event;
/// EventStream adapter for ClusterEvent.
pub mod event_publisher;

pub use cluster_event::ClusterEvent;
pub use event_publisher::ClusterEventPublisher;
