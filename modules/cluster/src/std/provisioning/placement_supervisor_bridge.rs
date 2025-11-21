//! Bridge for PlacementSupervisor integration with sequence gating.

extern crate alloc;
extern crate std;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::core::provisioning::descriptor::ProviderId;
use crate::core::provisioning::snapshot::ProviderSnapshot;

/// PlacementSupervisor consumer that actually applies ownership recalculation.
pub trait PlacementSupervisorPort: Send + Sync {
  /// Apply a new snapshot to PlacementSupervisor.
  fn apply_snapshot(&self, snapshot: &ProviderSnapshot);
  /// Notify provider change for failover sequencing.
  fn provider_changed(&self, from: ProviderId, to: ProviderId);
}

/// Bridge that enforces monotonic `seq_no` and rejects updates during shutdown.
pub struct PlacementSupervisorBridge {
  port:          Arc<dyn PlacementSupervisorPort>,
  last_seq:      Mutex<u64>,
  shutting_down: AtomicBool,
}

/// Bridge-side error.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum PlacementBridgeError {
  /// Provided sequence was not newer than the last accepted one.
  #[error("sequence {seq_no} is not newer than last {last_seq}")]
  OutOfOrder {
    /// Sequence number supplied by the caller.
    seq_no:   u64,
    /// Most recent sequence number accepted by the bridge.
    last_seq: u64,
  },
  /// Bridge is draining; new updates are rejected.
  #[error("shutdown in progress")]
  ShuttingDown,
}

impl PlacementSupervisorBridge {
  /// Create a bridge with the given consumer.
  pub fn new(port: Arc<dyn PlacementSupervisorPort>) -> Self {
    Self { port, last_seq: Mutex::new(0), shutting_down: AtomicBool::new(false) }
  }

  /// Apply a snapshot if `seq_no` is strictly increasing.
  pub fn apply_snapshot(
    &self,
    seq_no:    u64,
    snapshot:  &ProviderSnapshot,
  ) -> Result<(), PlacementBridgeError> {
    self.guard(seq_no)?;
    self.port.apply_snapshot(snapshot);
    Ok(())
  }

  /// Notify provider change with ordering guarantees.
  pub fn provider_changed(
    &self,
    seq_no: u64,
    from:   ProviderId,
    to:     ProviderId,
  ) -> Result<(), PlacementBridgeError> {
    self.guard(seq_no)?;
    self.port.provider_changed(from, to);
    Ok(())
  }

  /// Start graceful shutdown; subsequent updates are rejected.
  pub fn begin_shutdown(&self) {
    self.shutting_down.store(true, Ordering::SeqCst);
  }

  fn guard(&self, seq_no: u64) -> Result<(), PlacementBridgeError> {
    if self.shutting_down.load(Ordering::SeqCst) {
      return Err(PlacementBridgeError::ShuttingDown);
    }
    let mut last = self.last_seq.lock().expect("lock poisoned");
    if seq_no <= *last {
      return Err(PlacementBridgeError::OutOfOrder { seq_no, last_seq: *last });
    }
    *last = seq_no;
    Ok(())
  }
}

#[cfg(test)]
mod tests;
