use super::SerializationEventKind;
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
