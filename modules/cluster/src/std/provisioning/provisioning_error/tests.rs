use crate::std::provisioning::provisioning_error::{ProvisioningError, ProvisioningErrorCode};

#[test]
fn retains_code_and_message() {
  let err = ProvisioningError::new(ProvisioningErrorCode::Validation, "missing endpoint");
  assert_eq!(ProvisioningErrorCode::Validation, err.code);
  assert_eq!("missing endpoint", err.message);
}
