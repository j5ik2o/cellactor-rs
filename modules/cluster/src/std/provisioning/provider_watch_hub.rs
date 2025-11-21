//! Aggregates provider streams and delivers snapshots to consumers.

extern crate alloc;
extern crate std;

use std::sync::RwLock;

use crate::core::provisioning::snapshot::ProviderSnapshot;
use crate::std::provisioning::provider_event::{ProviderEvent, ProviderTermination};

/// In-memory hub that keeps the latest snapshot and termination info.
pub struct ProviderWatchHub {
  latest: RwLock<Option<ProviderSnapshot>>,
  terminated: RwLock<Option<ProviderTermination>>, 
}

impl ProviderWatchHub {
  /// Creates a new hub with empty snapshot.
  pub fn new() -> Self {
    Self { latest: RwLock::new(None), terminated: RwLock::new(None) }
  }

  /// Applies an incoming provider event.
  pub fn apply_event(&self, event: ProviderEvent) {
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
  }

  /// Returns the latest snapshot if any.
  pub fn latest_snapshot(&self) -> Option<ProviderSnapshot> {
    self.latest.read().expect("poison").clone()
  }

  /// Returns termination info if set.
  pub fn termination(&self) -> Option<ProviderTermination> {
    self.terminated.read().expect("poison").clone()
  }
}

#[cfg(test)]
mod tests;
