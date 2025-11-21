//! FailoverController と複数 ProviderStream を統合して配信するランナー。

extern crate alloc;
extern crate std;

use alloc::{collections::BTreeMap, sync::Arc};
use std::time::Duration;

use crate::{
  core::provisioning::descriptor::ProviderDescriptor,
  std::provisioning::{
    failover_controller::FailoverController,
    partition_manager_bridge::PartitionManagerBridge,
    placement_supervisor_bridge::PlacementSupervisorBridge,
    provider_stream::ProviderStream,
    provider_stream_driver::{ProviderStreamDriver, StreamProgress},
    provider_watch_hub::ProviderWatchHub,
    provisioning_metrics::ProvisioningMetrics,
  },
};

/// ランナー全体の進捗。
pub enum RunnerProgress {
  /// イベントを処理した。
  Advanced,
  /// バックアップへ切替えた。
  Switched,
  /// すべてのプロバイダが尽きて終了。
  Exhausted,
  /// イベントなし。
  Pending,
}

/// ランナー。
pub struct ProviderStreamRunner {
  failover:   FailoverController,
  drivers:    BTreeMap<String, ProviderStreamDriver>,
  hub:        Arc<ProviderWatchHub>,
  placement:  Arc<PlacementSupervisorBridge>,
  partition:  Arc<PartitionManagerBridge>,
  metrics:    Arc<ProvisioningMetrics>,
  current:    Option<String>,
  seq_no:     u64,
  delay_warn: Duration,
}

impl ProviderStreamRunner {
  /// 作成する。`delay_warn` はスナップショット遅延警告閾値。
  pub fn new(
    failover: FailoverController,
    streams: Vec<(ProviderDescriptor, Box<dyn ProviderStream>)>,
    hub: Arc<ProviderWatchHub>,
    placement: Arc<PlacementSupervisorBridge>,
    partition: Arc<PartitionManagerBridge>,
    metrics: Arc<ProvisioningMetrics>,
    delay_warn: Duration,
  ) -> Self {
    let mut drivers = BTreeMap::new();
    for (desc, stream) in streams {
      drivers.insert(
        desc.id().as_str().to_string(),
        ProviderStreamDriver::new(stream, hub.clone(), placement.clone(), partition.clone(), metrics.clone()),
      );
    }
    Self { failover, drivers, hub, placement, partition, metrics, current: None, seq_no: 0, delay_warn }
  }

  /// 現在のアクティブプロバイダを選択（必要なら切替）する。
  fn ensure_active(&mut self) -> bool {
    if let Some(cur) = &self.current {
      if self.failover.select_active().map(|d| d.id().as_str().to_string()) == Some(cur.clone()) {
        return true;
      }
    }
    if let Some(desc) = self.failover.select_active() {
      let next_id = desc.id().as_str().to_string();
      if let Some(prev) = self.current.replace(next_id.clone()) {
        // provider_changed 通知（seqを進めて発火）
        self.seq_no = self.seq_no.saturating_add(1);
        let from = ProviderDescriptor::new(
          crate::core::provisioning::descriptor::ProviderId::new(prev.clone()),
          desc.kind().clone(),
          desc.priority(),
        )
        .id()
        .clone();
        let to = desc.id().clone();
        let _ = self.placement.provider_changed(self.seq_no, from.clone(), to.clone());
        let _ = self.partition.provider_changed(self.seq_no, from, to);
        self.metrics.record_failover(self.seq_no);
      }
      return true;
    }
    false
  }

  /// 1 ステップ実行する。
  pub fn pump_once(&mut self) -> RunnerProgress {
    if !self.ensure_active() {
      return RunnerProgress::Exhausted;
    }
    let cur_id = self.current.clone().unwrap();
    let driver = match self.drivers.get_mut(&cur_id) {
      | Some(d) => d,
      | None => return RunnerProgress::Exhausted,
    };

    match driver.pump_once(&mut self.seq_no) {
      | Ok(StreamProgress::Advanced) => RunnerProgress::Advanced,
      | Ok(StreamProgress::Pending) => RunnerProgress::Pending,
      | Ok(StreamProgress::Terminated) => {
        // terminate -> mark failure and switch
        self.failover.record_failure(&cur_id, "terminated");
        if self.ensure_active() { RunnerProgress::Switched } else { RunnerProgress::Exhausted }
      },
      | Err(_) => {
        // エラー時は failover して次へ
        self.failover.record_failure(&cur_id, "stream-error");
        if self.ensure_active() { RunnerProgress::Switched } else { RunnerProgress::Exhausted }
      },
    }
  }
}

#[cfg(test)]
mod tests;
