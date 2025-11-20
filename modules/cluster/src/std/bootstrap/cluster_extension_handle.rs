//! Handle bundle returned after successful bootstrap.

use crate::std::bootstrap::BootstrapState;

/// Provides read-only access to the bootstrap result and handles.
pub struct ClusterExtensionHandle {
  state: BootstrapState,
}

impl ClusterExtensionHandle {
  /// Create a new handle from the given bootstrap state.
  #[must_use]
  pub fn new(state: BootstrapState) -> Self {
    Self { state }
  }

  /// Returns the bootstrap state.
  #[must_use]
  pub fn state(&self) -> &BootstrapState {
    &self.state
  }
}

#[cfg(test)]
mod tests;
