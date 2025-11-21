use crate::core::provisioning::descriptor::{ProviderDescriptor, ProviderId, ProviderKind};
use crate::std::provisioning::provider_validator::{ConnectivityChecker, ProviderValidationError, ProviderValidator};

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

fn desc(kind: ProviderKind) -> ProviderDescriptor {
  ProviderDescriptor::new(ProviderId::new("provider"), kind, 1)
    .with_endpoint("http://endpoint")
}

#[test]
fn validates_inmemory_without_connectivity() {
  let validator = ProviderValidator::new(OkChecker);
  validator.validate(&desc(ProviderKind::InMemory)).unwrap();
}

#[test]
fn fails_on_connectivity_error() {
  let validator = ProviderValidator::new(FailChecker);
  let err = validator.validate(&desc(ProviderKind::Consul)).unwrap_err();
  assert!(matches!(err, ProviderValidationError::Connectivity(msg) if msg == "no route"));
}

#[test]
fn missing_endpoint_for_consul_is_rejected() {
  let validator = ProviderValidator::new(OkChecker);
  let bad = ProviderDescriptor::new(ProviderId::new("consul"), ProviderKind::Consul, 1);
  let err = validator.validate(&bad).unwrap_err();
  assert_eq!(ProviderValidationError::MissingEndpoint, err);
}
