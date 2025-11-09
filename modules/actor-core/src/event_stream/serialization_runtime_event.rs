//! Runtime serialization telemetry event structures.

use core::fmt;

use crate::serialization::FieldPathHash;

/// Event emitted by the serialization telemetry system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SerializationEvent {
  field_path_hash: FieldPathHash,
  kind:            SerializationEventKind,
}

impl SerializationEvent {
  /// Creates a new event for the provided field path hash and kind.
  #[must_use]
  pub const fn new(field_path_hash: FieldPathHash, kind: SerializationEventKind) -> Self {
    Self { field_path_hash, kind }
  }

  /// Returns the associated field path hash.
  #[must_use]
  pub const fn field_path_hash(&self) -> FieldPathHash {
    self.field_path_hash
  }

  /// Returns the event kind.
  #[must_use]
  pub const fn kind(&self) -> &SerializationEventKind {
    &self.kind
  }
}

/// Describes the specific telemetry observation emitted for serialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SerializationEventKind {
  /// Field serialization completed successfully.
  Success,
  /// Field serialization failed.
  Failure(SerializationFailureKind),
  /// Field serialization latency observation (microseconds).
  Latency(u64),
}

/// Canonical failure reasons surfaced through telemetry events.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SerializationFailureKind {
  /// No serializer binding exists for the field type.
  MissingSerializer,
  /// Aggregate schema definition is invalid.
  InvalidAggregate,
  /// Serializer reported a recoverable failure.
  SerializationFailed,
  /// Deserializer failed to decode the payload.
  DeserializationFailed,
  /// Any other failure reason.
  Other,
}

impl fmt::Display for SerializationFailureKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      | Self::MissingSerializer => write!(f, "missing_serializer"),
      | Self::InvalidAggregate => write!(f, "invalid_aggregate"),
      | Self::SerializationFailed => write!(f, "serialization_failed"),
      | Self::DeserializationFailed => write!(f, "deserialization_failed"),
      | Self::Other => write!(f, "other"),
    }
  }
}
