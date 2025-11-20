# 要件ドキュメント

## 導入
cluster-extension-bootstrap では、ActorSystem へ ClusterExtension を無停止で組み込み、Topology 監視・Placement・Routing・観測をまとめて稼働させる初期ブートストラップ経路を確立する。

## 要件

### 要件1: ClusterExtension インストールとハンドル公開
**目的:** ランタイム運用者として ClusterExtension を安全に注入し、ClusterRuntime/Context へ一貫してアクセスしたい。

#### 受け入れ条件
1. ExtensionInstaller が cluster-extension-bootstrap feature を ActorSystemConfig に含めるとき、ClusterExtension ブートストラップサービスは ActorSystem::start 完了前に ClusterRuntime ハンドルを登録しなければならない。
2. ブートストラップ中に設定検証エラーが発生したとき、ClusterExtension ブートストラップサービスは 説明的な起動エラーを発行し、拡張の有効化をブロックしなければならない。
3. ClusterRuntime ハンドルが生存している間、ClusterExtension ブートストラップサービスは Installer API を通じて ClusterContext・PartitionBridge・メトリクスハンドルを公開し続けなければならない。
4. ブートストラップ検知で feature が無効と判断されたとき、ClusterExtension ブートストラップサービスは Runtime 生成をスキップし、ActorSystem を変更してはならない。
5. ClusterExtension ブートストラップサービスは 拡張の導入可否を照会できるよう、クラスタ準備ステータスを永続化しなければならない。

### 要件2: トポロジ監視とオーナー再計算
**目的:** クラスタ制御面として TopologyProvider の変化を捕捉し、所有ノードとリースを常に最新状態へ保ちたい。

#### 受け入れ条件
1. TopologyStream が新しいスナップショットを配信したとき、IdentityLookup サービスは ハッシュリングを再構築し、新しいリースを発行する前に ownership メタデータを更新しなければならない。
2. 最新スナップショットのハッシュが変化しない間、IdentityLookup サービスは 同一 ClusterIdentity に対して既存の ownership 決定を再利用しなければならない。
3. ClusterBlocklistHook がノードのブロック通知を受け取ったとき、ClusterRuntime は 当該ノードが所有するリースをすべて失効させ、BlockListApplied の ClusterEvent を発行しなければならない。
4. TopologyStream が終了または停止したとき、ClusterRuntime は 直近のスナップショットを保持しつつランタイム警告を記録し、更新を受信するまで待機しなければならない。

### 要件3: Placement 経路と Shutdown 管理
**目的:** オーケストレータとして ActivationRequest/Response と Terminated を一貫した経路で処理し、Graceful Shutdown でも整合性を保ちたい。

#### 受け入れ条件
1. ClusterRuntime が ClusterIdentity の所有ノードを解決したとき、PartitionBridge は リースと Props を含む ActivationRequest を PlacementActor へ投入しなければならない。
2. PlacementActor が失敗を示す ActivationResponse を返したとき、ClusterRuntime は 対応するリースを解放し、ActivationFailed の ClusterEvent を発行しなければならない。
3. Terminated 通知が追跡中のリースへ届いたとき、ClusterRuntime は リースを削除し、キャッシュ済み PID を無効化し、ActivationTerminated の ClusterEvent を発行しなければならない。
4. Graceful Shutdown 実行中は、ClusterRuntime は 新規の resolve/activation 要求を ShuttingDown ClusterError で拒否し、Ledger が空になるまで保留中リースの解放を継続しなければならない。
5. Graceful Shutdown が完了したとき、ClusterRuntime は 仮想アクター数ゲージをゼロへ更新し、ブートストラップ API へ完了を通知しなければならない。

### 要件4: Routing・リトライ・観測メトリクス
**目的:** クラスタ利用者として ClusterContext を介した Request/Retry と EventStream/metrics を通じて観測可能なルーティング体験を得たい。

#### 受け入れ条件
1. クライアントが ClusterContext 経由でリクエストを送信したとき、ClusterContext は まず PidCache を参照し、キャッシュヒットしない場合にのみ ResolveBridge を呼び出さなければならない。
2. リクエストが RetryPolicy で許可された試行回数を超えたとき、ClusterContext は Timeout ClusterError を返し、当該 ClusterIdentity のリトライ回数メトリクスを記録しなければならない。
3. 仮想アクターがアクティブな間、ClusterMetrics 実装は 有効リース数を反映した actor-count ゲージを公開し続けなければならない。
4. ActivationStarted または BlockListApplied イベントが発生したとき、ClusterEventPublisher は 対応する ClusterEvent を EventStream サブスクライバへ配送しなければならない。
5. リクエストが成功裏に完了したとき、ClusterMetrics 実装は 当該 ClusterIdentity の resolve/request 所要時間を記録しなければならない。
