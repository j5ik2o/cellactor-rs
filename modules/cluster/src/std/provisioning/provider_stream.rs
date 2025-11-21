//! Provider stream abstraction.

use crate::std::provisioning::provider_event::ProviderEvent;

/// Provider からトポロジイベントを取得するストリーム抽象。
pub trait ProviderStream: Send {
  /// 次のイベントを取得する。イベントが無ければ `None`。
  fn next_event(&mut self) -> Option<ProviderEvent>;
}
