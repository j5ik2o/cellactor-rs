# 実装計画

- [x] 1. ClusterExtension 基盤とハッシュリングを実装
  - _対応要件: 1.1, 1.2, 1.3, 1.4, 1.5_
  - _依存タスク: -_
  - _完了条件: 1.x 系子タスク完了かつ nightly ツールチェーンで PoC が green_

- [x] 1.1 ClusterConfig と ClusterRuntime の骨格を作成
  - ClusterConfig で rendezvous 設定・retry・topology 受信チャネルを扱えるようにする
  - ClusterRuntime に IdentityLookupService / ActivationLedger / ClusterMetrics への参照を保持させ、ExtensionInstaller から初期化
  - _対応要件: 1.1, 1.5_
  - _依存タスク: -_
  - _完了条件: コンフィグ生成と Runtime 起動が単体テストで確認できること_

- [x] 1.2 HashRingProvider と ActivationLedger を実装
  - hash_rings + rapidhash を包む HashRingProvider を作り、トップロジー更新に伴う再構築を実装
  - ActivationLedger で lease 取得／解放／ Revoked 状態を管理し、ClusterRuntime.resolve に接続
  - _対応要件: 1.1, 1.2, 1.3, 1.5_
  - _依存タスク: 1.1_
  - _完了条件: hash_rings PoC が nightly でビルド成功し、lease の状態遷移テストがグリーン_

- [x] 1.3 BlockList 連動と PidCache 無効化経路を組み込む
  - ClusterBlocklistHook を実装して RemoteAuthorityManager のイベントから lease を即時 Revoked に更新
  - BlockListApplied イベント発行と PidCache invalidation をシーケンス通りに接続し、二重経路（hook＋EventStream）を保証
  - _対応要件: 1.4, 4.1_
  - _依存タスク: 1.2_
  - _完了条件: BlockList シナリオの統合テストで古い PID が拒否されること_

- [x] 2. Activation と Placement の制御経路を実装
  - _対応要件: 2.1, 2.2, 2.3, 2.4, 2.5_
  - _依存タスク: 1.1-1.3_
  - _完了条件: 2.x 子タスク完了かつ Placement 経路の統合テストが green_

- [x] 2.1 PartitionBridge と Activation メッセージ層を構築
  - ActivationRequest/ActivationResponse 型と PartitionBridge trait を実装し、既存 PartitionManager から呼び出せるようにする
  - ClusterRuntime からの resolve 結果を PartitionBridge 経由で PlacementActor へ転送する
  - _対応要件: 2.1_
  - _依存タスク: 1.x_
  - _完了条件: PartitionBridge 経由のモックテストで要求が往復すること_

- [x] 2.2 PlacementActor のアクティベーション/ Terminated 処理を実装
  - Props への ClusterIdentity 注入、ActivationResponse 成功/失敗時の Ledger 更新、BlockList 通知を実装
  - Terminated 受信で lease を Released に切り替え、ClusterEvent::ActivationTerminated を publish
  - _対応要件: 2.1, 2.2, 2.3_
  - _依存タスク: 2.1_
  - _完了条件: Placement 経路の単体テストと EventStream の検証が green_

- [x] 2.3 Graceful Shutdown と lease リリースフローを実装
  - Runtime に shutdown API を追加し、LeaseStatus::Releasing/Released/TimedOut の遷移と Retry 拒否 (`ClusterError::ShuttingDown`) を実装
  - Shutdown シーケンスを統合テストし、未解放 lease が一定時間でタイムアウトすることを確認
  - _対応要件: 2.4, 2.5_
  - _依存タスク: 2.2_
  - _完了条件: Graceful Shutdown テストが green で、ログに release 完了が出力されること_

- [x] 3. ClusterContext とリトライ/Routing を実装
  - _対応要件: 3.1, 3.2, 3.3, 3.4, 3.5_
  - _依存タスク: 1.x, 2.x_
  - _完了条件: 3.x 子タスク完了かつ Request/RequestFuture の統合テストが green_

- [x] 3.1 PidCache と IdentityLookup 統合
- [x] 3.2 リトライポリシーと RequestFuture 実装
  - std では dashmap + ArcSwap、no_std では shard 付き ToolboxMutex map を実装し、ClusterRuntime resolve と連携
  - Cache miss/hit/invalidated のテストを用意し、BlockList 連動が反映されることを確認
  - _対応要件: 3.1, 3.5_
  - _依存タスク: 1.2, 1.3_
  - _完了条件: PidCache の単体テストが green で 1M identity ベンチが基準値内_

- [x] 3.2 リトライポリシーと RequestFuture 実装
  - RetryPolicy に指数バックオフ＋ jitter を実装し、Timeout/DeadLetter 時に PidCache invalidation を行う
  - RequestFuture が Context timeout を監視し、エラー時に ClusterMetrics へ記録する
  - _対応要件: 3.2, 3.3_
  - _依存タスク: 3.1_
  - _完了条件: Retry シナリオの統合テストとメトリクス発火が確認できること_

- [x] 3.3 Routing 統合テストを追加
  - ClusterContext 経由で複数 kind/identity へ Request/RequestFuture を投げ、成功/リトライ/BlockList など代表ケースを検証
  - ClusterError (Timeout/Blocked/ShuttingDown) が適切に伝播することを assertion
  - _対応要件: 3.1-3.5_
  - _依存タスク: 3.2_
  - _完了条件: 統合テスト suite が green で、CI 上で実行されること_

- [x] 4. 観測性・メトリクスと最終統合
  - _対応要件: 4.1, 4.2, 4.3, 4.4, 4.5_
  - _依存タスク: 1-3_
  - _完了条件: 4.x 子タスク完了＋CI で EventStream/metrics テストが green_

- [x] 4.1 ClusterEvent と EventStream 連携を実装
  - ActivationStarted/ActivationTerminated/BlockListApplied/RetryThrottled イベントを EventStreamEvent に追加し、各シナリオで publish
  - Std/NoStd サブスクライバがイベントを受信できるようアダプタを更新
  - _対応要件: 4.1, 4.2_
  - _依存タスク: 1-3_
  - _完了条件: EventStream の単体テストと subscriber mocks が green_

- [x] 4.2 ClusterMetrics を計測
  - resolve/request duration、retry count、virtual actor gauge、BlockList 件数などを OpenTelemetry で記録
  - Graceful Shutdown／BlockList シナリオでメトリクスが期待値になることを検証
  - _対応要件: 4.3, 4.4_
  - _依存タスク: 4.1_
  - _完了条件: メトリクス E2E テストが green でメトリクス名が docs と一致

- [x] 4.3 システム統合テストを実施
  - in-memory ClusterProvider で Joined/Left/Blocked をシミュレートし、Activation → Request → Shutdown までを通しで検証
  - Bazel/CI workflow に統合テストジョブを追加し、hash_rings PoC + no_std ベンチも自動実行
  - _対応要件: 全要件 (1.1-4.5)_
  - _依存タスク: 4.2_
  - _完了条件: 統合テストが CI で green、ベンチ結果がログに保存されること_
