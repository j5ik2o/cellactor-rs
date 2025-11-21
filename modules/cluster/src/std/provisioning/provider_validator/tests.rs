use crate::core::provisioning::descriptor::{ProviderDescriptor, ProviderId, ProviderKind};
use crate::std::provisioning::provider_validator::{
  CapabilityChecker, ConnectivityChecker, ProviderValidator, ValidationResult,
};
use crate::std::provisioning::provisioning_error::{ProvisioningError, ProvisioningErrorCode};

struct OkChecker;
impl ConnectivityChecker for OkChecker {
  fn check(&self, _descriptor: &ProviderDescriptor) -> Result<(), String> {
    Ok(())
  }
}

struct FailChecker;
impl ConnectivityChecker for FailChecker {
  fn check(&self, _descriptor: &ProviderDescriptor) -> Result<(), String> {
    Err("no route".to_string())
  }
}

struct WatchCapOk;
impl CapabilityChecker for WatchCapOk {
  fn check(&self, _descriptor: &ProviderDescriptor) -> Result<(), String> {
    Ok(())
  }
}

struct WatchCapMissing;
impl CapabilityChecker for WatchCapMissing {
  fn check(&self, _descriptor: &ProviderDescriptor) -> Result<(), String> {
    Err("watch capability missing".to_string())
  }
}

fn desc(kind: ProviderKind) -> ProviderDescriptor {
  ProviderDescriptor::new(ProviderId::new("provider"), kind, 1)
    .with_endpoint("http://endpoint")
}

#[test]
fn validates_inmemory_without_connectivity() {
  let validator = ProviderValidator::new(OkChecker, WatchCapOk);
  let res = validator.validate(&desc(ProviderKind::InMemory)).unwrap();
  assert_eq!(None, res.disabled_reason);
}

#[test]
fn fails_on_connectivity_error() {
  let validator = ProviderValidator::new(FailChecker, WatchCapOk);
  let err = validator.validate(&desc(ProviderKind::Consul)).unwrap_err();
  assert_eq!(ProvisioningErrorCode::Connectivity, err.code);
  assert_eq!("no route", err.message);
}

#[test]
fn missing_endpoint_for_consul_is_rejected() {
  let validator = ProviderValidator::new(OkChecker, WatchCapOk);
  let bad = ProviderDescriptor::new(ProviderId::new("consul"), ProviderKind::Consul, 1);
  let err = validator.validate(&bad).unwrap_err();
  assert_eq!(ProvisioningError { code: ProvisioningErrorCode::Validation, message: "missing endpoint".to_string() }, err);
}

#[test]
fn capability_missing_marks_disabled() {
  let validator = ProviderValidator::new(OkChecker, WatchCapMissing);
  let res: ValidationResult = validator.validate(&desc(ProviderKind::InMemory)).unwrap();
  assert_eq!(Some("watch capability missing".to_string()), res.disabled_reason);
}
