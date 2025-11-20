//! Activation management primitives.

/// Activation lease container.
pub mod activation_lease;
/// Ledger storing activation leases.
pub mod activation_ledger;
/// Lease identifier newtype.
pub mod lease_id;
/// Lease status enumeration.
pub mod lease_status;
/// Errors emitted by the ledger when operations fail.
pub mod ledger_error;

pub use activation_lease::ActivationLease;
pub use activation_ledger::ActivationLedger;
pub use lease_id::LeaseId;
pub use lease_status::LeaseStatus;
pub use ledger_error::LedgerError;
