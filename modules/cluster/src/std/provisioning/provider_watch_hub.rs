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
  shutting_down: RwLock<bool>,
}

impl ProviderWatchHub {
  /// Creates a new hub with empty snapshot.
  pub fn new() -> Self {
    Self { latest: RwLock::new(None), terminated: RwLock::new(None), shutting_down: RwLock::new(false) }
  }

  /// Applies an incoming provider event.
  pub fn apply_event(&self, event: ProviderEvent) -> Result<(), WatchError> {
    if *self.shutting_down.read().expect("poison") {
      return Err(WatchError::ShuttingDown);
    }
    match event {
      ProviderEvent::Snapshot(s) => {
        let mut latest = self.latest.write().expect("poison");
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
