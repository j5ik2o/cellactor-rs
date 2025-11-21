//! Structured error codes for provisioning.

extern crate alloc;

use alloc::string::String;

/// Provisioning error categories.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProvisioningErrorCode {
  /// Validation failure such as missing required field.
  Validation,
  /// Connectivity to external backend failed.
  Connectivity,
  /// Provider stream interrupted or terminated unexpectedly.
  StreamFailure,
  /// Persistence failure while loading or saving providers.
  Persistence,
  /// Duplicate registration detected.
  Duplicate,
}

/// Provisioning error with structured code.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("{message}")]
pub struct ProvisioningError {
  /// Error code for programmatic handling.
  pub code:    ProvisioningErrorCode,
  /// Human-readable description.
  pub message: String,
}

impl ProvisioningError {
  /// Create a new provisioning error.
  #[must_use]
  pub fn new(code: ProvisioningErrorCode, message: impl Into<String>) -> Self {
    Self { code, message: message.into() }
  }
}

#[cfg(test)]
mod tests;
