use thiserror::Error;

use super::activation_lease::ActivationLease;

/// Errors emitted when manipulating activation leases.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LedgerError {
    /// The identity is already locked by another owner.
    #[error("identity already owns an activation lease")]
    AlreadyOwned {
        /// Existing lease preventing another acquisition.
        existing: ActivationLease,
    },
}
