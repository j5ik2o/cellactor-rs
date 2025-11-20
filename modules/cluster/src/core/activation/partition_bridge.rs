use fraktor_utils_rs::core::runtime_toolbox::RuntimeToolbox;

use super::{
  activation_request::ActivationRequest, activation_response::ActivationResponse,
  partition_bridge_error::PartitionBridgeError,
};

#[cfg(test)]
mod tests;

/// Abstraction that forwards activation requests to the partition/placement layer.
pub trait PartitionBridge<TB>: Send + Sync + 'static
where
  TB: RuntimeToolbox + 'static, {
  /// Sends an activation request to the placement actor.
  ///
  /// # Errors
  ///
  /// Returns `PartitionBridgeError` if the request could not be delivered to the placement layer.
  fn send_activation_request(&self, request: ActivationRequest<TB>) -> Result<(), PartitionBridgeError>;

  /// Publishes an activation response back to the runtime.
  fn handle_activation_response(&self, response: &ActivationResponse);
}
