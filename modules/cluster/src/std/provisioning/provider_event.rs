//! Provider events consumed by WatchHub and emitted to Remoting.

extern crate alloc;

use alloc::string::{String, ToString};

use crate::core::{
  identity::NodeId,
  provisioning::{descriptor::ProviderId, snapshot::ProviderSnapshot},
};

/// Topology or control events emitted by providers.
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderEvent {
  /// New or updated snapshot.
  Snapshot(ProviderSnapshot),
  /// Provider stream terminated.
  Terminated {
    /// 終了理由。
    reason: ProviderTermination,
  },
}

/// Reasons for provider termination.
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderTermination {
  /// 正常終了。
  Ended,
  /// エラー終了。
  Errored {
    /// 人間可読な理由。
    reason: String,
  },
}

/// Remote topology change propagated to Remoting side.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteTopologyEvent {
  /// イベントシーケンス。冪等キーに利用する。
  pub seq_no:        u64,
  /// 発生元プロバイダ。
  pub provider_id:   ProviderId,
  /// 対象スナップショットのハッシュ。
  pub snapshot_hash: u64,
  /// 影響を受けるノード。
  pub node_id:       NodeId,
  /// イベントの種類。
  pub kind:          RemoteTopologyKind,
}

impl RemoteTopologyEvent {
  /// 冪等性を担保するキー (provider, seq_no)。
  pub fn idempotency_key(&self) -> (String, u64) {
    (self.provider_id.as_str().to_string(), self.seq_no)
  }
}

/// 遠隔トポロジイベントの種類。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteTopologyKind {
  /// ノード参加。
  Join,
  /// ノード離脱。
  Leave,
  /// ノード隔離。
  Blocked,
  /// 隔離解除。
  Unblocked,
}
