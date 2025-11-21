//! Aggregates provider streams and delivers snapshots to consumers.

extern crate alloc;
extern crate std;

use std::sync::RwLock;

use crate::core::provisioning::snapshot::ProviderSnapshot;
use crate::std::provisioning::provider_event::{ProviderEvent, ProviderTermination};

/// In-memory hub that keeps the latest snapshot and termination info.
pub struct ProviderWatchHub {
  latest:        RwLock<Option<ProviderSnapshot>>,
  terminated:    RwLock<Option<ProviderTermination>>,
  last_hash:     RwLock<Option<u64>>,
  last_invalid:  RwLock<bool>,
  shutting_down: RwLock<bool>,
}

impl ProviderWatchHub {
  /// Creates a new hub with empty snapshot.
  pub fn new() -> Self {
    Self {
      latest:        RwLock::new(None),
      terminated:    RwLock::new(None),
      last_hash:     RwLock::new(None),
      last_invalid:  RwLock::new(false),
      shutting_down: RwLock::new(false),
    }
  }

  /// Applies an incoming provider event.
  pub fn apply_event(&self, event: ProviderEvent) -> Result<(), WatchError> {
    if *self.shutting_down.read().expect("poison") {
      return Err(WatchError::ShuttingDown);
    }
    match event {
      ProviderEvent::Snapshot(s) => {
        let mut latest = self.latest.write().expect("poison");
        let mut last_hash = self.last_hash.write().expect("poison");
        let mut last_invalid = self.last_invalid.write().expect("poison");

        let invalidated = last_hash.map_or(false, |prev| prev != s.hash);
        *last_invalid = invalidated;
        *last_hash = Some(s.hash);
        *latest = Some(s);
      },
      ProviderEvent::Terminated { reason } => {
        let mut term = self.terminated.write().expect("poison");
        *term = Some(reason);
      },
    }
    Ok(())
  }

  /// Returns the latest snapshot if any.
  pub fn latest_snapshot(&self) -> Option<ProviderSnapshot> {
    self.latest.read().expect("poison").clone()
  }

  /// Returns latest snapshotとハッシュ変化有無をまとめて返す。
  pub fn latest_snapshot_with_invalidation(&self) -> Option<(ProviderSnapshot, bool)> {
    let snap = self.latest.read().expect("poison").clone()?;
    let invalid = *self.last_invalid.read().expect("poison");
    Some((snap, invalid))
  }

  /// Returns termination info if set.
  pub fn termination(&self) -> Option<ProviderTermination> {
    self.terminated.read().expect("poison").clone()
  }

  /// Initiates graceful shutdown: 以後のスナップショットを拒否し、最後のスナップショットを保持。
  pub fn begin_shutdown(&self) {
    let mut flag = self.shutting_down.write().expect("poison");
    *flag = true;
  }
}

/// WatchHub 操作エラー。
#[derive(Debug, PartialEq, Eq)]
pub enum WatchError {
  /// Shutown 中のため拒否。
  ShuttingDown,
}

#[cfg(test)]
mod tests;
