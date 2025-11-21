//! Remoting 連携用ブリッジ。冪等キーで重複を抑止する。

extern crate alloc;
extern crate std;

use alloc::string::String;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::std::provisioning::{
  provider_event::RemoteTopologyEvent,
  remoting_health::RemotingHealthMetrics,
  remoting_port::RemotingPort,
};

/// Remoting への送信をラップし冪等性を保証する。
pub struct RemotingBridge {
  port: Arc<dyn RemotingPort>,
  seen: Mutex<HashSet<(String, u64)>>,
  health: Option<Arc<RemotingHealthMetrics>>,
}

/// RemotingBridge 操作エラー。
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum RemotingBridgeError {
  /// 同一冪等キーの重複イベント。
  #[error("duplicate remote topology event: {provider}/{seq_no}")]
  Duplicate {
    /// プロバイダ ID。
    provider: String,
    /// シーケンス番号。
    seq_no:   u64,
  },
}

impl RemotingBridge {
  /// 新しいブリッジを作成。
  pub fn new(port: Arc<dyn RemotingPort>) -> Self {
    Self { port, seen: Mutex::new(HashSet::new()), health: None }
  }

  /// 健全性メトリクス付きで作成。
  pub fn with_health(port: Arc<dyn RemotingPort>, health: Arc<RemotingHealthMetrics>) -> Self {
    Self { port, seen: Mutex::new(HashSet::new()), health: Some(health) }
  }

  /// 冪等キーに基づき配信し、重複は拒否する。
  pub fn publish(&self, event: &RemoteTopologyEvent) -> Result<(), RemotingBridgeError> {
    let key = event.idempotency_key();
    {
      let mut seen = self.seen.lock().expect("poisoned");
      if !seen.insert(key.clone()) {
        return Err(RemotingBridgeError::Duplicate { provider: key.0, seq_no: key.1 });
      }
    }
    self.port.publish_remote_topology(event);
    if let Some(health) = &self.health {
      health.record_event(event);
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests;
