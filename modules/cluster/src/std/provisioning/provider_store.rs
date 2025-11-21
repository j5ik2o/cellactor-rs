//! 永続化抽象とファイル実装。

extern crate alloc;
extern crate std;

use alloc::{
  string::{String, ToString},
  vec::Vec,
};
use std::{
  fs::{File, OpenOptions},
  io::{BufReader, Write},
  path::PathBuf,
};

use serde_json::Deserializer;

use crate::core::provisioning::descriptor::ProviderDescriptor;

/// 永続化エラー。
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ProviderStoreError {
  /// IO エラー。
  #[error("io error: {0}")]
  Io(String),
  /// ファイル破損検知。
  #[error("corrupted provider file")]
  Corrupted,
}

/// ProviderDescriptor を保存/読込する永続化抽象。
pub trait ProviderStore: Send + Sync {
  /// ディスクリプタを読込む。
  fn load_descriptors(&self) -> Result<Vec<ProviderDescriptor>, ProviderStoreError>;
  /// ディスクリプタを保存する。
  fn save_descriptors(&self, descriptors: &[ProviderDescriptor]) -> Result<(), ProviderStoreError>;
}

/// JSON Lines を tempfile+rename で原子保存する実装。
pub struct FileProviderStore {
  path: PathBuf,
}

impl FileProviderStore {
  /// 作成する。存在しない場合は後で保存時に作られる。
  pub fn new(path: PathBuf) -> Self {
    Self { path }
  }

  fn to_jsonl(desc: &ProviderDescriptor) -> String {
    serde_json::to_string(desc).expect("serialize descriptor")
  }
}

impl ProviderStore for FileProviderStore {
  fn load_descriptors(&self) -> Result<Vec<ProviderDescriptor>, ProviderStoreError> {
    if !self.path.exists() {
      return Ok(Vec::new());
    }
    let file = File::open(&self.path).map_err(|e: std::io::Error| ProviderStoreError::Io(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    let stream = Deserializer::from_reader(reader).into_iter::<ProviderDescriptor>();
    for item in stream {
      let desc: ProviderDescriptor = item.map_err(|_| ProviderStoreError::Corrupted)?;
      out.push(desc);
    }
    Ok(out)
  }

  fn save_descriptors(&self, descriptors: &[ProviderDescriptor]) -> Result<(), ProviderStoreError> {
    let tmp_path = self.path.with_extension("tmp");
    let mut file = OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .open(&tmp_path)
      .map_err(|e: std::io::Error| ProviderStoreError::Io(e.to_string()))?;

    for (idx, desc) in descriptors.iter().enumerate() {
      if idx > 0 {
        writeln!(file).map_err(|e: std::io::Error| ProviderStoreError::Io(e.to_string()))?;
      }
      write!(file, "{}", Self::to_jsonl(desc)).map_err(|e: std::io::Error| ProviderStoreError::Io(e.to_string()))?;
    }
    file.sync_all().map_err(|e: std::io::Error| ProviderStoreError::Io(e.to_string()))?;
    std::fs::rename(&tmp_path, &self.path).map_err(|e: std::io::Error| ProviderStoreError::Io(e.to_string()))?;
    Ok(())
  }
}

impl From<std::io::Error> for ProviderStoreError {
  fn from(value: std::io::Error) -> Self {
    ProviderStoreError::Io(value.to_string())
  }
}

#[cfg(test)]
mod tests;
