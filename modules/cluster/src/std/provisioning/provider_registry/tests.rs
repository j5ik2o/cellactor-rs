use tempfile::tempdir;

use crate::{
  core::provisioning::descriptor::{ProviderDescriptor, ProviderId, ProviderKind},
  std::provisioning::{
    provider_registry::{ProviderRegistry, ProviderRegistryError},
    provider_store::FileProviderStore,
    provider_validator::{CapabilityChecker, ConnectivityChecker, ProviderValidator, ValidationResult},
  },
};

fn registry() -> (ProviderRegistry<FileProviderStore>, std::path::PathBuf, tempfile::TempDir) {
  let dir = tempdir().unwrap();
  let path = dir.path().join("providers.jsonl");
  let reg = ProviderRegistry::new(FileProviderStore::new(path.clone())).unwrap();
  (reg, path, dir)
}

fn desc(id: &str, prio: u8) -> ProviderDescriptor {
  ProviderDescriptor::new(ProviderId::new(id), ProviderKind::InMemory, prio)
}

#[derive(Clone, Copy)]
struct OkConn;
impl ConnectivityChecker for OkConn {
  fn check(&self, _descriptor: &ProviderDescriptor) -> Result<(), String> {
    Ok(())
  }
}

#[derive(Clone, Copy)]
struct HasWatch;
impl CapabilityChecker for HasWatch {
  fn check(&self, _descriptor: &ProviderDescriptor) -> Result<(), String> {
    Ok(())
  }
}

#[test]
fn registers_and_persists() {
  let (reg, path, _dir) = registry();
  let validator = ProviderValidator::new(OkConn, HasWatch);
  reg.register(&validator, desc("a", 1)).unwrap();
  reg.register(&validator, desc("b", 2)).unwrap();

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
  let validator = ProviderValidator::new(OkConn, HasWatch);
  reg.register(&validator, desc("dup", 1)).unwrap();
  let err = reg.register(&validator, desc("dup", 2)).unwrap_err();
  assert!(matches!(err, ProviderRegistryError::Duplicate(id) if id == "dup"));
}

#[test]
fn exposes_disabled_reason_from_validator() {
  struct NoWatch;
  impl CapabilityChecker for NoWatch {
    fn check(&self, _descriptor: &ProviderDescriptor) -> Result<(), String> {
      Err("no watch".to_string())
    }
  }
  let (reg, _path, _dir) = registry();
  let validator = ProviderValidator::new(OkConn, NoWatch);

  reg.register(&validator, desc("p1", 1)).unwrap();

  let statuses: Vec<ValidationResult> = reg.list_with_status();
  assert_eq!(1, statuses.len());
  assert_eq!(Some("no watch".to_string()), statuses[0].disabled_reason);
}
