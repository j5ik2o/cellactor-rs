//! Provides read-only access to serialization audit results.

use cellactor_utils_core_rs::sync::ArcShared;

use crate::{RuntimeToolbox, event_stream::SerializationAuditEvent, system::system_state::SystemStateGeneric};

/// Exposes serialization audit state for monitoring endpoints.
pub struct SerializationAuditMonitor<TB: RuntimeToolbox + 'static> {
  state: ArcShared<SystemStateGeneric<TB>>,
}

impl<TB: RuntimeToolbox + 'static> SerializationAuditMonitor<TB> {
  /// Creates a new monitor backed by the provided system state.
  #[must_use]
  pub const fn new(state: ArcShared<SystemStateGeneric<TB>>) -> Self {
    Self { state }
  }

  /// Returns the last published serialization audit event, if any.
  #[must_use]
  pub fn last_event(&self) -> Option<SerializationAuditEvent> {
    self.state.last_serialization_audit()
  }

  /// Indicates whether audit execution is currently enabled.
  #[must_use]
  pub fn audit_enabled(&self) -> bool {
    self.state.serialization_audit_enabled()
  }
}
