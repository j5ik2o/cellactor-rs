use alloc::vec::Vec;
use std::sync::Mutex;

use fraktor_actor_rs::core::{
  actor_prim::{Actor, ActorContextGeneric},
  error::ActorError,
  messaging::AnyMessageViewGeneric,
  props::PropsGeneric,
};
use fraktor_utils_rs::core::runtime_toolbox::NoStdToolbox;

use crate::core::{
  activation::{
    ActivationError, ActivationLease, ActivationRequest, ActivationResponse, LeaseId, LeaseStatus, PartitionBridge,
    PartitionBridgeError,
  },
  identity::{ClusterIdentity, NodeId},
};

struct RecordingBridge {
  requests:  Mutex<Vec<ClusterIdentity>>,
  responses: Mutex<Vec<ActivationResponse>>,
}

impl RecordingBridge {
  fn new() -> Self {
    Self { requests: Mutex::new(Vec::new()), responses: Mutex::new(Vec::new()) }
  }
}

impl PartitionBridge<NoStdToolbox> for RecordingBridge {
  fn send_activation_request(&self, request: ActivationRequest<NoStdToolbox>) -> Result<(), PartitionBridgeError> {
    self.requests.lock().unwrap().push(request.identity().clone());
    Ok(())
  }

  fn handle_activation_response(&self, response: ActivationResponse) {
    self.responses.lock().unwrap().push(response);
  }
}

struct TestActor;

impl Actor<NoStdToolbox> for TestActor {
  fn receive(
    &mut self,
    _ctx: &mut ActorContextGeneric<'_, NoStdToolbox>,
    _message: AnyMessageViewGeneric<'_, NoStdToolbox>,
  ) -> Result<(), ActorError> {
    Ok(())
  }
}

fn sample_request() -> ActivationRequest<NoStdToolbox> {
  let identity = ClusterIdentity::new("echo", "id");
  let lease = ActivationLease::new(LeaseId::new(1), NodeId::new("node-a"), 7, LeaseStatus::Active);
  let props = PropsGeneric::<NoStdToolbox>::from_fn(|| TestActor);
  ActivationRequest::new(identity, lease, props)
}

#[test]
fn records_requests() {
  let bridge = RecordingBridge::new();
  bridge.send_activation_request(sample_request()).expect("send");
  let requests = bridge.requests.lock().unwrap();
  assert_eq!(requests.len(), 1);
  assert_eq!(requests[0].identity(), "id");
}

#[test]
fn records_responses() {
  let bridge = RecordingBridge::new();
  let response =
    ActivationResponse::failure(ClusterIdentity::new("echo", "id"), ActivationError::SpawnFailed, LeaseId::new(1), 10);
  bridge.handle_activation_response(response.clone());
  let responses = bridge.responses.lock().unwrap();
  assert_eq!(responses.len(), 1);
  assert_eq!(responses[0], response);
}
