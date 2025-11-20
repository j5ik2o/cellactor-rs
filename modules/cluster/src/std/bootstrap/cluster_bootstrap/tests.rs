use alloc::{string::String, sync::Arc};

use crate::std::bootstrap::{
  BootstrapState, BootstrapStatusError, BootstrapStatusStore, ClusterBootstrap, ClusterBootstrapConfig,
  ClusterBootstrapError, InMemoryBootstrapStatusStore,
};

#[test]
fn disabled_config_skips_bootstrap_and_saves_disabled() {
  let store = Arc::new(InMemoryBootstrapStatusStore::new(BootstrapState::Ready));
  let config = ClusterBootstrapConfig::new(store.clone()).with_enabled(false);

  let result = ClusterBootstrap::install(config).expect("disabled bootstrap should succeed");

  assert_eq!(BootstrapState::Disabled, *result.state());
  assert_eq!(Ok(BootstrapState::Disabled), store.load());
}

#[test]
fn validator_failure_returns_error_and_persists_error_state() {
  let store = Arc::new(InMemoryBootstrapStatusStore::new(BootstrapState::Disabled));
  let validator = Arc::new(|| Err(String::from("missing topology")));
  let config = ClusterBootstrapConfig::new(store.clone()).with_validator(validator);

  let result = ClusterBootstrap::install(config);

  assert!(matches!(result, Err(ClusterBootstrapError::InvalidConfig { reason }) if reason == "missing topology"));
  assert_eq!(Ok(BootstrapState::Error { reason: String::from("missing topology") }), store.load());
}

#[test]
fn successful_install_sets_ready_state() {
  let store = Arc::new(InMemoryBootstrapStatusStore::new(BootstrapState::Disabled));
  let config = ClusterBootstrapConfig::new(store.clone());

  let result = ClusterBootstrap::install(config).expect("bootstrap should succeed");

  assert_eq!(BootstrapState::Ready, *result.state());
  assert_eq!(Ok(BootstrapState::Ready), store.load());
}

#[test]
fn load_failure_returns_status_load_error() {
  struct FailingStore;

  impl BootstrapStatusStore for FailingStore {
    fn load(&self) -> Result<BootstrapState, BootstrapStatusError> {
      Err(BootstrapStatusError::LoadFailed(String::from("boom")))
    }

    fn save(&self, _state: &BootstrapState) -> Result<(), BootstrapStatusError> {
      Ok(())
    }
  }

  let store = Arc::new(FailingStore);
  let config = ClusterBootstrapConfig::new(store);

  let result = ClusterBootstrap::install(config);

  assert!(matches!(result, Err(ClusterBootstrapError::StatusLoadFailed(_))));
}

#[test]
fn save_failure_returns_status_save_error() {
  struct SaveFailStore;

  impl BootstrapStatusStore for SaveFailStore {
    fn load(&self) -> Result<BootstrapState, BootstrapStatusError> {
      Ok(BootstrapState::Ready)
    }

    fn save(&self, _state: &BootstrapState) -> Result<(), BootstrapStatusError> {
      Err(BootstrapStatusError::SaveFailed(String::from("persist failed")))
    }
  }

  let store = Arc::new(SaveFailStore);
  let config = ClusterBootstrapConfig::new(store);

  let result = ClusterBootstrap::install(config);

  assert!(matches!(result, Err(ClusterBootstrapError::StatusSaveFailed(_))));
}
