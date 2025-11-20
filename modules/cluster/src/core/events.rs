//! Cluster-specific events flowing through EventStream.

/// Event payload shared with EventStream subscribers.
pub mod cluster_event;
/// EventStream adapter for ClusterEvent.
pub mod event_publisher;
/// Adapter that bridges ClusterEvent to the actor EventStream.
pub mod event_stream_adapter;

pub use cluster_event::ClusterEvent;
pub use event_publisher::ClusterEventPublisher;
pub use event_stream_adapter::ClusterEventStreamAdapter;
