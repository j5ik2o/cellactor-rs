use alloc::vec::Vec;

use fraktor_actor_rs::core::{
  actor_prim::{Actor, ActorContextGeneric, Pid},
  error::ActorError,
  messaging::AnyMessageViewGeneric,
  props::PropsGeneric,
};
use fraktor_utils_rs::core::runtime_toolbox::NoStdToolbox;

use crate::core::{
  activation::{ActivationError, ActivationLease, ActivationRequest, LeaseId, LeaseStatus},
  identity::{ClusterIdentity, NodeId},
  placement::{PlacementActor, PlacementSpawner, PlacementSpawnerError},
};

struct RecordingSpawner {
  responses: Vec<Result<Pid, PlacementSpawnerError>>,
}

impl RecordingSpawner {
  fn new(responses: Vec<Result<Pid, PlacementSpawnerError>>) -> Self {
    Self { responses }
  }
}

impl PlacementSpawner<NoStdToolbox> for RecordingSpawner {
  fn spawn(&self, _: &ClusterIdentity, _: PropsGeneric<NoStdToolbox>) -> Result<Pid, PlacementSpawnerError> {
    self.responses.get(0).cloned().unwrap_or_else(|| Err(PlacementSpawnerError::SpawnFailed))
  }
}

fn sample_request() -> ActivationRequest<NoStdToolbox> {
  let identity = ClusterIdentity::new("echo", "grain");
  let lease = ActivationLease::new(LeaseId::new(7), NodeId::new("node-a"), 3, LeaseStatus::Active);
  let props = PropsGeneric::<NoStdToolbox>::from_fn(|| TestActor);
  ActivationRequest::new(identity, lease, props)
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

#[test]
fn successful_activation_response_contains_pid() {
  let spawner = RecordingSpawner::new(vec![Ok(Pid::new(1, 0))]);
  let actor = PlacementActor::<NoStdToolbox, _>::new(spawner);

  let response = actor.handle_activation(sample_request());

  assert_eq!(response.pid(), Some(Pid::new(1, 0)));
  assert!(response.error().is_none());
}

#[test]
fn failed_activation_response_propagates_error() {
  let spawner = RecordingSpawner::new(vec![Err(PlacementSpawnerError::UnknownKind)]);
  let actor = PlacementActor::<NoStdToolbox, _>::new(spawner);

  let response = actor.handle_activation(sample_request());

  assert_eq!(response.pid(), None);
  assert_eq!(response.error(), Some(&ActivationError::UnknownKind));
}

#[test]
fn terminated_handler_returns_failure() {
  let spawner = RecordingSpawner::new(vec![]);
  let actor = PlacementActor::<NoStdToolbox, _>::new(spawner);
  let response = actor.handle_terminated(ClusterIdentity::new("echo", "grain"), LeaseId::new(9), 11);

  assert_eq!(response.error(), Some(&ActivationError::Terminated));
}
