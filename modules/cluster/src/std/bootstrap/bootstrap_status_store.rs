//! Trait and helpers for bootstrap status persistence.

use crate::std::bootstrap::{BootstrapState, BootstrapStatusError};

/// Persistence contract for cluster bootstrap status.
pub trait BootstrapStatusStore: Send + Sync {
  /// Load the persisted bootstrap state.
  fn load(&self) -> Result<BootstrapState, BootstrapStatusError>;

  /// Persist the provided bootstrap state.
  fn save(&self, state: &BootstrapState) -> Result<(), BootstrapStatusError>;
}
