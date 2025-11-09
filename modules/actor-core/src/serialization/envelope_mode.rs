//! Describes how a field should be wrapped into parent envelopes.

/// Envelope handling behavior for nested fields.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EnvelopeMode {
  /// Preserve Pekko-compatible ordering and headers.
  PreserveOrder,
}
