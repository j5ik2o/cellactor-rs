//! Activation management primitives.

/// Activation error enumeration for responses.
mod activation_error;
/// Activation lease container.
mod activation_lease;
/// Ledger storing activation leases.
mod activation_ledger;
/// Incoming activation request payloads.
mod activation_request;
/// Activation responses emitted by PlacementActor.
mod activation_response;
/// Lease identifier newtype.
mod lease_id;
/// Lease status enumeration.
mod lease_status;
/// Errors emitted by the ledger when operations fail.
mod ledger_error;
/// Bridge trait forwarding requests to partition/placement layer.
mod partition_bridge;
/// Errors produced by the partition bridge.
mod partition_bridge_error;

pub use activation_error::ActivationError;
pub use activation_lease::ActivationLease;
pub use activation_ledger::ActivationLedger;
pub use activation_request::ActivationRequest;
pub use activation_response::ActivationResponse;
pub use lease_id::LeaseId;
pub use lease_status::LeaseStatus;
pub use ledger_error::LedgerError;
pub use partition_bridge::PartitionBridge;
pub use partition_bridge_error::PartitionBridgeError;
