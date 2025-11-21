//! ProviderValidator: 設定検証と接続チェックの枠組み。

extern crate alloc;

use alloc::string::String;

use crate::core::provisioning::descriptor::{ProviderDescriptor, ProviderKind};

/// バリデーションエラー。
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ProviderValidationError {
  /// 必須エンドポイント欠落。
  #[error("missing endpoint")]
  MissingEndpoint,
  /// 要求能力が満たされていない。
  #[error("unsupported capability")]
  UnsupportedCapability,
  /// 接続性検証失敗。
  #[error("connectivity check failed: {0}")]
  Connectivity(String),
}

/// 外部接続チェッカー。
pub trait ConnectivityChecker: Send + Sync {
  /// プロバイダの外部接続性を検証する。
  fn check(&self, descriptor: &ProviderDescriptor) -> Result<(), String>;
}

/// 設定検証を行う。
pub struct ProviderValidator<C: ConnectivityChecker> {
  checker: C,
}

impl<C: ConnectivityChecker> ProviderValidator<C> {
  /// 新しいバリデータを作成する。
  pub fn new(checker: C) -> Self {
    Self { checker }
  }

  /// ディスクリプタを検証する。
  pub fn validate(&self, descriptor: &ProviderDescriptor) -> Result<(), ProviderValidationError> {
    // endpoint 必須 (Consul/K8s/custom で想定)。InMemory は不要。
    if matches!(descriptor.kind(), ProviderKind::Consul | ProviderKind::Kubernetes | ProviderKind::Custom(_)) {
      if descriptor.endpoint().is_none() || descriptor.endpoint().map(|e| e.is_empty()).unwrap_or(true) {
        return Err(ProviderValidationError::MissingEndpoint);
      }
    }

    self
      .checker
      .check(descriptor)
      .map_err(|e| ProviderValidationError::Connectivity(e))?;

    Ok(())
  }
}

#[cfg(test)]
mod tests;
