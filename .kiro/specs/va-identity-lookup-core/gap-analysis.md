# ギャップ分析（va-identity-lookup-core）

## 1. 現状把握
- **ActorSystem と拡張機構**: `SystemStateGeneric` がシステム名/authority 情報を `PathIdentity` に保持し、Remoting 設定もここで参照可能（`modules/actor/src/core/system/system_state.rs`）。`ExtendedActorSystem` は Extension 登録や system guardian 配下での `spawn_system_actor` を提供し、クラスタ系サービスを差し込む余地はあるが実装は存在しない（`modules/actor/src/core/system/extended_actor_system.rs`）。
- **Remoting 基盤**: `RemotingExtension` が transport supervisor を起動し、`RemotingControlHandle` を公開（`modules/remote/src/core/remoting_extension.rs`）。`RemoteActorRefProvider` は association/watch を `RemoteAuthorityManager` と協調して管理するが、ClusterIdentity や Rendezvous などの概念は未登場（`modules/remote/src/core/remote_actor_ref_provider.rs`）。
- **イベント/メトリクス**: EventStream には Lifecycle/RemoteAuthority/TickDriver 等が登録済みで、Cluster 特有のイベントは未定義（`modules/actor/src/core/event_stream/event_stream_event.rs`）。Tick/Scheduler メトリクスはあるが、VA/Pid 解決向けのカウンタは無し。
- **既存コードの欠落**: `rg` による検索でも Cluster/IdentityLookup/PidCache/Rendezvous 等のシンボルがヒットせず、クラスタ機能は全く未実装の状態。

## 2. 要件適合性とギャップ
| 要件領域 | 既存アセット | ギャップ |
| --- | --- | --- |
| 識別子オーナー決定/ロック | RemoteAuthorityManager が authority 状態を保持し deferred キューを提供（`modules/actor/src/core/system/remote_authority.rs`）。 | **Missing**: ClusterIdentity、MemberList、Rendezvous ハッシュ、Lock ストア。BlockList をクラスタレベルで共有する仕組みも不在。 |
| アクティベーション生成/終端 | System guardian 配下への spawn API と Props 注入は利用可能（`modules/actor/src/core/system/extended_actor_system.rs`）。 | **Missing**: PlacementActor、ActivationRequest/Response、ClusterIdentity を Props に埋め込む仕掛け、ActivationTerminated 広報経路。 |
| リクエストルーティング/リトライ | ask/RequestFuture、EventStream、ログスロットル等の基盤は揃っている。 | **Missing**: ClusterContext、PidCache、リトライポリシー、IdentityLookup API、BlockList に伴うキャッシュ無効化。 |
| ステート通知/観測 | EventStream 拡張余地・メトリクス基盤あり。 | **Missing**: Cluster専用イベント型、ClusterMetrics カウンタ、BlockList/Activation イベントの発火ポイント。 |

## 3. 実装アプローチ候補
- **Option A: RemotingExtension 増築**  
  - RemotingExtension 配下に VA サービスを追加し、既存の RemoteActorRefProvider/RemoteAuthorityManager を流用。  
  - *利点*: transport・Authority 状態と密に連携できる、ファイル追加が少ない。  
  - *欠点*: RemotingExtension の責務増大、クラスタ以外の Remoting 利用と干渉するリスク。
- **Option B: 新規 Cluster 拡張 crate**  
  - `fraktor-cluster-rs`（仮）を新設し、ExtensionInstaller で ActorSystem に組み込む。Rendezvous/PlacementActor 等は独立モジュール化。  
  - *利点*: 責務分離が明確、protoactor-go の構造を直接参照しやすい。  
  - *欠点*: 新規 crate/feature 配線や CI 設定が必要で初期コストが高い。
- **Option C: Hybrid（段階導入）**  
  - Phase1 で ActorSystem 拡張 + Remoting hook（IdentityLookup, PlacementActor, PidCache）を既存 crate 内に導入し、その後 Gossip/MemberList を別 crate へ抽出。  
  - *利点*: 段階的に導入でき既存構造への影響を抑えられる。  
  - *欠点*: 中間状態で責務境界が不明瞭になる恐れ。

## 4. 労力・リスク評価
- **Effort: L (1〜2 週間)** — IdentityLookup/PlacementActor/PidCache/Gossip/EventStream 拡張など多岐にわたる新規実装と統合テストが必要。
- **Risk: High** — 分散ロックとリトライ制御を誤ると既存 Remoting パスに悪影響が及ぶ。no_std 対応の Rendezvous・バックオフ実装も不確実性が高い。

## 5. Research Needed
1. **Rendezvous ハッシュ実装の選定**: no_std かつ FNV/Murmur 互換を保つ手法の調査。
2. **ロック/永続化ストア**: Phase1 はインメモリで進めるとしても、etcd など外部ストアを将来的に差し替えるパス設計が必要。
3. **EventStream 拡張方針**: Cluster 用イベント追加時のサブスクライバ互換性とバックプレッシャを要確認。
4. **OpenTelemetry 統合**: ClusterMetrics をどの crate で提供するか、既存メトリクスとの整合性を調査。

## 6. 推奨事項（設計フェーズ向け）
- Option C を軸に Phase1 で IdentityLookup/PlacementActor/PidCache を既存 crate に追加し、評価後に専用 crate 化を検討する案が現実的。
- 設計時には ExtensionInstaller/API での提供方法、EventStream/metrics の具体的な型名、Rendezvous 実装方針を明文化すること。
- Research 項目の検証結果を設計ドキュメントへ反映し、後続フェーズで等価テストケースを列挙する。
