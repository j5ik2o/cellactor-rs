use thiserror::Error;

use crate::core::activation::ActivationLease;

/// Errors encountered while resolving cluster identities.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ResolveError {
  /// The hash ring is currently empty, so no owner can be determined.
  #[error("hash ring contains no nodes")]
  RingEmpty,
  /// A conflicting lease already exists for the identity.
  #[error("activation lease already exists for identity")]
  LeaseConflict {
    /// Existing lease that currently guards the identity.
    existing: ActivationLease,
  },
  /// Runtime is shutting down and no new activations are accepted.
  #[error("cluster runtime is shutting down")]
  ShuttingDown,
}
