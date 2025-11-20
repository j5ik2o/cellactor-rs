//! Errors emitted during cluster bootstrap orchestration.

extern crate alloc;
extern crate std;

use alloc::string::String;

use crate::std::bootstrap::BootstrapStatusError;

/// Error returned by `ClusterBootstrap::install`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterBootstrapError {
  /// Loading bootstrap status failed.
  StatusLoadFailed(BootstrapStatusError),
  /// Persisting bootstrap status failed.
  StatusSaveFailed(BootstrapStatusError),
  /// Provided configuration is invalid.
  InvalidConfig {
    /// Human-readable reason describing why validation failed.
    reason: String,
  },
}

impl core::fmt::Display for ClusterBootstrapError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      | ClusterBootstrapError::StatusLoadFailed(error) => {
        write!(f, "failed to load bootstrap status: {error}")
      },
      | ClusterBootstrapError::StatusSaveFailed(error) => {
        write!(f, "failed to save bootstrap status: {error}")
      },
      | ClusterBootstrapError::InvalidConfig { reason } => {
        write!(f, "invalid bootstrap config: {reason}")
      },
    }
  }
}

impl std::error::Error for ClusterBootstrapError {}
