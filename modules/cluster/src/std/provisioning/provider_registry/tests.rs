use tempfile::tempdir;

use crate::core::provisioning::descriptor::{ProviderDescriptor, ProviderId, ProviderKind};
use crate::std::provisioning::provider_registry::{ProviderRegistry, ProviderRegistryError};
use crate::std::provisioning::provider_store::FileProviderStore;

fn registry() -> (ProviderRegistry<FileProviderStore>, std::path::PathBuf, tempfile::TempDir) {
  let dir = tempdir().unwrap();
  let path = dir.path().join("providers.jsonl");
  let reg = ProviderRegistry::new(FileProviderStore::new(path.clone())).unwrap();
  (reg, path, dir)
}

fn desc(id: &str, prio: u8) -> ProviderDescriptor {
  ProviderDescriptor::new(ProviderId::new(id), ProviderKind::InMemory, prio)
}

#[test]
fn registers_and_persists() {
  let (reg, path, _dir) = registry();
  reg.register(desc("a", 1)).unwrap();
  reg.register(desc("b", 2)).unwrap();

  let list = reg.list();
  assert_eq!(2, list.len());

  reg.save().unwrap();

  // reload from disk
  let reg2 = ProviderRegistry::new(FileProviderStore::new(path)).unwrap();
  let list2 = reg2.list();
  assert_eq!(2, list2.len());
}

#[test]
fn rejects_duplicates() {
  let (reg, _path, _dir) = registry();
  reg.register(desc("dup", 1)).unwrap();
  let err = reg.register(desc("dup", 2)).unwrap_err();
  assert!(matches!(err, ProviderRegistryError::Duplicate(id) if id == "dup"));
}
