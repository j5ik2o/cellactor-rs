//! Remoting 側へトポロジイベントを配信するポート定義。

use crate::std::provisioning::provider_event::RemoteTopologyEvent;

/// Remoting への送信インターフェイス。
pub trait RemotingPort: Send + Sync {
  /// RemoteTopologyEvent を配信する。
  fn publish_remote_topology(&self, event: &RemoteTopologyEvent);
}
