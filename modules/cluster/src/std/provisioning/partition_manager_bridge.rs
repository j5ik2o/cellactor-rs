//! Bridge for PartitionManager with snapshot cache and sequencing.

extern crate alloc;
extern crate std;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use crate::core::provisioning::descriptor::ProviderId;
use crate::core::provisioning::snapshot::ProviderSnapshot;

/// PartitionManager consumer to receive updates.
pub trait PartitionManagerPort: Send + Sync {
  /// Apply the snapshot to partition maps.
  fn apply_snapshot(&self, snapshot: &ProviderSnapshot);
  /// Notify provider change for partition recalculation.
  fn provider_changed(&self, from: ProviderId, to: ProviderId);
}

/// Bridge that keeps the latest snapshot and enforces seq ordering.
pub struct PartitionManagerBridge {
  port:          Arc<dyn PartitionManagerPort>,
  latest:        RwLock<Option<ProviderSnapshot>>,
  last_seq:      Mutex<u64>,
  shutting_down: AtomicBool,
}

/// PartitionManager bridge errors.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum PartitionManagerError {
  /// No snapshot was cached yet.
  #[error("no snapshot available")]
  NoSnapshot,
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

impl PartitionManagerBridge {
  /// Create bridge with empty snapshot cache.
  pub fn new(port: Arc<dyn PartitionManagerPort>) -> Self {
    Self {
      port,
      latest: RwLock::new(None),
      last_seq: Mutex::new(0),
      shutting_down: AtomicBool::new(false),
    }
  }

  /// Apply a snapshot, caching and forwarding if sequence is newer.
  pub fn apply_snapshot(
    &self,
    seq_no:   u64,
    snapshot: ProviderSnapshot,
  ) -> Result<(), PartitionManagerError> {
    self.guard(seq_no)?;
    {
      let mut latest = self.latest.write().expect("poisoned");
      *latest = Some(snapshot.clone());
    }
    self.port.apply_snapshot(&snapshot);
    Ok(())
  }

  /// Return the latest cached snapshot or fail with `NoSnapshot`.
  pub fn latest_snapshot(&self) -> Result<ProviderSnapshot, PartitionManagerError> {
    self
      .latest
      .read()
      .expect("poisoned")
      .clone()
      .ok_or(PartitionManagerError::NoSnapshot)
  }

  /// Notify partition manager of provider change respecting ordering.
  pub fn provider_changed(
    &self,
    seq_no: u64,
    from:   ProviderId,
    to:     ProviderId,
  ) -> Result<(), PartitionManagerError> {
    self.guard(seq_no)?;
    self.port.provider_changed(from, to);
    Ok(())
  }

  /// Begin graceful shutdown: rejects new updates but retains the last snapshot for drain.
  pub fn begin_shutdown(&self) {
    self.shutting_down.store(true, Ordering::SeqCst);
  }

  fn guard(&self, seq_no: u64) -> Result<(), PartitionManagerError> {
    if self.shutting_down.load(Ordering::SeqCst) {
      return Err(PartitionManagerError::ShuttingDown);
    }
    let mut last = self.last_seq.lock().expect("poisoned");
    if seq_no <= *last {
      return Err(PartitionManagerError::OutOfOrder { seq_no, last_seq: *last });
    }
    *last = seq_no;
    Ok(())
  }
}

#[cfg(test)]
mod tests;
