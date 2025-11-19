# Proto.Actor Go クラスター中核機能メモ

## 背景
- 目的: Proto.Actor Go 実装（`references/protoactor-go/cluster`）を分析し、Rust で互換クラスター機能を実装する際の必須要素を抽出する。
- 仮想アクターモデルを核に、メンバーシップ検知・ID ルックアップ・ルーティング・PubSub・観測性までを一体化した構造になっている。

## 中核機能一覧

### 1. クラスター基盤 (`cluster.go`, `config.go`, `kind.go`)
- `Cluster` は ActorSystem 拡張として Gossip/PubSub/PidCache/MemberList/IdentityLookup を初期化し、`StartMember`/`StartClient` で Remote→Kind 登録→IdentityLookup→Gossip→ClusterProvider の順に起動する。
- `Config` はリモート設定・タイムアウト・MemberStrategyBuilder・PubSubConfig などを束ね、Kind 登録時にクラスタ用ミドルウェアで `ClusterInit` を注入、Grain 側が `ClusterIdentity` と `Cluster` 参照を得られるようにしている。
- `ActivatedKind` は Kind ごとの Props とメンバーストラテジーを保持し、Spawn/Terminate 時にカウンタを更新、OpenTelemetry のメトリクス測定にも利用される。

### 2. メンバーシップとトポロジ制御 (`member_list.go`, `member_strategy.go`, `rendezvous.go`, `round_robin.go`, `members.go`)
- `MemberList` は ClusterProvider から得たメンバー集合を BlockList 差し引きでクリーンアップし、Join/Leave 差分をクラスタへ通知、Remote BlockList 更新や EndpointTerminatedEvent の発火も担う。
- Kind 別の `MemberStrategy` が全メンバー集合を保持し、`Rendezvous` ハッシュでパーティション先を決め、`SimpleRoundRobin` でアクティベーション要求の送り先を分散させる。
- `MemberSet`/`Members` がハッシュ値・差集合・Union を提供し、トポロジ変化の検出と `TopologyHash` 計算に利用される。

### 3. Gossip とコンセンサス (`gossiper.go`, `gossip_actor.go`, `informer.go`, `gossip_state_management.go`, `consensus*.go`)
- `Gossiper` と `GossipActor` が `GossipState` をキーごとにマージし、`MemberStateDelta` を別ノードへ Push。`ClusterTopology` や `GracefullyLeft` のようなキー更新がイベントストリームに流れる。
- `Informer` がローカルシーケンス番号と送信水位線を追跡し、ファンアウト数と MaxSend 数を制御。`setKey` や `mergeState` でシーケンス優先ルールを統一している。
- `ConsensusCheckBuilder`/`ConsensusChecks` が Gossip キー群の一致判定と結果キャッシュを提供し、トポロジの合意 (TopologyConsensus) や他のキーの合意判定ができる。

### 4. Identity Lookup と仮想アクター管理 (`identity_lookup.go`, `identitylookup/disthash/*`, `default_context.go`, `pid_cache.go`)
- `IdentityLookup` インタフェースが Get/RemovePid/Setup/Shutdown を定義。Go 実装では `disthash` が PartitionManager+placementActor 構成になっており、Rendezvous ハッシュで所有アドレスを決めて `ActivationRequest` RPC で PID を取得する。
- `placementActor` は Kind ごとの Props からアクターを Spawn し、`ActivationTerminated` イベントを `MemberList.BroadcastEvent` で広報、トポロジ変更時には Rendezvous 再計算で所有権が変わった Grain を Poison する。
- `DefaultContext` は `PidCache`→`IdentityLookup` の順で PID を解決し、リトライ/バックオフ/タイムアウト/メトリクス記録を行った上で Request/Future を実行する。PidCache は Identity+Kind キーで PID を保持し、Member 離脱時や Terminate 時に掃除する。

### 5. Grain RPC とメッセージ定義 (`grain.go`, `grain.proto`, `cluster.proto`, `errors.go`)
- `grain.proto` は `GrainRequest` (method index + Any シリアライズ) と `GrainResponse` を定義し、仮想アクター RPC の汎用フォーマットとして利用する。
- `cluster.proto` は `ClusterTopology`, `ActivationRequest/Response`, `IdentityHandover`, `MemberHeartbeat` など、クラスタ制御用の Proto メッセージを網羅している。
- `errors.go` では gRPC に似た Reason をもつ `GrainErrorResponse` を提供し、call サイドが `Reason(err)` で分類できる。

### 6. PubSub と配信パイプライン (`pubsub*.go`)
- `PubSub` 拡張が `$pubsub-delivery` アクターを起動し、`PubSubMemberDeliveryActor` が `DeliverBatchRequest` を受けて PID または ClusterIdentity へ `RequestFuture` でバッチ配信、失敗時は `NotifyAboutFailingSubscribersRequest` を TopicActor に返す。
- `TopicActor` はサブスク状態をメモリ＋`KeyValueStore` に保持し、ClusterTopology イベントで離脱ノードのサブスクを自動撤退させる。Publish 時は同一アドレスごとにバッチをまとめ、DeliveryActor 経由で転送する。
- `Publisher` インタフェース＋`BatchingProducer` が TopicActor RPC をラップし、キュー/バッチ/再試行/タイムアウト制御を提供。これにより大量 Publish を効率化する。

### 7. 観測性と補助モジュール (`metrics/cluster_metrics.go`, `member_status_events.go`, `pubsub_extensions.go`)
- OpenTelemetry ベースの `ClusterMetrics` が Spawn 時間・Request 時間・Retry 回数・PID 解決時間・仮想アクター数・メンバー数をメトリクスとして公開する。
- MemberStatus イベント型や PubSub 拡張ポイント (`PubSubConfig`, `PublisherIdleTimeout`) など、周辺モジュールも標準化されている。

## Rust 実装へ活用する際のヒント
- 仮想アクターの恒常性を守るには「IdentityLookup (所有ノード決定) → ActivationRequest → PlacementActor → ClusterInit」というシーケンスを再現することが重要。
- メンバーシップと Gossip を二層構造に分け、ClusterProvider でソース・オブ・トゥルースを取得しつつ Gossip で BlockList や補助情報を伝播させる点を踏襲する。
- リトライポリシーと PID キャッシュはクラスター体験の滑らかさに直結するため、PidCache の掃除タイミングやメトリクス計測ポイントを Rust 版でも意識する。
- PubSub/Broadcast も Grain 呼び出しで実現しているため、TopicActor のような「状態をクラスタ内で冪等に再構築できる仮想アクター」を汎用的に作れる仕組みがあると互換性を保ちやすい。
- Proto メッセージ群（cluster.proto / gossip.proto / grain.proto）を Rust 版にも取り込み、互換線を揃えることで将来的な異言語連携を見据えられる。

