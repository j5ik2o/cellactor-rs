//! Configuration for cluster bootstrap.

extern crate alloc;
extern crate std;

use alloc::{string::String, sync::Arc};

use crate::std::bootstrap::BootstrapStatusStore;

/// Immutable configuration for cluster bootstrap.
pub struct ClusterBootstrapConfig {
  enabled:      bool,
  status_store: Arc<dyn BootstrapStatusStore>,
  validator:    Option<Arc<dyn Fn() -> Result<(), String> + Send + Sync>>, /* TODO: replace with concrete validation
                                                                            * inputs in later tasks */
}

impl ClusterBootstrapConfig {
  /// Create a new config with the given status store.
  #[must_use]
  pub fn new(status_store: Arc<dyn BootstrapStatusStore>) -> Self {
    Self { enabled: true, status_store, validator: None }
  }

  /// Enable or disable the bootstrap execution.
  #[must_use]
  pub fn with_enabled(mut self, enabled: bool) -> Self {
    self.enabled = enabled;
    self
  }

  /// Inject a validation closure that runs before bootstrap.
  #[must_use]
  pub fn with_validator(mut self, validator: Arc<dyn Fn() -> Result<(), String> + Send + Sync>) -> Self {
    self.validator = Some(validator);
    self
  }

  /// Returns whether bootstrap is enabled.
  #[must_use]
  pub fn enabled(&self) -> bool {
    self.enabled
  }

  /// Returns the status store.
  #[must_use]
  pub fn status_store(&self) -> &Arc<dyn BootstrapStatusStore> {
    &self.status_store
  }

  /// Executes validator if configured.
  pub fn validate(&self) -> Result<(), String> {
    if let Some(validator) = &self.validator {
      return validator();
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests;
