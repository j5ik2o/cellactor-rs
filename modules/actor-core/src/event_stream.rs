//! Event stream package.
//!
//! This module contains event publishing and subscription.

mod base;
mod event_stream_event;
mod event_stream_subscriber;
mod event_stream_subscriber_entry;
mod event_stream_subscription;
mod serialization_event;
mod serialization_runtime_event;

pub use base::{EventStream, EventStreamGeneric};
pub use event_stream_event::EventStreamEvent;
pub use event_stream_subscriber::EventStreamSubscriber;
pub use event_stream_subscriber_entry::{EventStreamSubscriberEntry, EventStreamSubscriberEntryGeneric};
pub use event_stream_subscription::{EventStreamSubscription, EventStreamSubscriptionGeneric};
pub use serialization_event::{SerializationAuditEvent, SerializationAuditIssue};
pub use serialization_runtime_event::{SerializationEvent, SerializationEventKind, SerializationFailureKind};
