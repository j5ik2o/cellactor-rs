use super::{SerializationDebugInfo, SerializationFailureKind, SerializationFallbackReason};

/// Describes the specific telemetry observation emitted for serialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SerializationEventKind {
  /// Field serialization completed successfully.
  Success,
  /// Field serialization failed.
  Failure(SerializationFailureKind),
  /// Field serialization latency observation (microseconds).
  Latency(u64),
  /// Serialization fell back to a secondary path.
  Fallback(SerializationFallbackReason),
  /// Debug trace entry containing manifest and size.
  DebugTrace(SerializationDebugInfo),
}
