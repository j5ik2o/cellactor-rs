# 要件ドキュメント

## 導入
VA・IdentityLookup 周り（PartitionManager/PlacementActor/Request経路）実装の第一段階として、クラスタ内の仮想アクター識別子を安定的に解決し、アクティベーションとルーティングを制御するための振る舞いを定義する。

## 要件

### 要件1: 識別子オーナー決定とロック管理
**目的:** クラスタ運用者として 仮想アクターの所有ノードを常に一意に決定し、競合なくロックできるようにし、再配置や再起動時の衝突を防ぎたい。

#### 受け入れ条件
1. When クラスタプロバイダから新しいトポロジースナップショットを受信したとき、VA Identityサービスは ClusterIdentity ごとのオーナー候補を Rendezvous ハッシュで再計算しなければならない。
2. When 識別子のアクティベーションロック要求を受け取ったとき、VA Identityサービスは 1 つの所有ノードに対してのみロックを発行し、他ノードには待機指示を返さなければならない。
3. While アクティベーションロックが保持されている間、VA Identityサービスは同一 ClusterIdentity への追加ロック要求を拒否し続けなければならない。
4. If 所有ノードが BlockList に追加された場合、VA Identityサービスは そのノードが保持していたロックと所有情報を直ちに無効化しなければならない。
5. The VA Identityサービス shall 計算済みのオーナー情報を PartitionManager へ公開し、PlacementActor が即時に参照できるようにしなければならない。

### 要件2: アクティベーション生成と終端処理
**目的:** クラスタ運用者として 仮想アクターの生成・停止を制御面から一元把握し、所有ノードの変更時も整合したクリーンアップを行いたい。

#### 受け入れ条件
1. When PlacementActor が ActivationRequest を受信したとき、PlacementActor は 要求された Kind が登録済みであれば Props に ClusterIdentity を注入して新規アクターを生成しなければならない。
2. If PlacementActor が 未登録の Kind を受け取った場合、PlacementActor は ActivationResponse.failed = true を返して要求元へ失敗を通知しなければならない。
3. When アクティベート済みアクターが Terminated 通知を発行したとき、PlacementActor は 該当 ClusterIdentity の登録を削除し、VA Identityサービスへ ActivationTerminated をブロードキャストしなければならない。
4. When 新しいトポロジーハッシュで所有ノードが変更されたとき、PlacementActor は 自ノードが所有者でなくなった ClusterIdentity のアクターを Poison し、ロックを解放しなければならない。
5. While Graceful Shutdown を実施している間、PlacementActor は 未処理の ActivationRequest を拒否し、既存アクターの Poison 終了を待機し続けなければならない。

### 要件3: リクエストルーティングとリトライ
**目的:** エンドユーザアプリとして 仮想アクターへの Request/RequestFuture を透過的に成功させ、ネットワーク変動時も自動リトライで安定させたい。

#### 受け入れ条件
1. When クライアントが ClusterIdentity への Request を行ったとき、ClusterContext は PidCache を参照し、PID が存在しない場合にのみ IdentityLookup へ問い合わせなければならない。
2. When ClusterContext が ErrTimeout/DeadLetter を受け取ったとき、ClusterContext は 構成された最大リトライ回数まで指数待機を挟みつつ再送しなければならない。
3. While RequestFuture が実行中の間、ClusterContext は コンテキストタイムアウトを監視し、期限切れ時に Pending Future をキャンセルしなければならない。
4. If IdentityLookup から PID 解決に失敗した場合、ClusterContext は 呼び出し元へエラーを返し、メトリクスへ失敗イベントを記録しなければならない。
5. When PidCache に紐づくノードがクラスタから離脱したとき、ClusterContext は 当該エントリを無効化し、次回リクエストで新規 PID を取得しなければならない。

### 要件4: ステート通知と観測性
**目的:** 観測担当として アクティベーションや所有者変更のイベントを系統的に把握し、テレメトリへ反映したい。

#### 受け入れ条件
1. When PlacementActor が ActivationResponse を成功させたとき、VA Identityサービスは EventStream へ ActivationStarted イベントを発行しなければならない。
2. When PlacementActor が ActivationTerminated をブロードキャストしたとき、MemberList は クラスタ全体の EventStream へ同イベントを転送しなければならない。
3. The ClusterMetricsサービス shall 仮想アクター数・PID 解決時間・リトライ回数のメトリクスを記録しなければならない。
4. When Gossip から BlockList 更新を受け取ったとき、VA Identityサービスは BlockList 適用件数をイベントとしてロギングしなければならない。
5. While 要求ログのスロットル条件を満たしている間、ClusterContext は リトライ失敗の警告をロガーへ出力し続けなければならない。
