//! No-op telemetry handler used when no backend is configured.

use crate::{
  RuntimeToolbox, event_stream::SerializationAuditEvent,
  telemetry::serialization_audit_notifier::SerializationAuditNotifier,
};

/// Telemetry handler that discards all serialization audit events.
pub struct NoopSerializationAuditNotifier;

impl NoopSerializationAuditNotifier {
  /// Creates a new no-op telemetry handler.
  #[must_use]
  pub const fn new() -> Self {
    Self
  }
}

impl<TB: RuntimeToolbox + 'static> SerializationAuditNotifier<TB> for NoopSerializationAuditNotifier {
  fn on_serialization_audit(&self, _event: &SerializationAuditEvent) {}
}
