//! ProviderValidator: 設定検証と接続チェックの枠組み。

extern crate alloc;

use alloc::string::String;

use crate::core::provisioning::descriptor::{ProviderDescriptor, ProviderKind};
use crate::std::provisioning::provisioning_error::{ProvisioningError, ProvisioningErrorCode};

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
  pub fn validate(&self, descriptor: &ProviderDescriptor) -> Result<(), ProvisioningError> {
    // endpoint 必須 (Consul/K8s/custom で想定)。InMemory は不要。
    if matches!(descriptor.kind(), ProviderKind::Consul | ProviderKind::Kubernetes | ProviderKind::Custom(_)) {
      if descriptor.endpoint().is_none() || descriptor.endpoint().map(|e| e.is_empty()).unwrap_or(true) {
        return Err(ProvisioningError::new(ProvisioningErrorCode::Validation, "missing endpoint"));
      }
    }

    self
      .checker
      .check(descriptor)
      .map_err(|e| ProvisioningError::new(ProvisioningErrorCode::Connectivity, e))?;

    Ok(())
  }
}

#[cfg(test)]
mod tests;
