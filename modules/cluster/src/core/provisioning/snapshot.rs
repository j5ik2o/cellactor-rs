//! Provider snapshots and health.

use alloc::vec::Vec;

use crate::core::identity::{ClusterNode, NodeId};

/// Provider health status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderHealth {
  /// Healthy provider delivering snapshots.
  Healthy,
  /// Degraded but still delivering snapshots.
  Degraded,
  /// Unreachable provider.
  Unreachable,
}

/// Immutable snapshot of provider membership state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderSnapshot {
  /// 現在のメンバー一覧。
  pub members:       Vec<ClusterNode>,
  /// スナップショットのハッシュ。
  pub hash:          u64,
  /// Block/Unblock 済みノード。
  pub blocked_nodes: Vec<NodeId>,
  /// プロバイダの健全性。
  pub health:        ProviderHealth,
}
