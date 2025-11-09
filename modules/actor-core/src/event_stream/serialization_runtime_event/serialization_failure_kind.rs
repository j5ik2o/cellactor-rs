use core::fmt;

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
