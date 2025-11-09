//! Builder-friendly options for aggregate fields.

use super::envelope_mode::EnvelopeMode;

/// Options used when registering a field inside an aggregate schema.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FieldOptions {
  envelope_mode:               EnvelopeMode,
  external_serializer_allowed: bool,
}

impl FieldOptions {
  /// Creates a new set of options with the desired envelope mode.
  #[must_use]
  pub const fn new(envelope_mode: EnvelopeMode) -> Self {
    Self { envelope_mode, external_serializer_allowed: false }
  }

  /// Marks the field as delegating to an external serializer when allowed.
  #[must_use]
  pub const fn with_external_serializer_allowed(mut self, allowed: bool) -> Self {
    self.external_serializer_allowed = allowed;
    self
  }

  /// Returns the configured envelope mode.
  #[must_use]
  pub const fn envelope_mode(&self) -> EnvelopeMode {
    self.envelope_mode
  }

  /// Returns whether external serializers are permitted.
  #[must_use]
  pub const fn external_serializer_allowed(&self) -> bool {
    self.external_serializer_allowed
  }
}
