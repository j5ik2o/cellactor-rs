use alloc::{string::String, sync::Arc};

use crate::std::bootstrap::{BootstrapState, ClusterBootstrapConfig, InMemoryBootstrapStatusStore};

#[test]
fn enabled_flag_is_respected() {
  let store = Arc::new(InMemoryBootstrapStatusStore::new(BootstrapState::Ready));

  let config = ClusterBootstrapConfig::new(store).with_enabled(false);

  assert!(!config.enabled());
}

#[test]
fn validator_is_invoked() {
  let store = Arc::new(InMemoryBootstrapStatusStore::new(BootstrapState::Ready));
  let validator_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
  let flag = Arc::clone(&validator_called);

  let validator = Arc::new(move || {
    flag.store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
  });

  let config = ClusterBootstrapConfig::new(store).with_validator(validator);

  let result = config.validate();

  assert!(result.is_ok());
  assert!(validator_called.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn validator_failure_is_returned() {
  let store = Arc::new(InMemoryBootstrapStatusStore::new(BootstrapState::Ready));
  let validator = Arc::new(|| Err(String::from("invalid")));
  let config = ClusterBootstrapConfig::new(store).with_validator(validator);

  let result = config.validate();

  assert_eq!(Err(String::from("invalid")), result);
}
