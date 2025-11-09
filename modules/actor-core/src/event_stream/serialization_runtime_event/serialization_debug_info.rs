use alloc::string::String;

/// Additional context published when debug tracing is enabled.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerializationDebugInfo {
  manifest:   String,
  size_bytes: u32,
}

impl SerializationDebugInfo {
  /// Creates a new debug entry using the provided manifest and payload size.
  #[must_use]
  pub fn new(manifest: String, size_bytes: u32) -> Self {
    Self { manifest, size_bytes }
  }

  /// Returns the serialized manifest associated with the field.
  #[must_use]
  pub fn manifest(&self) -> &str {
    &self.manifest
  }

  /// Returns the payload size in bytes.
  #[must_use]
  pub const fn size_bytes(&self) -> u32 {
    self.size_bytes
  }
}
