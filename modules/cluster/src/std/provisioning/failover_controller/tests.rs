use std::time::Duration;

use crate::core::provisioning::descriptor::{ProviderDescriptor, ProviderId, ProviderKind};
use crate::core::provisioning::snapshot::ProviderHealth;
use crate::std::provisioning::failover_controller::{FailoverConfig, FailoverController};

fn desc(id: &str, prio: u8) -> ProviderDescriptor {
  ProviderDescriptor::new(ProviderId::new(id), ProviderKind::InMemory, prio)
}

fn cfg() -> FailoverConfig {
  FailoverConfig::default()
}

#[test]
fn selects_highest_priority_healthy() {
  let mut fc = FailoverController::new(vec![desc("a", 1), desc("b", 5)], cfg());
  let active = fc.select_active().unwrap();
  assert_eq!("b", active.id().as_str());
}

#[test]
fn fails_over_after_max_errors() {
  let mut fc = FailoverController::new(vec![desc("a", 5), desc("b", 1)], cfg());
  // three failures on a -> unreachable -> choose b
  fc.record_failure("a", "timeout");
  fc.record_failure("a", "timeout");
  fc.record_failure("a", "timeout");
  let active = fc.select_active().unwrap();
  assert_eq!("b", active.id().as_str());
}

#[test]
fn cooldown_restores_primary() {
  let mut cfg = cfg();
  cfg.cooldown = Duration::from_millis(1);
  let mut fc = FailoverController::new(vec![desc("a", 5), desc("b", 1)], cfg);
  fc.record_failure("a", "timeout");
  fc.record_failure("a", "timeout");
  fc.record_failure("a", "timeout");
  // immediately after failures, b is selected
  assert_eq!("b", fc.select_active().unwrap().id().as_str());
  // wait for cooldown
  std::thread::sleep(Duration::from_millis(5));
  let active = fc.select_active().unwrap();
  assert_eq!("a", active.id().as_str());
  assert_eq!(ProviderHealth::Healthy, fc.states[0].health);
}

#[test]
fn backoff_blocks_until_retry_window() {
  let mut cfg = cfg();
  cfg.backoff_init = Duration::from_millis(5);
  cfg.max_errors = 1;
  let mut fc = FailoverController::new(vec![desc("primary", 5), desc("backup", 1)], cfg);

  fc.record_failure("primary", "timeout");
  // immediately after failure the primary should be skipped due to backoff
  let active = fc.select_active().unwrap();
  assert_eq!("backup", active.id().as_str());

  std::thread::sleep(Duration::from_millis(6));
  // after backoff expires primary degradesから回復途上として再挑戦できる
  let active2 = fc.select_active().unwrap();
  assert_eq!("primary", active2.id().as_str());
}

#[test]
fn snapshot_delay_degrades_provider() {
  let mut cfg = cfg();
  let mut fc = FailoverController::new(vec![desc("slow", 1), desc("fast", 2)], cfg);

  // latency below threshold -> remains healthy
  fc.record_snapshot_delay("slow", Duration::from_millis(5), Duration::from_millis(10));
  assert_eq!(ProviderHealth::Healthy, fc.states[1].health);

  // latency exceeds threshold -> degrade
  fc.record_snapshot_delay("slow", Duration::from_millis(20), Duration::from_millis(10));
  assert_eq!(ProviderHealth::Degraded, fc.states[1].health);

  // active selection prefers higher priority healthy (fast)
  let active = fc.select_active().unwrap();
  assert_eq!("fast", active.id().as_str());
}
