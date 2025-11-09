# serializer-nested-field: シリアライズ/デシリアライズ設計の破綻

## 問題概要
- `NestedSerializerOrchestrator` は AGGR ヘッダ付きの複合フォーマットを生成する一方、`Serialization::deserialize::<T>` および `bind_type::<T>(…, decoder)` が期待するのは「型 `T` を単独でバイナリ化したバイト列」であり、両者のフォーマットが一致しない。
- そのため `bind_type::<PurchaseOrder>` で登録したデコーダ（bincode ベース）は `serialization.serialize(&purchase_order)` の出力を復元できず、サンプルでは `deserialize_payload` と手書きの AGGR 解析で迂回している。
- 結果として `Serialization::serialize` と `Serialization::deserialize` が対称性を失い、Pekko 互換 API という仕様を満たさない。`bind_type` や `SerializerImpl::deserialize` を実装しても呼び出されず、LSP/DRY を守れない。

## 影響範囲
- `modules/actor-core/src/serialization/*` 全般（特に registry / type_binding / nested_serializer_orchestrator / serialization extension）。
- `examples/serialization_nested_demo_no_std` など、AGGR 出力をラウンドトリップしたい全サンプル・テストコード。
- `TelemetryService` や監査系 API は AGGR 前提で実装されているため、設計修正時に動作確認が必須。

## 望ましい解決策の方向性
1. **共通フォーマットの確立**  
   - `SerializedPayload` が保持する `manifest` と `serializer_id` に加え、AGGR 用のメタデータを `FieldEnvelopeBuilder` と対になる `FieldEnvelopeParser`（公開 API）で抽象化し、`bind_type` 登録時のデコーダから利用できるようにする。
2. **Registry/API の再設計**  
   - Aggregate 型を登録する際に「子フィールドの AGGR フォーマットを復元するデコーダ」を自動生成する、もしくは `deserialize::<T>` 側で `AggregateSchema` を参照しながら復元する仕組みを導入し、利用者が手動で AGGR を解析しなくて済むようにする。
3. **サンプルとテストの更新**  
   - `serialization.deserialize::<PurchaseOrder>` がそのまま使えるようになったら、現行サンプルの手動解析コードは削除、テストも API 対称性を検証する形に書き換える。

## 次のアクション候補
- 設計文書 (`.kiro/specs/serializer-nested-field/design.md`) をアップデートし、Aggregate デシリアライズ手順を明文化。
- `field_envelope_builder.rs` に対応する `field_envelope_parser.rs` を追加し、AGGR フォーマットを一元的に扱えるようにする。
- Registry/TypeBinding に「Aggregate の場合は Parser 経由で復元する」分岐を実装し、`bind_type` で個別の bincode デコーダを要求しない API へ移行する。
