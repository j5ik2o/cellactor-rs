//! Provider events consumed by WatchHub.

extern crate alloc;

use alloc::string::String;
use crate::core::identity::NodeId;
use crate::core::provisioning::snapshot::ProviderSnapshot;

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

/// Event delivered to Remoting about remote topology changes.
#[derive(Debug, Clone, PartialEq)]
pub enum RemoteTopologyEvent {
  /// ノード参加。
  Join(NodeId),
  /// ノード離脱。
  Leave(NodeId),
  /// ノード隔離。
  Blocked(NodeId),
  /// 隔離解除。
  Unblocked(NodeId),
}
