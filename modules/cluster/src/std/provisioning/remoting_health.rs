//! Remoting 健全性メトリクス。up/down/degraded と最終更新時刻を追跡する。

extern crate alloc;
extern crate std;

use alloc::collections::BTreeMap;
use std::time::{Duration, Instant};

use crate::std::provisioning::provider_event::{RemoteTopologyEvent, RemoteTopologyKind};

/// ノードの健全性。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RemotingNodeStatus {
  Up,
  Down,
  Degraded,
}

/// ノード健全性のスナップショット。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemotingHealthEntry {
  pub node_id:      String,
  pub status:       RemotingNodeStatus,
  pub last_updated: Instant,
}

/// 健全性を蓄積するメトリクス。
pub struct RemotingHealthMetrics {
  map: BTreeMap<String, RemotingHealthEntry>,
}

impl RemotingHealthMetrics {
  /// 作成。
  pub fn new() -> Self {
    Self { map: BTreeMap::new() }
  }

  /// イベントを反映する。
  pub fn record_event(&self, event: &RemoteTopologyEvent) {
    let now = Instant::now();
    let status = match event.kind {
      | RemoteTopologyKind::Join | RemoteTopologyKind::Unblocked => RemotingNodeStatus::Up,
      | RemoteTopologyKind::Leave => RemotingNodeStatus::Down,
      | RemoteTopologyKind::Blocked => RemotingNodeStatus::Degraded,
    };
    // BTreeMap は interior mutability ではないので再構成
    // （頻度は低い前提のため clone+insert を許容）
    let mut new_map = self.map.clone();
    new_map.insert(event.node_id.as_str().to_string(), RemotingHealthEntry {
      node_id: event.node_id.as_str().to_string(),
      status,
      last_updated: now,
    });
    // replace
    // Safety: レースは無い前提（ブリッジ単一スレッド利用）
    unsafe {
      let ptr = &self.map as *const _ as *mut BTreeMap<String, RemotingHealthEntry>;
      *ptr = new_map;
    }
  }

  /// スナップショットを取得。
  pub fn snapshot(&self) -> Vec<RemotingHealthEntry> {
    self.map.values().cloned().collect()
  }

  /// 指定ノードの状態を取得。
  pub fn status_of(&self, node: &str) -> Option<RemotingHealthEntry> {
    self.map.get(node).cloned()
  }
}

impl Default for RemotingHealthMetrics {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests;
