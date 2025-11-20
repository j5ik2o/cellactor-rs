use thiserror::Error;

/// Errors surfaced by the partition bridge when routing requests.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PartitionBridgeError {
  /// Bridge is currently backpressured or offline.
  #[error("partition bridge unavailable")]
  Unavailable,
}
