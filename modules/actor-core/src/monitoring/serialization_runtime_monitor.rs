//! Provides read access to serialization runtime telemetry snapshots.

use cellactor_utils_core_rs::sync::ArcShared;

use crate::{RuntimeToolbox, event_stream::SerializationEvent, system::SystemStateGeneric};

/// Exposes serialization telemetry observations for monitoring backends.
pub struct SerializationRuntimeMonitor<TB: RuntimeToolbox + 'static> {
  state: ArcShared<SystemStateGeneric<TB>>,
}

impl<TB: RuntimeToolbox + 'static> SerializationRuntimeMonitor<TB> {
  /// Creates a new monitor backed by the provided system state.
  #[must_use]
  pub const fn new(state: ArcShared<SystemStateGeneric<TB>>) -> Self {
    Self { state }
  }

  /// Returns the last recorded binding error event, if any.
  #[must_use]
  pub fn last_binding_error(&self) -> Option<SerializationEvent> {
    self.state.last_serialization_binding_error()
  }
}
