//! Trait describing how telemetry backends consume serialization audit events.

use crate::{RuntimeToolbox, event_stream::SerializationAuditEvent};

/// Records serialization audit events for external telemetry systems.
pub trait SerializationAuditNotifier<TB: RuntimeToolbox + 'static>: Send + Sync {
  /// Called whenever the actor system completes a serialization registry audit.
  fn on_serialization_audit(&self, event: &SerializationAuditEvent);
}
