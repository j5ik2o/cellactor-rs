use std::fs;

use tempfile::tempdir;

use crate::{
  core::provisioning::descriptor::{ProviderDescriptor, ProviderId, ProviderKind},
  std::provisioning::provider_store::{FileProviderStore, ProviderStore, ProviderStoreError},
};

fn sample_desc(id: &str, prio: u8) -> ProviderDescriptor {
  ProviderDescriptor::new(ProviderId::new(id), ProviderKind::InMemory, prio)
}

#[test]
fn save_and_load_roundtrip() {
  let dir = tempdir().unwrap();
  let path = dir.path().join("providers.jsonl");
  let store = FileProviderStore::new(path.clone());
  let descriptors = vec![sample_desc("a", 1), sample_desc("b", 5)];

  store.save_descriptors(&descriptors).unwrap();

  let loaded = store.load_descriptors().unwrap();
  assert_eq!(descriptors, loaded);
}

#[test]
fn load_returns_empty_when_file_absent() {
  let dir = tempdir().unwrap();
  let path = dir.path().join("providers.jsonl");
  let store = FileProviderStore::new(path);

  let loaded = store.load_descriptors().unwrap();
  assert!(loaded.is_empty());
}

#[test]
fn corrupted_file_is_rejected() {
  let dir = tempdir().unwrap();
  let path = dir.path().join("providers.jsonl");
  fs::write(&path, "{not-json}").unwrap();
  let store = FileProviderStore::new(path);

  let err = store.load_descriptors().unwrap_err();
  assert_eq!(ProviderStoreError::Corrupted, err);
}
