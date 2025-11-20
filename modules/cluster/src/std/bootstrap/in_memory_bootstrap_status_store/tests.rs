use alloc::string::String;
use std::sync::Arc;

use crate::std::bootstrap::{BootstrapState, BootstrapStatusStore, InMemoryBootstrapStatusStore};

#[test]
fn load_returns_initial_state() {
  let store = InMemoryBootstrapStatusStore::new(BootstrapState::Disabled);

  let result = store.load();

  assert_eq!(Ok(BootstrapState::Disabled), result);
}

#[test]
fn save_persists_new_state() {
  let store = InMemoryBootstrapStatusStore::new(BootstrapState::Ready);

  let saved = BootstrapState::Error { reason: String::from("init failed") };
  store.save(&saved).expect("save state");

  let loaded = store.load();

  assert_eq!(Ok(saved), loaded);
}

#[test]
fn load_and_save_are_thread_safe() {
  let store = Arc::new(InMemoryBootstrapStatusStore::new(BootstrapState::Ready));
  let store_for_thread = Arc::clone(&store);

  let join = std::thread::spawn(move || {
    let next = BootstrapState::Error { reason: String::from("worker failed") };
    store_for_thread.save(&next).expect("thread save");
  });

  join.join().expect("thread join");

  let loaded = store.load();

  assert_eq!(Ok(BootstrapState::Error { reason: String::from("worker failed") }), loaded);
}
