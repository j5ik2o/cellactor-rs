# cluster-extension-bootstrap ギャップ分析

## スコープ概要
- 目的: ActorSystem へ ClusterExtension を無停止で組み込み、Topology 監視・Placement・Routing・観測を統合するブートストラップ経路を確立する。
- 状況: 要件は生成済・未承認。cluster コア機能は進行中だが Installer/Bootstrap 層が未実装。

## 現状整理
- コア実装: IdentityLookup/ActivationLedger/PlacementActor/PidCache/RetryPolicyRunner などは存在。EventStream と ClusterEvent 定義あり。StdClusterMetrics 追加済。
- 欠落: ClusterExtension/ExtensionInstaller、ActorSystemConfig への配線、ClusterRuntime ハンドル公開、ブートストラップ状態管理。
- 部分実装: Ledger に Releasing/Released 追加、BlockList hook あり。Graceful Shutdown は部分的。観測はログ変換の EventStreamAdapter のみ。
- Lint/属性: 各 crate lib.rs の clippy 属性は統一されたが cluster 側 clippy 指摘は多数未処理（const化、Errors/Panics doc、expect 避けなど）。

## 要件→資産マップ（ギャップ・Research Needed）
- 要件1 Installer/ブートストラップ: **Missing** - Installer, ClusterExtension, ハンドル公開なし。
- 要件2 Topology監視/BlockList: **Partial** - IdentityLookup/ActivationLedger/BlockList hook あり。TopologyStream 終了時の保持・警告、ハッシュ未変化時の再利用ロジックが未実装。
- 要件3 Placement/Shutdown: **Partial** - Activation/Placement 基礎あり。ActivationFailed/Terminated/Graceful 完了通知を ClusterEvent/metrics 経由で返す経路と Installer 完了シグナルが未整合。
- 要件4 Routing/観測/metrics: **Partial** - ClusterContext/PidCache/Retry あり。成功時の latency 記録、ClusterEvent publish、Otel メトリクス送出が不足。

## アプローチ案
- オプションA: 既存 cluster モジュールを拡張し、Installer/Bootstrap を既存ファイル内に追加。小さな変更だがファイル肥大化リスク。
- オプションB: `modules/cluster/src/std/bootstrap/` など新設し、Installer/Bootstrap Actor/metrics 配線を分離。責務明確・テストしやすいが新規ファイル増。
- オプションC: ハイブリッド（Installer/Bootstrap を新設し、Runtime/Context 配線のみ拡張）。段階的移行向き。**推奨**。

## 労力・リスク
- Effort: M（3–7日）— Installer設計、観測/metrics 配線、テスト整備。
- Risk: Medium — no_std/std 並行・厳格 clippy・EventStream/Otel 統合で影響範囲中程度。

## 次ステップ（設計フェーズへの入力）
1. オプションCベースで設計を起こす（Installer, Bootstrap actor, 状態管理、観測/metrics 配線）。
2. clippy 対応方針を設計内で定義（const/must_use/doc、expect排除）。
3. `/prompts:kiro-spec-design cluster-extension-bootstrap` を実行してタスク分解へ進む。
