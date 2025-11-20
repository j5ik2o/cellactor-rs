//! Activation management primitives.

/// Activation lease container.
pub mod activation_lease;
/// Activation error enumeration for responses.
pub mod activation_error;
/// Incoming activation request payloads.
pub mod activation_request;
/// Activation responses emitted by PlacementActor.
pub mod activation_response;
/// Ledger storing activation leases.
pub mod activation_ledger;
/// Lease identifier newtype.
pub mod lease_id;
/// Lease status enumeration.
pub mod lease_status;
/// Errors emitted by the ledger when operations fail。
pub mod ledger_error;
/// Bridge trait forwarding requests to partition/placement層。
pub mod partition_bridge;
/// Errors produced by the partition bridge。
pub mod partition_bridge_error;

pub use activation_lease::ActivationLease;
pub use activation_error::ActivationError;
pub use activation_request::ActivationRequest;
pub use activation_response::ActivationResponse;
pub use activation_ledger::ActivationLedger;
pub use lease_id::LeaseId;
pub use lease_status::LeaseStatus;
pub use ledger_error::LedgerError;
pub use partition_bridge::PartitionBridge;
pub use partition_bridge_error::PartitionBridgeError;
