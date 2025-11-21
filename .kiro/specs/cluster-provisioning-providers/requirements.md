# 要件ドキュメント

## 導入
cluster モジュールの Provisioning 開発フェーズ。references/protoactor-go 互換仕様として、クラスタプロバイダ（in-memory/Consul/K8s など）の登録・監視・設定を統合し、PlacementSupervisor / PartitionManager および Remoting と連携するプロビジョニング経路を確立する。

## 要件

### 要件1: プロバイダ登録と設定検証
**目的:** 運用者として、複数 ClusterProvider を安全に登録し、無効設定を起動前に遮断したい。

#### 受け入れ条件
1. When プロバイダ設定が送信されたとき、ClusterProvisioning Service は必須フィールドを検証し、欠落があれば設定を拒否しなければならない。
2. If プロバイダ名が既存と衝突した場合、ClusterProvisioning Service は命名衝突エラーを返し、既存プロバイダを維持しなければならない。
3. When プロバイダが監視機能などの必須能力不足を示したとき、ClusterProvisioning Service は当該プロバイダを Disabled とし、理由を記録しなければならない。
4. Where プロバイダが Consul または K8s 等の外部バックエンドを要求する場合、ClusterProvisioning Service は受け入れ前に接続性を検証しなければならない。
5. The ClusterProvisioning Service shall 永続ストアに受理済みプロバイダ定義を保存し、再起動後も同一セットを復元しなければならない。

### 要件2: トポロジウォッチとスナップショット供給
**目的:** PlacementSupervisor / PartitionManager が安定したトポロジ更新を継続的に受け取りたい。

#### 受け入れ条件
1. While プロバイダが Active の間、ClusterProvisioning Service はプロバイダ固有の Watch ハンドルでトポロジ更新を PlacementSupervisor と PartitionManager にストリーミングしなければならない。
2. When プロバイダのストリームが終了またはエラーになったとき、ClusterProvisioning Service は最新スナップショットのハッシュ付き終了シグナルを送出し、最後のスナップショットを利用可能に保持しなければならない。
3. When 新しいスナップショットのハッシュが前回と異なるとき、ClusterProvisioning Service は順序を保って配信し、古いキャッシュを無効化対象としてマークしなければならない。
4. If プロバイダがメンバーゼロを報告した場合、ClusterProvisioning Service はトポロジを空としてマークし、配置を一時停止するよう通知しなければならない。
5. Where 複数プロバイダが設定されている場合、ClusterProvisioning Service は最優先で健全なプロバイダを選択し、ストリーム終了時にヘルシーな代替へフェイルオーバしなければならない。

### 要件3: PlacementSupervisor / PartitionManager 連携
**目的:** プロビジョニング済みトポロジを用いた所有権計算とパーティション配置を一貫させたい。

#### 受け入れ条件
1. When トポロジスナップショットを受信したとき、ClusterProvisioning Service はメンバー情報とハッシュを PlacementSupervisor に渡し、配置要求処理前に所有権再計算を実行させなければならない。
2. When PartitionManager がパーティションマップの更新を要求したとき、ClusterProvisioning Service は最新スナップショットを返すか、利用不可理由を明示して失敗させなければならない。
3. If PlacementSupervisor が所有権変更を指示した場合、ClusterProvisioning Service は同一論理ティック内に更新リースメタデータを PartitionManager に公開しなければならない。
4. When フェイルオーバで代替プロバイダに切り替えるとき、ClusterProvisioning Service は provider-changed イベントを PlacementSupervisor と PartitionManager へ送出し、その後の配置処理に適用しなければならない。
5. While Graceful Shutdown が進行している間、ClusterProvisioning Service は新規スナップショット配信を拒否し、ドレイン用に最後のスナップショットを保持しなければならない。

### 要件4: Remoting 連携とノード観測
**目的:** Remoting が外部ノードの参加/離脱・隔離を検知し、リモートチャネル健全性を維持できるようにしたい。

#### 受け入れ条件
1. When プロバイダがノード参加または離脱を報告したとき、ClusterProvisioning Service は対応する RemoteTopology イベントを発行し、Remoting がチャネルを調整できるようにしなければならない。
2. If Remoting がノードを隔離した場合、ClusterProvisioning Service はアクティブなスナップショット上で当該ノードを blocked としてマークし、消費者へ通知しなければならない。
3. While プロバイダが接続エラーで Degraded 状態の間、ClusterProvisioning Service は Remoting に警告状態を提示し、陳腐データに基づく新規リモートチャネル追加を避けなければならない。
4. When Remoting が隔離ノードとの接続を回復したとき、ClusterProvisioning Service は次回の正常リフレッシュ後にノードを再活性化させなければならない。
5. The ClusterProvisioning Service shall プロバイダ/リモートの健全性メトリクス（up/down/degraded と最終更新時刻）を監視向けに公開しなければならない。

### 要件5: 観測・エラーハンドリング・フェイルセーフ
**目的:** SRE として、プロビジョニング経路の健全性を計測し、異常時に安全側へ倒すメカニズムを確保したい。

#### 受け入れ条件
1. The ClusterProvisioning Service shall スナップショット遅延、プロバイダフェイルオーバ回数、ストリーム中断回数のメトリクスを記録しなければならない。
2. When 検証失敗がしきい値を超えて繰り返されたとき、ClusterProvisioning Service は以後のプロバイダ有効化を抑止し、警告イベントを発行しなければならない。
3. If すべてのプロバイダが利用不能になった場合、ClusterProvisioning Service は provisioning-unavailable 状態を全コンシューマへ示し、プロバイダ回復まで新規配置を停止しなければならない。
4. When 設定変更を適用するとき、ClusterProvisioning Service は原子的にリロードし、新セットまたは旧セットのどちらか一方のみが有効になるようにしなければならない。
5. The ClusterProvisioning Service shall 検証・接続・ストリーム失敗向けの構造化エラーコードを提供し、テストで決定的に扱えるようにしなければならない。
