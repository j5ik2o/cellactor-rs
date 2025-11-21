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

/// 必須機能の存在を検査する。
pub trait CapabilityChecker: Send + Sync {
  /// 例: watch をサポートしているか。問題があれば理由文字列を Err で返す。
  fn check(&self, descriptor: &ProviderDescriptor) -> Result<(), String>;
}

/// 検証結果。Disabled の場合は理由を保持する。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationResult {
  pub descriptor:      ProviderDescriptor,
  pub disabled_reason: Option<String>,
}

/// 設定検証を行う。
pub struct ProviderValidator<C: ConnectivityChecker, K: CapabilityChecker> {
  connectivity: C,
  capability:   K,
}

impl<C: ConnectivityChecker, K: CapabilityChecker> ProviderValidator<C, K> {
  /// 新しいバリデータを作成する。
  pub fn new(connectivity: C, capability: K) -> Self {
    Self { connectivity, capability }
  }

  /// ディスクリプタを検証する。能力不足は Disabled として許容し、理由を保持する。
  pub fn validate(&self, descriptor: &ProviderDescriptor) -> Result<ValidationResult, ProvisioningError> {
    // endpoint 必須 (Consul/K8s/custom で想定)。InMemory は不要。
    if matches!(descriptor.kind(), ProviderKind::Consul | ProviderKind::Kubernetes | ProviderKind::Custom(_)) {
      if descriptor.endpoint().is_none() || descriptor.endpoint().map(|e| e.is_empty()).unwrap_or(true) {
        return Err(ProvisioningError::new(ProvisioningErrorCode::Validation, "missing endpoint"));
      }
    }

    self
      .connectivity
      .check(descriptor)
      .map_err(|e| ProvisioningError::new(ProvisioningErrorCode::Connectivity, e))?;

    let disabled_reason = self.capability.check(descriptor).err();

    Ok(ValidationResult { descriptor: descriptor.clone(), disabled_reason })
  }
}

#[cfg(test)]
mod tests;
