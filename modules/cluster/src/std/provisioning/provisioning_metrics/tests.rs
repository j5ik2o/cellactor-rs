use std::time::Duration;

use crate::std::provisioning::provisioning_metrics::ProvisioningMetrics;

#[test]
fn records_all_counters_with_seq() {
  let metrics = ProvisioningMetrics::new();

  metrics.record_snapshot_latency(1, Duration::from_millis(5));
  metrics.record_failover(2);
  metrics.record_stream_interrupt(3);

  assert_eq!(vec![(1, Duration::from_millis(5))], metrics.snapshot_latencies());
  assert_eq!(vec![2], metrics.failovers());
  assert_eq!(vec![3], metrics.interruptions());
}
