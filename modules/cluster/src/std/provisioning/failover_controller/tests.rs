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
