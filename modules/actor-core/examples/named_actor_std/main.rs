#![cfg(feature = "std")]

mod guardian;
mod lifecycle_printer;
mod printer;
mod start_message;

use std::{thread, time::Duration};

use cellactor_actor_core_rs::{ActorSystem, AnyMessage, EventStreamSubscriber, Props};
use cellactor_utils_core_rs::sync::ArcShared;
use guardian::GuardianActor;
use lifecycle_printer::LifecyclePrinter;
use start_message::Start;

fn main() {
  let system_props = Props::from_fn(|| GuardianActor).with_name("named-guardian");
  let system = ActorSystem::new(&system_props).expect("ユーザーガーディアンの起動に失敗しました");

  let lifecycle_subscriber: ArcShared<dyn EventStreamSubscriber> = ArcShared::new(LifecyclePrinter::default());
  let _subscription = system.subscribe_event_stream(&lifecycle_subscriber);

  let termination = system.when_terminated();

  system.user_guardian_ref().tell(AnyMessage::new(Start)).expect("Start メッセージの送信に失敗しました");

  thread::sleep(Duration::from_millis(50));

  system.terminate().expect("システムの停止に失敗しました");

  while !termination.is_ready() {
    thread::sleep(Duration::from_millis(10));
  }
}
