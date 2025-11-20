//! Orchestrates cluster bootstrap using provided configuration.

extern crate alloc;
extern crate std;

use alloc::sync::Arc;

use crate::std::bootstrap::{BootstrapState, ClusterBootstrapConfig, ClusterBootstrapError, ClusterExtensionHandle};

/// Entry point for installing the cluster extension into an ActorSystem.
pub struct ClusterBootstrap;

impl ClusterBootstrap {
  /// Install cluster extension based on the given configuration.
  pub fn install(config: ClusterBootstrapConfig) -> Result<ClusterExtensionHandle, ClusterBootstrapError> {
    let store = Arc::clone(config.status_store());

    let _current = store.load().map_err(ClusterBootstrapError::StatusLoadFailed)?;

    if !config.enabled() {
      let disabled = BootstrapState::Disabled;
      store.save(&disabled).map_err(ClusterBootstrapError::StatusSaveFailed)?;
      return Ok(ClusterExtensionHandle::new(disabled));
    }

    if let Err(reason) = config.validate() {
      let error_state = BootstrapState::Error { reason: reason.clone() };
      store.save(&error_state).map_err(ClusterBootstrapError::StatusSaveFailed)?;
      return Err(ClusterBootstrapError::InvalidConfig { reason });
    }

    let ready_state = BootstrapState::Ready;
    let status_result = store.save(&ready_state);
    match status_result {
      | Ok(()) => Ok(ClusterExtensionHandle::new(ready_state)),
      | Err(error) => Err(ClusterBootstrapError::StatusSaveFailed(error)),
    }
  }
}

#[cfg(test)]
mod tests;
