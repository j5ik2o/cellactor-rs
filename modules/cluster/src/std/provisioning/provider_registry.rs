//! ProviderRegistry: 登録と永続化を扱う。

extern crate alloc;
extern crate std;

use alloc::{string::String, string::ToString, vec::Vec};
use std::collections::HashMap;
use std::sync::RwLock;

use crate::core::provisioning::descriptor::ProviderDescriptor;
use crate::std::provisioning::provider_store::ProviderStore;

/// Registry 操作時のエラー。
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ProviderRegistryError {
  /// 重複 id を検出。
  #[error("duplicate provider id: {0}")]
  Duplicate(String),
  /// 永続化ストアでエラー。
  #[error("store error: {0}")]
  Store(String),
}

type StoreResult<T> = Result<T, ProviderRegistryError>;

/// Provider の登録と永続化を担う。
pub struct ProviderRegistry<S: ProviderStore> {
  store:  S,
  map:    RwLock<HashMap<String, ProviderDescriptor>>,
}

impl<S: ProviderStore> ProviderRegistry<S> {
  /// ストアからロードして新規作成。
  pub fn new(store: S) -> Result<Self, ProviderRegistryError> {
    let descriptors = store
      .load_descriptors()
      .map_err(|e: crate::std::provisioning::provider_store::ProviderStoreError| {
        ProviderRegistryError::Store(e.to_string())
      })?;
    let mut map = HashMap::new();
    for desc in descriptors {
      map.insert(desc.id().as_str().to_string(), desc);
    }
    Ok(Self { store, map: RwLock::new(map) })
  }

  /// 新規登録。重複 id は拒否。
  pub fn register(&self, descriptor: ProviderDescriptor) -> StoreResult<()> {
    let mut guard = self.map.write().expect("poison");
    if guard.contains_key(descriptor.id().as_str()) {
      return Err(ProviderRegistryError::Duplicate(descriptor.id().as_str().to_string()));
    }
    guard.insert(descriptor.id().as_str().to_string(), descriptor);
    self.persist_locked(&guard)
  }

  /// 永続化のみ実行。
  pub fn save(&self) -> StoreResult<()> {
    let guard = self.map.read().expect("poison");
    self.persist_locked(&guard)
  }

  /// 登録済み一覧。
  pub fn list(&self) -> Vec<ProviderDescriptor> {
    let guard = self.map.read().expect("poison");
    guard.values().cloned().collect()
  }

  fn persist_locked(&self, map: &HashMap<String, ProviderDescriptor>) -> StoreResult<()> {
    let mut list: Vec<ProviderDescriptor> = map.values().cloned().collect();
    list.sort_by_key(|d: &ProviderDescriptor| d.id().as_str().to_string());
    self
      .store
      .save_descriptors(&list)
      .map_err(|e: crate::std::provisioning::provider_store::ProviderStoreError| {
        ProviderRegistryError::Store(e.to_string())
      })
  }
}

#[cfg(test)]
mod tests;
