use core::{
  num::NonZeroUsize,
  pin::Pin,
  task::{Context, Poll},
};

use cellactor_utils_core_rs::sync::ArcShared;
use portable_atomic::AtomicU8;

use super::{
  dispatch_executor::DispatchExecutor, dispatcher_sender::block_hint, dispatcher_state::DispatcherState,
  schedule_waker::ScheduleWaker,
};
use crate::{
  ActorRuntimeMutex,
  actor_error::ActorError,
  any_message::AnyMessage,
  mailbox::{EnqueueOutcome, Mailbox, MailboxMessage, MailboxOfferFuture},
  message_invoker::MessageInvoker,
  send_error::SendError,
  system_message::SystemMessage,
};

const DEFAULT_THROUGHPUT: usize = 300;

/// Entity that drains the mailbox and invokes messages.
pub(super) struct DispatcherCore {
  mailbox:          ArcShared<Mailbox>,
  executor:         ArcShared<dyn DispatchExecutor>,
  invoker:          ActorRuntimeMutex<Option<ArcShared<dyn MessageInvoker>>>,
  state:            AtomicU8,
  throughput_limit: Option<NonZeroUsize>,
}

impl DispatcherCore {
  pub(super) fn new(
    mailbox: ArcShared<Mailbox>,
    executor: ArcShared<dyn DispatchExecutor>,
    throughput_limit: Option<NonZeroUsize>,
  ) -> Self {
    Self {
      mailbox,
      executor,
      invoker: ActorRuntimeMutex::new(None),
      state: AtomicU8::new(DispatcherState::Idle.as_u8()),
      throughput_limit,
    }
  }

  pub(super) const fn mailbox(&self) -> &ArcShared<Mailbox> {
    &self.mailbox
  }

  pub(super) fn register_invoker(&self, invoker: ArcShared<dyn MessageInvoker>) {
    *self.invoker.lock() = Some(invoker);
  }

  pub(super) fn executor(&self) -> &ArcShared<dyn DispatchExecutor> {
    &self.executor
  }

  pub(super) const fn state(&self) -> &AtomicU8 {
    &self.state
  }

  pub(super) fn drive(self_arc: ArcShared<Self>) {
    let dispatcher = self_arc;
    loop {
      {
        let this = &*dispatcher;
        this.process_batch();
      }

      let should_continue = {
        let this = &*dispatcher;
        DispatcherState::Idle.store(&this.state);
        this.has_pending_work()
          && DispatcherState::compare_exchange(DispatcherState::Idle, DispatcherState::Running, &this.state).is_ok()
      };

      if should_continue {
        continue;
      }

      break;
    }
  }

  fn process_batch(&self) {
    let limit = self.throughput_limit.map(NonZeroUsize::get).unwrap_or(DEFAULT_THROUGHPUT);
    let mut processed = 0_usize;

    while processed < limit {
      match self.mailbox.dequeue() {
        | Some(MailboxMessage::System(msg)) => {
          self.handle_system_message(msg);
          processed += 1;
        },
        | Some(MailboxMessage::User(msg)) => {
          self.handle_user_message(msg);
          processed += 1;
        },
        | None => break,
      }
    }
  }

  fn handle_system_message(&self, message: SystemMessage) {
    match message {
      | SystemMessage::Suspend => self.mailbox.suspend(),
      | SystemMessage::Resume => self.mailbox.resume(),
      | other => {
        let _ = self.invoke_system_message(other);
      },
    }
  }

  fn handle_user_message(&self, message: AnyMessage) {
    let _ = self.invoke_user_message(message);
  }

  fn invoke_user_message(&self, message: AnyMessage) -> Result<(), ActorError> {
    if let Some(invoker) = self.invoker.lock().as_ref() {
      return invoker.invoke_user_message(message);
    }
    Ok(())
  }

  fn invoke_system_message(&self, message: SystemMessage) -> Result<(), ActorError> {
    if let Some(invoker) = self.invoker.lock().as_ref() {
      return invoker.invoke_system_message(message);
    }
    Ok(())
  }

  pub(super) fn enqueue_user(self_arc: &ArcShared<Self>, message: AnyMessage) -> Result<(), SendError> {
    match self_arc.mailbox.enqueue_user(message) {
      | Ok(EnqueueOutcome::Enqueued) => {
        super::dispatcher_struct::Dispatcher::from_core(self_arc.clone()).schedule();
        Ok(())
      },
      | Ok(EnqueueOutcome::Pending(mut future)) => {
        Self::drain_offer_future(self_arc, &mut future)?;
        super::dispatcher_struct::Dispatcher::from_core(self_arc.clone()).schedule();
        Ok(())
      },
      | Err(error) => Err(error),
    }
  }

  pub(super) fn enqueue_system(self_arc: &ArcShared<Self>, message: SystemMessage) -> Result<(), SendError> {
    self_arc.mailbox.enqueue_system(message)?;
    super::dispatcher_struct::Dispatcher::from_core(self_arc.clone()).schedule();
    Ok(())
  }

  fn drain_offer_future(self_arc: &ArcShared<Self>, future: &mut MailboxOfferFuture) -> Result<(), SendError> {
    let waker = ScheduleWaker::into_waker(self_arc.clone());
    let mut cx = Context::from_waker(&waker);

    loop {
      match Pin::new(&mut *future).poll(&mut cx) {
        | Poll::Ready(Ok(_)) => return Ok(()),
        | Poll::Ready(Err(error)) => return Err(error),
        | Poll::Pending => {
          super::dispatcher_struct::Dispatcher::from_core(self_arc.clone()).schedule();
          block_hint();
        },
      }
    }
  }

  fn has_pending_work(&self) -> bool {
    self.mailbox.system_len() > 0 || (!self.mailbox.is_suspended() && self.mailbox.user_len() > 0)
  }
}
