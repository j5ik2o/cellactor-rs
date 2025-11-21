//! Block/Unblock イベントをスナップショットへ反映する補助関数。

extern crate alloc;
extern crate std;

use alloc::{format, string::String};

use crate::{
  core::provisioning::snapshot::ProviderSnapshot,
  std::provisioning::provider_event::{RemoteTopologyEvent, RemoteTopologyKind},
};

/// Block/Unblock を snapshot.blocked_nodes に反映し、Degraded 時は警告を通知する。
pub fn apply_block_event(snapshot: &mut ProviderSnapshot, event: &RemoteTopologyEvent, warn: impl FnOnce(String)) {
  match event.kind {
    | RemoteTopologyKind::Blocked => {
      if !snapshot.blocked_nodes.iter().any(|n| n == &event.node_id) {
        snapshot.blocked_nodes.push(event.node_id.clone());
      }
      if snapshot.health.is_degraded() {
        warn(format!("node {} blocked while provider degraded", event.node_id.as_str()));
      }
    },
    | RemoteTopologyKind::Unblocked => {
      snapshot.blocked_nodes.retain(|n| n != &event.node_id);
    },
    | _ => {},
  }
}

trait HealthExt {
  fn is_degraded(&self) -> bool;
}

impl HealthExt for crate::core::provisioning::snapshot::ProviderHealth {
  fn is_degraded(&self) -> bool {
    matches!(self, crate::core::provisioning::snapshot::ProviderHealth::Degraded)
  }
}

#[cfg(test)]
mod tests;
