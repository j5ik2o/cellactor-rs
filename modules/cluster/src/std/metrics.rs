extern crate alloc;
extern crate std;

use std::{sync::Mutex, time::Duration};

use crate::core::{
  config::ClusterMetricsConfig,
  identity::{ClusterIdentity, NodeId},
  metrics::ClusterMetrics,
};

/// Host環境向けのメトリクス実装。OpenTelemetry 等の実計測が未導入でも
/// 計数・ゲージ・最終値を保持し、テストや診断に利用できる。
pub struct StdClusterMetrics {
  enabled:               bool,
  resolves:              Mutex<u64>,
  requests:              Mutex<u64>,
  retries:               Mutex<u64>,
  timeouts:              Mutex<u64>,
  block_lists:           Mutex<u64>,
  gauge:                 Mutex<usize>,
  last_resolve_duration: Mutex<Option<Duration>>,
  last_request_duration: Mutex<Option<Duration>>,
}

impl StdClusterMetrics {
  /// 新しいメトリクスシンクを構築する。
  #[must_use]
  pub fn new(config: &ClusterMetricsConfig) -> Self {
    Self {
      enabled:               config.enabled(),
      resolves:              Mutex::new(0),
      requests:              Mutex::new(0),
      retries:               Mutex::new(0),
      timeouts:              Mutex::new(0),
      block_lists:           Mutex::new(0),
      gauge:                 Mutex::new(0),
      last_resolve_duration: Mutex::new(None),
      last_request_duration: Mutex::new(None),
    }
  }

  /// 記録済みの解決数を取得（テスト用）。
  #[must_use]
  pub fn resolve_count(&self) -> u64 {
    *self.resolves.lock().expect("mutex poisoned")
  }

  /// 記録済みのリトライ数を取得（テスト用）。
  #[must_use]
  pub fn retry_count(&self) -> u64 {
    *self.retries.lock().expect("mutex poisoned")
  }

  /// 記録済みのタイムアウト数を取得（テスト用）。
  #[must_use]
  pub fn timeout_count(&self) -> u64 {
    *self.timeouts.lock().expect("mutex poisoned")
  }

  /// 記録済みのリクエスト数を取得（テスト用）。
  #[must_use]
  pub fn request_count(&self) -> u64 {
    *self.requests.lock().expect("mutex poisoned")
  }

  /// 記録済みの BlockList 件数を取得（テスト用）。
  #[must_use]
  pub fn block_list_count(&self) -> u64 {
    *self.block_lists.lock().expect("mutex poisoned")
  }

  /// 現在の VA ゲージを取得（テスト用）。
  #[must_use]
  pub fn virtual_actor_gauge(&self) -> usize {
    *self.gauge.lock().expect("mutex poisoned")
  }

  /// 最新の解決所要時間を取得（テスト用）。
  #[must_use]
  pub fn last_resolve_duration(&self) -> Option<Duration> {
    *self.last_resolve_duration.lock().expect("mutex poisoned")
  }

  /// 最新のリクエスト所要時間を取得（テスト用）。
  #[must_use]
  pub fn last_request_duration(&self) -> Option<Duration> {
    *self.last_request_duration.lock().expect("mutex poisoned")
  }
}

impl ClusterMetrics for StdClusterMetrics {
  fn as_any(&self) -> &dyn core::any::Any {
    self
  }

  fn record_resolve_duration(&self, _identity: &ClusterIdentity, duration: Duration) {
    if !self.enabled {
      return;
    }
    *self.resolves.lock().expect("mutex poisoned") += 1;
    *self.last_resolve_duration.lock().expect("mutex poisoned") = Some(duration);
  }

  fn record_request_duration(&self, _identity: &ClusterIdentity, duration: Duration) {
    if !self.enabled {
      return;
    }
    *self.requests.lock().expect("mutex poisoned") += 1;
    *self.last_request_duration.lock().expect("mutex poisoned") = Some(duration);
  }

  fn record_retry_attempt(&self, _identity: &ClusterIdentity) {
    if !self.enabled {
      return;
    }
    *self.retries.lock().expect("mutex poisoned") += 1;
  }

  fn record_timeout(&self, _identity: &ClusterIdentity) {
    if !self.enabled {
      return;
    }
    *self.timeouts.lock().expect("mutex poisoned") += 1;
  }

  fn set_virtual_actor_gauge(&self, value: usize) {
    if !self.enabled {
      return;
    }
    *self.gauge.lock().expect("mutex poisoned") = value;
  }

  fn increment_block_list(&self, _node: &NodeId) {
    if !self.enabled {
      return;
    }
    *self.block_lists.lock().expect("mutex poisoned") += 1;
  }
}

#[cfg(test)]
mod tests;
