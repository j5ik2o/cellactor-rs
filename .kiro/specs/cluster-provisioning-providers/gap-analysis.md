# cluster-provisioning-providers ギャップ分析

## スコープ概要
- 目的: ClusterProvisioning Service を追加し、複数 ClusterProvider（in-memory/Consul/K8s など）の登録・監視・フェイルオーバを提供、PlacementSupervisor/PartitionManager/Remoting へ安定したトポロジを供給する。
- 状況: 要件生成済（未承認）。既存 cluster モジュールは単一 TopologyWatch を ClusterConfig に保持するのみで、複数プロバイダ管理・フェイルオーバ・Remoting 連携は未実装。

## 現状資産の調査
- 構成・パターン: 2018 モジュール、1ファイル1型、core/std 分離。cluster/core/config に `TopologyStream` と `TopologyWatch`（単一ストリーム）あり。ハッシュリング、ActivationLedger、ClusterRuntime、EventStream/metrics 実装済み。
- 依存方向: utils/core → actor/core → cluster/core → cluster/std。cluster/std には metrics と bootstrap しかなく、プロバイダ/プロビジョニング関連コードなし。
- テスト配置: `<module>/tests.rs` パターン。topology_watch, cluster_config に単体テストあり。プロバイダ関連のテストなし。

## 要件→資産マップとギャップ
- 要件1 (プロバイダ登録/検証): **Missing** — 複数プロバイダの登録・検証・永続化機構なし。TopologyWatch は 1つのみ。
- 要件2 (トポロジウォッチ/スナップショット供給): **Partial→Missing** — TopologyStream trait はあるが、複数ストリーム管理・ハッシュ差分配信・終了シグナル保持・フェイルオーバは未実装。
- 要件3 (PlacementSupervisor/PartitionManager 連携): **Missing** — 該当コンポーネント自体がコードベースに存在しない。ClusterRuntime へのスナップショット連携は TopologyWatch 1本のみ。
- 要件4 (Remoting 連携・ノード観測): **Missing** — Remoting 連携イベントや RemoteTopology イベントなし。隔離ノードをプロバイダスナップショットに反映する経路なし。
- 要件5 (観測・フェイルセーフ): **Partial/Missing** — ClusterMetrics に限られた項目のみ。プロバイダ/ストリーム系メトリクス、エラーコード体系、フェイルオーバカウントは未実装。

## アプローチ案
### Option A: 既存 ClusterConfig/TopologyWatch を拡張
- ClusterConfig に複数プロバイダリスト＋優先度を追加し、ClusterRuntime に複数 Watch を扱うレイヤを追加。
- 長所: 既存ビルダー/設定を再利用。短期で着手可能。
- 短所: ClusterConfig・Runtime の責務膨張、1ファイル1型制約下で肥大化リスク。

### Option B: 新規 provisioning サブモジュールを追加（推奨）
- `modules/cluster/src/std/provisioning/` に ClusterProvisioningService（std）、ProviderRegistry、ProviderWatchHandle、FailoverPolicy を新設。core には必要最小限の trait (ProviderSnapshot, ProviderStream) を追加し std に実装を隔離。
- 長所: 責務分離、no_std 汚染を避けつつ std で外部システム(Consul/K8s)対応可能。テスト境界明確。
- 短所: 新規ファイル増、ClusterRuntime とのブリッジ設計が必要。

### Option C: ハイブリッド
- core に Provider 抽象と Snapshot マージロジックを置き、std 側で実プロバイダ実装とフェイルオーバ制御を行う。ClusterConfig には minimal hook のみ追加。
- 長所: 両層の責務バランスがとれる。
- 短所: 抽象/具象の分割線を誤ると二重実装リスク。

## リスクと工数感
- Effort: L（1–2 週間）— 新規モジュール追加、複数プロバイダ・フェイルオーバ・観測・イベント統合が必要。
- Risk: Medium/High — 外部バックエンド接続（Consul/K8s）とフェイルオーバロジック、Remoting 連携設計の不確実性。

## Research Needed
- Consul/K8s での topology watch 実装手法（protoactor-go 互換）
- Remoting 既存設計（リポジトリ内 remote クレート）とのイベント整合性
- フェイルオーバ優先度・ヘルス評価アルゴリズム（タイムアウト閾値・バックオフ）

## 推奨（設計フェーズへの入力）
- Option B をベースに、core へ最小限の Provider 抽象（Snapshot, Stream, Health）を追加し、std provisioning サービスで登録/検証/フェイルオーバ/観測を実装。
- Remoting / Placement 連携はイベント種とメトリクス項目を先に閉じ、設計で API/イベント契約を明文化。
