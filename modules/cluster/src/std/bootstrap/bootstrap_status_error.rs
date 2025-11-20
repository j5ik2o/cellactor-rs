//! Error type for bootstrap status persistence.

extern crate alloc;
extern crate std;

use alloc::string::String;
use core::fmt::{Display, Formatter, Result as FmtResult};

/// Error returned when reading or writing bootstrap status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapStatusError {
  /// Loading the bootstrap state failed.
  LoadFailed(String),
  /// Saving the bootstrap state failed.
  SaveFailed(String),
}

impl Display for BootstrapStatusError {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    match self {
      | BootstrapStatusError::LoadFailed(reason) => write!(f, "failed to load bootstrap state: {reason}"),
      | BootstrapStatusError::SaveFailed(reason) => write!(f, "failed to save bootstrap state: {reason}"),
    }
  }
}

impl std::error::Error for BootstrapStatusError {}
