use alloc::{string::ToString, vec, vec::Vec};

use cellactor_utils_core_rs::sync::{ArcShared, NoStdMutex};

use super::ActorSystem;
use crate::{
  NoStdToolbox,
  actor_prim::Actor,
  dead_letter::DeadLetterReason,
  dispatcher::{DispatchExecutor, DispatchSharedGeneric},
  event_stream::{EventStreamEvent, EventStreamSubscriber, EventStreamSubscriptionGeneric, SerializationAuditEvent},
  lifecycle::LifecycleStage,
  messaging::SystemMessage,
  monitoring::serialization_audit_monitor::SerializationAuditMonitor,
  props::{DispatcherConfig, Props},
  serialization::{
    AggregateSchemaBuilder, BincodeSerializer, FieldPath, FieldPathDisplay, FieldPathSegment, SERIALIZATION_EXTENSION,
    SerializationError, SerializerHandle, TraversalPolicy,
  },
  spawn::SpawnError,
  system::system_state::SystemStateGeneric,
  telemetry::serialization_audit_notifier::SerializationAuditNotifier,
};

struct TestActor;

impl Actor for TestActor {
  fn receive(
    &mut self,
    _context: &mut crate::actor_prim::ActorContextGeneric<'_, NoStdToolbox>,
    _message: crate::messaging::AnyMessageView<'_, NoStdToolbox>,
  ) -> Result<(), crate::error::ActorError> {
    Ok(())
  }
}

struct SpawnRecorderActor {
  log: ArcShared<NoStdMutex<Vec<&'static str>>>,
}

impl SpawnRecorderActor {
  fn new(log: ArcShared<NoStdMutex<Vec<&'static str>>>) -> Self {
    Self { log }
  }
}

impl Actor for SpawnRecorderActor {
  fn pre_start(
    &mut self,
    _ctx: &mut crate::actor_prim::ActorContextGeneric<'_, NoStdToolbox>,
  ) -> Result<(), crate::error::ActorError> {
    self.log.lock().push("pre_start");
    Ok(())
  }

  fn receive(
    &mut self,
    _context: &mut crate::actor_prim::ActorContextGeneric<'_, NoStdToolbox>,
    _message: crate::messaging::AnyMessageView<'_, NoStdToolbox>,
  ) -> Result<(), crate::error::ActorError> {
    self.log.lock().push("receive");
    Ok(())
  }
}

struct FailingStartActor;

impl Actor for FailingStartActor {
  fn receive(
    &mut self,
    _context: &mut crate::actor_prim::ActorContextGeneric<'_, NoStdToolbox>,
    _message: crate::messaging::AnyMessageView<'_, NoStdToolbox>,
  ) -> Result<(), crate::error::ActorError> {
    Ok(())
  }

  fn pre_start(
    &mut self,
    _ctx: &mut crate::actor_prim::ActorContextGeneric<'_, NoStdToolbox>,
  ) -> Result<(), crate::error::ActorError> {
    Err(crate::error::ActorError::recoverable("boom"))
  }
}

struct LifecycleEventWatcher {
  stages: ArcShared<NoStdMutex<Vec<LifecycleStage>>>,
}

impl LifecycleEventWatcher {
  fn new(stages: ArcShared<NoStdMutex<Vec<LifecycleStage>>>) -> Self {
    Self { stages }
  }
}

impl EventStreamSubscriber<NoStdToolbox> for LifecycleEventWatcher {
  fn on_event(&self, event: &EventStreamEvent<NoStdToolbox>) {
    if let EventStreamEvent::Lifecycle(lifecycle) = event {
      self.stages.lock().push(lifecycle.stage());
    }
  }
}

struct RecordingSubscriber {
  events: ArcShared<NoStdMutex<Vec<EventStreamEvent<NoStdToolbox>>>>,
}

impl RecordingSubscriber {
  fn new(events: ArcShared<NoStdMutex<Vec<EventStreamEvent<NoStdToolbox>>>>) -> Self {
    Self { events }
  }
}

impl EventStreamSubscriber<NoStdToolbox> for RecordingSubscriber {
  fn on_event(&self, event: &EventStreamEvent<NoStdToolbox>) {
    self.events.lock().push(event.clone());
  }
}

struct RecordingAuditNotifier {
  events: ArcShared<NoStdMutex<Vec<SerializationAuditEvent>>>,
}

impl RecordingAuditNotifier {
  fn new(events: ArcShared<NoStdMutex<Vec<SerializationAuditEvent>>>) -> Self {
    Self { events }
  }
}

impl SerializationAuditNotifier<NoStdToolbox> for RecordingAuditNotifier {
  fn on_serialization_audit(&self, event: &SerializationAuditEvent) {
    self.events.lock().push(event.clone());
  }
}

struct NoopExecutor;

impl NoopExecutor {
  const fn new() -> Self {
    Self
  }
}

impl DispatchExecutor<NoStdToolbox> for NoopExecutor {
  fn execute(&self, _dispatcher: DispatchSharedGeneric<NoStdToolbox>) {}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct ParentAggregate;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct ChildAggregate(u32);

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct ManifestRootA;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct ManifestRootB;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct SharedField(u32);

const PARENT_AGG_SAMPLE: ParentAggregate = ParentAggregate;
const CHILD_AGG_SAMPLE: ChildAggregate = ChildAggregate(0);
const SHARED_FIELD_SAMPLE: SharedField = SharedField(0);

fn decode_child(bytes: &[u8]) -> Result<ChildAggregate, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode::config::standard().with_fixed_int_encoding())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
}

#[test]
fn actor_system_new_empty() {
  let system = ActorSystem::new_empty();
  assert!(!system.state().is_terminated());
}

#[test]
fn actor_system_from_state() {
  let state = crate::system::system_state::SystemState::new();
  let system = ActorSystem::from_state(ArcShared::new(state));
  assert!(!system.state().is_terminated());
}

#[test]
fn actor_system_clone() {
  let system1 = ActorSystem::new_empty();
  let system2 = system1.clone();
  assert!(!system1.state().is_terminated());
  assert!(!system2.state().is_terminated());
}

#[test]
fn actor_system_allocate_pid() {
  let system = ActorSystem::new_empty();
  let pid1 = system.allocate_pid();
  let pid2 = system.allocate_pid();
  assert_ne!(pid1.value(), pid2.value());
}

#[test]
fn actor_system_state() {
  let system = ActorSystem::new_empty();
  let state = system.state();
  assert!(!state.is_terminated());
}

#[test]
fn actor_system_event_stream() {
  let system = ActorSystem::new_empty();
  let stream = system.event_stream();
  let _ = stream;
}

#[test]
fn actor_system_deadletters() {
  let system = ActorSystem::new_empty();
  let deadletters = system.dead_letters();
  assert_eq!(deadletters.len(), 0);
}

#[test]
fn actor_system_emit_log() {
  let system = ActorSystem::new_empty();
  let pid = system.allocate_pid();
  system.emit_log(crate::logging::LogLevel::Info, "test message", Some(pid));
}

#[test]
fn actor_system_when_terminated() {
  let system = ActorSystem::new_empty();
  let future = system.when_terminated();
  assert!(!future.is_ready());
}

#[test]
fn actor_system_actor_ref_for_nonexistent_pid() {
  let system = ActorSystem::new_empty();
  let pid = system.allocate_pid();
  assert!(system.actor_ref(pid).is_none());
}

#[test]
fn actor_system_children_for_nonexistent_parent() {
  let system = ActorSystem::new_empty();
  let parent_pid = system.allocate_pid();
  let children = system.children(parent_pid);
  assert_eq!(children.len(), 0);
}

#[test]
fn actor_system_spawn_child_with_invalid_parent() {
  let system = ActorSystem::new_empty();
  let props = Props::from_fn(|| TestActor);
  let invalid_parent = system.allocate_pid();

  let result = system.spawn_child(invalid_parent, &props);
  assert!(result.is_err());
}

#[test]
fn actor_system_spawn_without_guardian() {
  let system = ActorSystem::new_empty();
  let props = Props::from_fn(|| TestActor);

  let result = system.spawn(&props);
  assert!(result.is_err());
}

#[test]
fn actor_system_drain_ready_ask_futures() {
  let system = ActorSystem::new_empty();
  let futures = system.drain_ready_ask_futures();
  assert_eq!(futures.len(), 0);
}

#[test]
fn actor_system_terminate_without_guardian() {
  let system = ActorSystem::new_empty();
  let result = system.terminate();
  assert!(result.is_ok());
  assert!(system.state().is_terminated());
}

#[test]
fn actor_system_terminate_when_already_terminated() {
  let system = ActorSystem::new_empty();
  system.state().mark_terminated();
  let result = system.terminate();
  assert!(result.is_ok());
}

#[test]
fn spawn_does_not_block_when_dispatcher_never_runs() {
  let system = ActorSystem::new_empty();
  let log: ArcShared<NoStdMutex<Vec<&'static str>>> = ArcShared::new(NoStdMutex::new(Vec::new()));
  let props = Props::from_fn({
    let log = log.clone();
    move || SpawnRecorderActor::new(log.clone())
  })
  .with_dispatcher(DispatcherConfig::from_executor(ArcShared::new(NoopExecutor::new())));

  let child = system.spawn_with_parent(None, &props).expect("spawn succeeds");
  assert!(log.lock().is_empty());
  assert!(system.state().cell(&child.pid()).is_some());
}

#[test]
fn spawn_succeeds_even_if_pre_start_fails() {
  let system = ActorSystem::new_empty();
  let props = Props::from_fn(|| FailingStartActor);
  let child = system.spawn_with_parent(None, &props).expect("spawn succeeds despite failure");

  assert!(system.state().cell(&child.pid()).is_none());
}

#[test]
fn create_send_failure_triggers_rollback() {
  let system = ActorSystem::new_empty();
  let props = Props::from_fn(|| TestActor);
  let pid = system.allocate_pid();
  let name = system.state().assign_name(None, props.name(), pid).expect("name assigned");
  let cell = system.build_cell_for_spawn(pid, None, name, &props);
  system.state().register_cell(cell.clone());

  system.state().remove_cell(&pid);
  let result = system.perform_create_handshake(None, pid, &cell);

  match result {
    | Err(crate::spawn::SpawnError::InvalidProps(reason)) => {
      assert_eq!(reason, super::CREATE_SEND_FAILED);
    },
    | other => panic!("unexpected handshake result: {:?}", other),
  }

  assert!(system.state().cell(&pid).is_none());
  let retry = system.state().assign_name(None, Some(cell.name()), pid);
  assert!(retry.is_ok());
}

#[test]
fn spawn_returns_child_ref_even_if_dispatcher_is_idle() {
  let system = ActorSystem::new_empty();
  let props =
    Props::from_fn(|| TestActor).with_dispatcher(DispatcherConfig::from_executor(ArcShared::new(NoopExecutor::new())));
  let result = system.spawn_with_parent(None, &props);

  assert!(result.is_ok());
}

#[test]
fn lifecycle_events_cover_restart_transitions() {
  let system = ActorSystem::new_empty();
  let stages: ArcShared<NoStdMutex<Vec<LifecycleStage>>> = ArcShared::new(NoStdMutex::new(Vec::new()));
  let subscriber_impl = ArcShared::new(LifecycleEventWatcher::new(stages.clone()));
  let subscriber: ArcShared<dyn EventStreamSubscriber<NoStdToolbox>> = subscriber_impl;
  let _subscription = system.subscribe_event_stream(&subscriber);

  let props = Props::from_fn(|| TestActor);
  let child = system.spawn_with_parent(None, &props).expect("spawn succeeds");
  let pid = child.pid();

  system.state().send_system_message(pid, SystemMessage::Recreate).expect("recreate enqueued");

  let snapshot = stages.lock().clone();
  assert_eq!(snapshot, vec![LifecycleStage::Started, LifecycleStage::Stopped, LifecycleStage::Restarted]);
}

#[test]
fn actor_system_bootstrap_fails_when_serialization_audit_fails() {
  let props = Props::from_fn(|| TestActor);
  let result = ActorSystem::new_with(&props, |system| {
    let serialization = system.extension(&SERIALIZATION_EXTENSION).expect("serialization extension");
    let registry = serialization.registry();
    let mut builder = AggregateSchemaBuilder::<ParentAggregate>::new(
      TraversalPolicy::DepthFirst,
      FieldPathDisplay::from_str("parent").expect("display"),
    );
    builder
      .add_value_field::<ChildAggregate, _>(
        FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
        FieldPathDisplay::from_str("parent.child").expect("display"),
        false,
        |_| &CHILD_AGG_SAMPLE,
      )
      .expect("add child");
    let schema = builder.finish().expect("schema");
    registry.register_aggregate_schema(schema).expect("register schema");
    Ok(())
  });
  assert!(matches!(result, Err(SpawnError::InvalidProps(super::SERIALIZATION_AUDIT_FAILED))));
}

#[test]
fn actor_system_emits_serialization_audit_event_on_success() {
  let props = Props::from_fn(|| TestActor);
  let events: ArcShared<NoStdMutex<Vec<EventStreamEvent<NoStdToolbox>>>> = ArcShared::new(NoStdMutex::new(Vec::new()));
  let subscriber_impl = ArcShared::new(RecordingSubscriber::new(events.clone()));
  let subscription_slot: ArcShared<NoStdMutex<Option<EventStreamSubscriptionGeneric<NoStdToolbox>>>> =
    ArcShared::new(NoStdMutex::new(None));

  let system = ActorSystem::new_with(&props, |system| {
    let subscriber: ArcShared<dyn EventStreamSubscriber<NoStdToolbox>> = subscriber_impl.clone();
    let subscription = system.subscribe_event_stream(&subscriber);
    *subscription_slot.lock() = Some(subscription);

    let serialization = system.extension(&SERIALIZATION_EXTENSION).expect("serialization extension");
    let registry = serialization.registry();
    let handle = SerializerHandle::new(BincodeSerializer::new());
    registry.bind_type::<ChildAggregate, _>(&handle, Some("child".into()), decode_child).expect("bind child");

    let mut builder = AggregateSchemaBuilder::<ParentAggregate>::new(
      TraversalPolicy::DepthFirst,
      FieldPathDisplay::from_str("parent").expect("display"),
    );
    builder
      .add_value_field::<ChildAggregate, _>(
        FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
        FieldPathDisplay::from_str("parent.child").expect("display"),
        false,
        |_| &CHILD_AGG_SAMPLE,
      )
      .expect("add child");
    let schema = builder.finish().expect("schema");
    registry.register_aggregate_schema(schema).expect("register schema");
    Ok(())
  })
  .expect("actor system");

  // keep subscription alive during bootstrap
  let _subscription = subscription_slot.lock().take();

  let captured = events.lock();
  assert!(captured.iter().any(|event| matches!(event, EventStreamEvent::SerializationAudit(audit) if audit.success())));
  let monitor = system.serialization_audit_monitor();
  assert!(monitor.audit_enabled());
  let last_audit = monitor.last_event().expect("last audit");
  assert!(last_audit.success());

  drop(system);
}

#[test]
fn actor_system_reports_serialization_audit_to_all_channels_on_failure() {
  let props = Props::from_fn(|| TestActor);
  let events: ArcShared<NoStdMutex<Vec<EventStreamEvent<NoStdToolbox>>>> = ArcShared::new(NoStdMutex::new(Vec::new()));
  let subscriber_impl = ArcShared::new(RecordingSubscriber::new(events.clone()));
  let telemetry_events: ArcShared<NoStdMutex<Vec<SerializationAuditEvent>>> =
    ArcShared::new(NoStdMutex::new(Vec::new()));
  let subscription_slot: ArcShared<NoStdMutex<Option<EventStreamSubscriptionGeneric<NoStdToolbox>>>> =
    ArcShared::new(NoStdMutex::new(None));
  let state_slot: ArcShared<NoStdMutex<Option<ArcShared<SystemStateGeneric<NoStdToolbox>>>>> =
    ArcShared::new(NoStdMutex::new(None));

  let result = ActorSystem::new_with(&props, |system| {
    let subscriber: ArcShared<dyn EventStreamSubscriber<NoStdToolbox>> = subscriber_impl.clone();
    let subscription = system.subscribe_event_stream(&subscriber);
    *subscription_slot.lock() = Some(subscription);
    let notifier: ArcShared<dyn SerializationAuditNotifier<NoStdToolbox>> =
      ArcShared::new(RecordingAuditNotifier::new(telemetry_events.clone()));
    system.set_serialization_audit_notifier(notifier);
    *state_slot.lock() = Some(system.state());

    let serialization = system.extension(&SERIALIZATION_EXTENSION).expect("serialization extension");
    let registry = serialization.registry();

    let mut builder = AggregateSchemaBuilder::<ParentAggregate>::new(
      TraversalPolicy::DepthFirst,
      FieldPathDisplay::from_str("parent").expect("display"),
    );
    builder
      .add_value_field::<ChildAggregate, _>(
        FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
        FieldPathDisplay::from_str("parent.child").expect("display"),
        false,
        |_| &CHILD_AGG_SAMPLE,
      )
      .expect("add child");
    let schema = builder.finish().expect("schema");
    registry.register_aggregate_schema(schema).expect("register schema");
    Ok(())
  });

  assert!(matches!(result, Err(SpawnError::InvalidProps(super::SERIALIZATION_AUDIT_FAILED))));
  let _ = subscription_slot.lock().take();

  let captured = events.lock();
  assert!(captured.iter().any(
    |event| matches!(event, EventStreamEvent::SerializationAudit(audit) if !audit.success() && !audit.issues.is_empty())
  ));
  assert!(captured.iter().any(|event| matches!(event, EventStreamEvent::Log(_))));
  assert!(captured.iter().any(
    |event| matches!(event, EventStreamEvent::DeadLetter(entry) if entry.reason() == DeadLetterReason::ExplicitRouting)
  ));
  let telemetry = telemetry_events.lock();
  assert_eq!(telemetry.len(), 1);
  assert!(!telemetry[0].success());
  drop(telemetry);
  let state_handle = state_slot.lock().take().expect("state handle");
  assert!(!state_handle.dead_letters().is_empty());
  let monitor = SerializationAuditMonitor::new(state_handle.clone());
  let last_event = monitor.last_event().expect("last audit");
  assert!(monitor.audit_enabled());
  assert!(!last_event.success());
}

#[test]
fn actor_system_skips_serialization_audit_when_disabled() {
  let props = Props::from_fn(|| TestActor);
  let events: ArcShared<NoStdMutex<Vec<EventStreamEvent<NoStdToolbox>>>> = ArcShared::new(NoStdMutex::new(Vec::new()));
  let subscriber_impl = ArcShared::new(RecordingSubscriber::new(events.clone()));
  const NAME: &str = "parent";
  let subscription_slot: ArcShared<NoStdMutex<Option<EventStreamSubscriptionGeneric<NoStdToolbox>>>> =
    ArcShared::new(NoStdMutex::new(None));

  let system = ActorSystem::new_with(&props, |system| {
    system.set_serialization_audit_enabled(false);
    let subscriber: ArcShared<dyn EventStreamSubscriber<NoStdToolbox>> = subscriber_impl.clone();
    let subscription = system.subscribe_event_stream(&subscriber);
    *subscription_slot.lock() = Some(subscription);

    let serialization = system.extension(&SERIALIZATION_EXTENSION).expect("serialization extension");
    let registry = serialization.registry();
    let mut builder = AggregateSchemaBuilder::<ParentAggregate>::new(
      TraversalPolicy::DepthFirst,
      FieldPathDisplay::from_str(NAME).expect("display"),
    );
    builder
      .add_value_field::<ChildAggregate, _>(
        FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
        FieldPathDisplay::from_str("parent.child").expect("display"),
        false,
        |_| &CHILD_AGG_SAMPLE,
      )
      .expect("add child");
    registry.register_aggregate_schema(builder.finish().expect("schema")).expect("register schema");
    Ok(())
  })
  .expect("actor system");
  let _subscription = subscription_slot.lock().take();

  let captured = events.lock();
  assert!(!captured.iter().any(|event| matches!(event, EventStreamEvent::SerializationAudit(_))));
  let monitor = system.serialization_audit_monitor();
  assert!(!monitor.audit_enabled());
  assert!(monitor.last_event().is_none());

  drop(system);
}

#[test]
fn actor_system_routes_serialization_audit_to_custom_notifier() {
  let props = Props::from_fn(|| TestActor);
  let audit_events: ArcShared<NoStdMutex<Vec<SerializationAuditEvent>>> = ArcShared::new(NoStdMutex::new(Vec::new()));
  let system = ActorSystem::new_with(&props, |system| {
    let notifier: ArcShared<dyn SerializationAuditNotifier<NoStdToolbox>> =
      ArcShared::new(RecordingAuditNotifier::new(audit_events.clone()));
    system.set_serialization_audit_notifier(notifier);

    let serialization = system.extension(&SERIALIZATION_EXTENSION).expect("serialization extension");
    let registry = serialization.registry();
    let handle = SerializerHandle::new(BincodeSerializer::new());
    registry.bind_type::<ChildAggregate, _>(&handle, Some("child".into()), decode_child).expect("bind child");

    let mut builder = AggregateSchemaBuilder::<ParentAggregate>::new(
      TraversalPolicy::DepthFirst,
      FieldPathDisplay::from_str("parent").expect("display"),
    );
    builder
      .add_value_field::<ChildAggregate, _>(
        FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
        FieldPathDisplay::from_str("parent.child").expect("display"),
        false,
        |_| &CHILD_AGG_SAMPLE,
      )
      .expect("add child");
    let schema = builder.finish().expect("schema");
    registry.register_aggregate_schema(schema).expect("register schema");
    Ok(())
  })
  .expect("actor system");

  let monitor = system.serialization_audit_monitor();
  assert!(monitor.last_event().is_some());
  let recorded = audit_events.lock();
  assert_eq!(recorded.len(), 1);
  assert!(recorded[0].success());

  drop(system);
}

#[test]
fn actor_system_bootstrap_fails_when_serialization_cycle_detected() {
  let props = Props::from_fn(|| TestActor);
  let result = ActorSystem::new_with(&props, |system| {
    let serialization = system.extension(&SERIALIZATION_EXTENSION).expect("serialization extension");
    let registry = serialization.registry();

    let mut parent_builder = AggregateSchemaBuilder::<ParentAggregate>::new(
      TraversalPolicy::DepthFirst,
      FieldPathDisplay::from_str("parent").expect("display"),
    );
    parent_builder
      .add_value_field::<ChildAggregate, _>(
        FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
        FieldPathDisplay::from_str("parent.child").expect("display"),
        false,
        |_| &CHILD_AGG_SAMPLE,
      )
      .expect("add child");

    let mut child_builder = AggregateSchemaBuilder::<ChildAggregate>::new(
      TraversalPolicy::DepthFirst,
      FieldPathDisplay::from_str("child").expect("display"),
    );
    child_builder
      .add_value_field::<ParentAggregate, _>(
        FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
        FieldPathDisplay::from_str("child.parent").expect("display"),
        false,
        |_| &PARENT_AGG_SAMPLE,
      )
      .expect("add parent");

    let _ = registry.register_aggregate_schema(parent_builder.finish().expect("parent schema"));
    let _ = registry.register_aggregate_schema(child_builder.finish().expect("child schema"));
    Ok(())
  });
  assert!(matches!(result, Err(SpawnError::InvalidProps(super::SERIALIZATION_AUDIT_FAILED))));
}

#[test]
fn actor_system_bootstrap_fails_when_manifest_collides() {
  let props = Props::from_fn(|| TestActor);
  let result = ActorSystem::new_with(&props, |system| {
    let serialization = system.extension(&SERIALIZATION_EXTENSION).expect("serialization extension");
    let registry = serialization.registry();

    let mut builder_a = AggregateSchemaBuilder::<ManifestRootA>::new(
      TraversalPolicy::DepthFirst,
      FieldPathDisplay::from_str("root_a").expect("display"),
    );
    builder_a
      .add_value_field::<SharedField, _>(
        FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
        FieldPathDisplay::from_str("shared.manifest").expect("display"),
        false,
        |_| &SHARED_FIELD_SAMPLE,
      )
      .expect("add shared field");

    let mut builder_b = AggregateSchemaBuilder::<ManifestRootB>::new(
      TraversalPolicy::DepthFirst,
      FieldPathDisplay::from_str("root_b").expect("display"),
    );
    builder_b
      .add_value_field::<SharedField, _>(
        FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
        FieldPathDisplay::from_str("shared.manifest").expect("display"),
        false,
        |_| &SHARED_FIELD_SAMPLE,
      )
      .expect("add shared field");

    let _ = registry.register_aggregate_schema(builder_a.finish().expect("schema"));
    let _ = registry.register_aggregate_schema(builder_b.finish().expect("schema"));
    Ok(())
  });
  assert!(matches!(result, Err(SpawnError::InvalidProps(super::SERIALIZATION_AUDIT_FAILED))));
}
