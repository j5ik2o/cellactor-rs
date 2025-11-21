//! フェイルオーバ判定とバックオフ制御。

extern crate alloc;
extern crate std;

use alloc::vec::Vec;
use std::time::{Duration, Instant};

use crate::core::provisioning::descriptor::ProviderDescriptor;
use crate::core::provisioning::snapshot::ProviderHealth;

/// フェイルオーバ設定。
#[derive(Clone, Debug)]
pub struct FailoverConfig {
  /// 無応答とみなすタイムアウト。
  pub timeout:      Duration,
  /// 許容連続エラー回数。
  pub max_errors:   u32,
  /// バックオフ初期値。
  pub backoff_init: Duration,
  /// バックオフ上限。
  pub backoff_max:  Duration,
  /// フェイルオーバ後のクールダウン。
  pub cooldown:     Duration,
}

impl Default for FailoverConfig {
  fn default() -> Self {
    Self {
      timeout:      Duration::from_secs(15),
      max_errors:   3,
      backoff_init: Duration::from_secs(2),
      backoff_max:  Duration::from_secs(30),
      cooldown:     Duration::from_secs(10),
    }
  }
}

#[derive(Clone, Debug)]
struct ProviderState {
  descriptor:   ProviderDescriptor,
  errors:       u32,
  backoff:      Duration,
  last_failure: Option<Instant>,
  health:       ProviderHealth,
}

impl ProviderState {
  fn new(descriptor: ProviderDescriptor, cfg: &FailoverConfig) -> Self {
    Self {
      descriptor,
      errors:       0,
      backoff:      cfg.backoff_init,
      last_failure: None,
      health:       ProviderHealth::Healthy,
    }
  }
}

/// 優先度付きフェイルオーバ制御。
pub struct FailoverController {
  cfg:     FailoverConfig,
  states:  Vec<ProviderState>,
}

impl FailoverController {
  /// プロバイダ一覧から生成する（priority 高い順に並べ替え）。
  pub fn new(mut providers: Vec<ProviderDescriptor>, cfg: FailoverConfig) -> Self {
    providers.sort_by(|a, b| b.priority().cmp(&a.priority()));
    let states = providers.into_iter().map(|d| ProviderState::new(d, &cfg)).collect();
    Self { cfg, states }
  }

  /// ヘルス判定を更新し、利用可能な最優先プロバイダを返す。
  pub fn select_active(&mut self) -> Option<ProviderDescriptor> {
    let now = Instant::now();
    for state in &mut self.states {
      if let Some(last) = state.last_failure {
        if now.duration_since(last) >= self.cfg.cooldown {
          state.errors = 0;
          state.backoff = self.cfg.backoff_init;
          state.health = ProviderHealth::Healthy;
          state.last_failure = None;
        }
      }
      // timeout-based degradation
      if let Some(last) = state.last_failure {
        if now.duration_since(last) >= self.cfg.timeout {
          state.health = ProviderHealth::Unreachable;
        }
      }
    }

    self.states.iter().find(|s| s.health != ProviderHealth::Unreachable).map(|s| s.descriptor.clone())
  }

  /// 失敗を記録し、必要ならフェイルオーバ対象にする。
  pub fn record_failure(&mut self, provider_id: &str, _reason: &str) {
    let now = Instant::now();
    if let Some(state) = self.states.iter_mut().find(|s| s.descriptor.id().as_str() == provider_id) {
      state.errors = state.errors.saturating_add(1);
      state.last_failure = Some(now);
      if state.errors >= self.cfg.max_errors {
        state.health = ProviderHealth::Unreachable;
        // backoff update
        state.backoff = (state.backoff * 2).min(self.cfg.backoff_max);
      } else {
        state.health = ProviderHealth::Degraded;
      }
    }
  }

  /// 成功を記録しヘルスを回復。
  pub fn record_success(&mut self, provider_id: &str) {
    if let Some(state) = self.states.iter_mut().find(|s| s.descriptor.id().as_str() == provider_id) {
      state.errors = 0;
      state.health = ProviderHealth::Healthy;
      state.backoff = self.cfg.backoff_init;
      state.last_failure = None;
    }
  }
}

#[cfg(test)]
mod tests;
