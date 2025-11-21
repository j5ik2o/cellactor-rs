//! Immutable snapshot of provider membership state.

use alloc::vec::Vec;

use super::ProviderHealth;
use crate::core::identity::{ClusterNode, NodeId};

/// Immutable snapshot of provider membership state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderSnapshot {
  /// Current member list.
  pub members:       Vec<ClusterNode>,
  /// Snapshot hash value.
  pub hash:          u64,
  /// Blocked/Unblocked nodes.
  pub blocked_nodes: Vec<NodeId>,
  /// Provider health status.
  pub health:        ProviderHealth,
}
