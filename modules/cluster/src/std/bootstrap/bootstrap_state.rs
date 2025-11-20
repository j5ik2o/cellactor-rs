//! Bootstrap lifecycle state for the cluster extension.

extern crate alloc;

use alloc::string::String;

/// Represents the readiness of the cluster bootstrap pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapState {
  /// Extension is ready and was installed successfully.
  Ready,
  /// Extension is intentionally disabled by configuration.
  Disabled,
  /// Extension failed to install; reason describes the failure.
  Error {
    /// Human-readable description of the failure.
    reason: String,
  },
}
