use crate::core::{
  activation::ActivationLease,
  identity::{ClusterIdentity, NodeId},
};

/// Events emitted by cluster subsystems.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClusterEvent {
  /// Activation started on the given node.
  ActivationStarted {
    /// Identity whose activation began.
    identity: ClusterIdentity,
    /// Owning node executing the activation.
    owner:    NodeId,
  },
  /// Activation terminated and lease released.
  ActivationTerminated {
    /// Identity whose activation finished.
    identity: ClusterIdentity,
    /// Lease that was terminated.
    lease:    ActivationLease,
  },
  /// Block list was applied to a node.
  BlockListApplied {
    /// Node that was block-listed.
    node: NodeId,
  },
  /// Retry throttling triggered for a request.
  RetryThrottled {
    /// Identity whose requests exceeded retry budget.
    identity: ClusterIdentity,
  },
}
