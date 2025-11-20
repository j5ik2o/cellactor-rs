//! In-memory implementation of the bootstrap status store.

extern crate alloc;
extern crate std;

use alloc::string::ToString;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::std::bootstrap::{BootstrapState, BootstrapStatusError, BootstrapStatusStore};

/// Thread-safe in-memory bootstrap status store using `RwLock`.
#[derive(Debug)]
pub struct InMemoryBootstrapStatusStore {
  state: RwLock<BootstrapState>,
}

impl InMemoryBootstrapStatusStore {
  /// Create a new store with the provided initial state.
  #[must_use]
  pub fn new(initial_state: BootstrapState) -> Self {
    Self { state: RwLock::new(initial_state) }
  }

  fn read_state(&self) -> Result<RwLockReadGuard<'_, BootstrapState>, BootstrapStatusError> {
    self.state.read().map_err(|error| BootstrapStatusError::LoadFailed(error.to_string()))
  }

  fn write_state(&self) -> Result<RwLockWriteGuard<'_, BootstrapState>, BootstrapStatusError> {
    self.state.write().map_err(|error| BootstrapStatusError::SaveFailed(error.to_string()))
  }
}

impl BootstrapStatusStore for InMemoryBootstrapStatusStore {
  fn load(&self) -> Result<BootstrapState, BootstrapStatusError> {
    let state = self.read_state()?;
    Ok(state.clone())
  }

  fn save(&self, state: &BootstrapState) -> Result<(), BootstrapStatusError> {
    let mut guard = self.write_state()?;
    *guard = state.clone();
    Ok(())
  }
}

#[cfg(test)]
mod tests;
