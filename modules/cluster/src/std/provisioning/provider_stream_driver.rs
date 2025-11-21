//! ProviderStream をウォッチハブと各ブリッジへ中継するドライバ。

extern crate alloc;
extern crate std;

use alloc::sync::Arc;
use std::time::{Duration, Instant};

use crate::std::provisioning::partition_manager_bridge::{PartitionManagerBridge, PartitionManagerError};
use crate::std::provisioning::placement_supervisor_bridge::{PlacementBridgeError, PlacementSupervisorBridge};
use crate::std::provisioning::provider_event::ProviderEvent;
use crate::std::provisioning::provider_stream::ProviderStream;
use crate::std::provisioning::provider_watch_hub::{ProviderWatchHub, WatchError};
use crate::std::provisioning::provisioning_metrics::ProvisioningMetrics;

/// ストリームを1ステップ駆動した結果。
pub enum StreamProgress {
  /// 新しいイベントを処理した。
  Advanced,
  /// 終了イベントを処理し、ストリームが停止した。
  Terminated,
  /// イベントが無く idle。
  Pending,
}

/// ドライバ内で発生し得るエラー。
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum StreamError {
  /// WatchHub で拒否された。
  #[error("watch error: {0:?}")]
  Watch(WatchError),
  /// Placement ブリッジで順序違反などが発生。
  #[error("placement error")]
  Placement,
  /// Partition ブリッジで順序違反などが発生。
  #[error("partition error")]
  Partition,
}

/// ProviderStream をハブと各ブリッジへ中継する単純ドライバ。
pub struct ProviderStreamDriver<S: ProviderStream> {
  stream:        S,
  hub:           Arc<ProviderWatchHub>,
  placement:     Arc<PlacementSupervisorBridge>,
  partition:     Arc<PartitionManagerBridge>,
  metrics:       Arc<ProvisioningMetrics>,
  seq_no:        u64,
  started_at:    Instant,
}

impl<S: ProviderStream> ProviderStreamDriver<S> {
  /// 新しいドライバを作成。
  pub fn new(
    stream:    S,
    hub:       Arc<ProviderWatchHub>,
    placement: Arc<PlacementSupervisorBridge>,
    partition: Arc<PartitionManagerBridge>,
    metrics:   Arc<ProvisioningMetrics>,
  ) -> Self {
    Self { stream, hub, placement, partition, metrics, seq_no: 0, started_at: Instant::now() }
  }

  /// イベントを1件処理する。戻り値で進捗を示す。
  pub fn pump_once(&mut self) -> Result<StreamProgress, StreamError> {
    let Some(event) = self.stream.next_event() else {
      return Ok(StreamProgress::Pending);
    };

    match &event {
      ProviderEvent::Snapshot(snapshot) => {
        self.seq_no = self.seq_no.saturating_add(1);
        // 記録: latency は簡易に取得時刻差を利用
        let now = Instant::now();
        self
          .metrics
          .record_snapshot_latency(self.seq_no, now.saturating_duration_since(self.started_at));

        self.hub.apply_event(event).map_err(StreamError::Watch)?;

        if let Some((snap, invalid)) = self.hub.latest_snapshot_with_invalidation() {
          // ハッシュが変わらない場合は配信を省略
          if !invalid {
            return Ok(StreamProgress::Advanced);
          }
          self
            .placement
            .apply_snapshot(self.seq_no, &snap)
            .map_err(|_| StreamError::Placement)?;
          self.partition.apply_snapshot(self.seq_no, snap).map_err(|_| StreamError::Partition)?;
        }
        Ok(StreamProgress::Advanced)
      },
      ProviderEvent::Terminated { .. } => {
        self.hub.apply_event(event).map_err(StreamError::Watch)?;
        self.metrics.record_stream_interrupt(self.seq_no);
        Ok(StreamProgress::Terminated)
      },
    }
  }
}

#[cfg(test)]
mod tests;
