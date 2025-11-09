# タスク1.1 完了条件チェックレポート

**タスク**: 起動時検証と監査イベントの実装
**ステータス**: tasks.mdでは `[ ]` (未完了)
**レビュー日時**: 2025-11-09

---

## 完了条件の検証

### タスク1.1の要求事項

**実装内容**:
1. ActorSystem ブートストラップ時に全 `AggregateSchema` を走査
2. 循環・欠落・manifest 衝突・`FieldPathDisplay` 上限超過を検出
3. 起動を停止するフロー
4. 監査フラグが有効なときに EventStream/Telemetry/DeadLetter/監視 API へ検証レポートを発行

**完了条件**:
- 起動時検証をカバーする統合テストが追加され、エラー時のイベント内容がアサートされる

**Requirements**: R2.AC2, R2.AC4, R2.AC5

---

## 実装完了度: **50%** ⭐⭐⭐

### ✅ 実装済み（完全）

#### 1. ActorSystem ブートストラップ時の監査実行
```rust
// system/base.rs:346-351
fn bootstrap<F>(&self, user_guardian_props: &PropsGeneric<TB>, configure: F) -> Result<(), SpawnError> {
    // ...
    let audit_report = serialization.registry().audit();  // ✅ 監査実行
    let audit_event = SerializationAuditEvent::from(&audit_report);
    self.publish_event(&EventStreamEvent::SerializationAudit(audit_event.clone()));  // ✅ EventStream発行
    if !audit_event.success() {
        return Err(SpawnError::invalid_props(SERIALIZATION_AUDIT_FAILED));  // ✅ 起動停止
    }
    // ...
}
```

**評価**: ✅ 完全実装

#### 2. EventStream への SerializationAuditEvent 発行
```rust
// event_stream/event_stream_event.rs:32
pub enum EventStreamEvent<TB: RuntimeToolbox = NoStdToolbox> {
    // ...
    SerializationAudit(SerializationAuditEvent),  // ✅ イベント型追加
}

// event_stream/serialization_event.rs:18-27
pub struct SerializationAuditEvent {
    pub success:         bool,
    pub schemas_checked: usize,
    pub issues:          Vec<SerializationAuditIssue>,  // ✅ 詳細なissue情報
}
```

**評価**: ✅ 完全実装

#### 3. 欠落バインディングの検出
```rust
// serialization/registry.rs:177-195
pub fn audit(&self) -> RegistryAuditReport {
    let schemas_guard = self.aggregate_schemas.lock();
    let type_bindings_guard = self.type_bindings.lock();
    let mut issues = Vec::new();

    for schema in schemas_guard.values() {
        for field in schema.fields() {
            if !field.external_serializer_allowed() && !type_bindings_guard.contains_key(&field.type_id()) {
                issues.push(RegistryAuditIssue {
                    field_path: field.display().as_str().to_string(),
                    type_name:  field.type_name(),
                    reason:     "serializer not registered",  // ✅ 欠落検出
                });
            }
        }
    }

    RegistryAuditReport::new(schemas_guard.len(), issues)
}
```

**評価**: ✅ 完全実装（欠落検出のみ）

#### 4. 監査テストの存在
```rust
// serialization/registry/tests.rs:159-178
#[test]
fn audit_reports_missing_serializer() {
    let registry = SerializerRegistry::<NoStdToolbox>::new();
    // スキーマ登録（シリアライザは未登録）
    registry.register_aggregate_schema(schema).expect("register schema");
    let report = registry.audit();
    assert!(!report.success());  // ✅ 失敗検出
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].reason, "serializer not registered");  // ✅ 理由確認
}

// registry/tests.rs:181-205
#[test]
fn audit_succeeds_when_all_fields_are_bound() {
    // 全フィールドがバインドされた場合の成功テスト
    let report = registry.audit();
    assert!(report.success());  // ✅ 成功検証
    assert_eq!(report.schemas_checked, 1);
    assert!(report.issues.is_empty());
}
```

**評価**: ✅ 実装済み（欠落検出のみ）

---

### ❌ 未実装（要対応）

#### 1. 循環参照の検出（🔴優先度: 高）

**問題**:
```rust
// registry.rs の audit() メソッドに循環検出のコードが存在しない
// 以下のような循環を検出できない:
// struct A { b: B }
// struct B { a: A }  // 循環参照
```

**影響**:
- 循環参照を持つスキーマが登録されるとスタックオーバーフローやデッドロックのリスク
- Requirements R2.AC4 が未達

**推奨実装**:
```rust
pub fn audit(&self) -> RegistryAuditReport {
    let schemas_guard = self.aggregate_schemas.lock();
    let type_bindings_guard = self.type_bindings.lock();
    let mut issues = Vec::new();

    // 循環検出の追加
    for schema in schemas_guard.values() {
        let mut visited = hashbrown::HashSet::new();
        let mut stack = Vec::new();

        if self.detect_cycle(schema, &schemas_guard, &mut visited, &mut stack) {
            issues.push(RegistryAuditIssue {
                field_path: format!("{:?}", stack),  // 循環チェーン
                type_name: schema.root_type_name(),
                reason: "circular reference detected",
            });
        }

        // 既存の欠落検出コード
        for field in schema.fields() {
            // ...
        }
    }

    RegistryAuditReport::new(schemas_guard.len(), issues)
}

fn detect_cycle(
    &self,
    schema: &AggregateSchema,
    all_schemas: &HashMap<TypeId, ArcShared<AggregateSchema>>,
    visited: &mut HashSet<TypeId>,
    stack: &mut Vec<TypeId>,
) -> bool {
    let type_id = schema.root_type();

    if stack.contains(&type_id) {
        return true;  // 循環検出
    }

    if visited.contains(&type_id) {
        return false;  // 既に検証済み
    }

    visited.insert(type_id);
    stack.push(type_id);

    for field in schema.fields() {
        if let Some(child_schema) = all_schemas.get(&field.type_id()) {
            if self.detect_cycle(child_schema, all_schemas, visited, stack) {
                return true;
            }
        }
    }

    stack.pop();
    false
}
```

**見積もり**: 3-4時間

---

#### 2. manifest 衝突の検出（🔴優先度: 高）

**問題**:
```rust
// registry.rs の audit() メソッドに manifest 衝突検出のコードが存在しない
// 異なる型が同じ manifest を持つケースを検出できない:
// Type A → manifest "Foo"
// Type B → manifest "Foo"  // 衝突
```

**影響**:
- デシリアライズ時に間違った型が復元される可能性
- Requirements R2.AC2 が未達

**推奨実装**:
```rust
pub fn audit(&self) -> RegistryAuditReport {
    let schemas_guard = self.aggregate_schemas.lock();
    let type_bindings_guard = self.type_bindings.lock();
    let manifest_bindings_guard = self.manifest_bindings.lock();
    let mut issues = Vec::new();

    // manifest 衝突検出の追加
    let mut manifest_to_types: HashMap<(&u32, &str), Vec<&'static str>> = HashMap::new();

    for binding in type_bindings_guard.values() {
        let key = (&binding.serializer_id(), binding.manifest().as_str());
        manifest_to_types.entry(key).or_default().push(binding.type_name());
    }

    for ((serializer_id, manifest), type_names) in manifest_to_types.iter() {
        if type_names.len() > 1 {
            issues.push(RegistryAuditIssue {
                field_path: format!("serializer={}, manifest={}", serializer_id, manifest),
                type_name: type_names[0],
                reason: "manifest collision detected",
            });
        }
    }

    // 既存の欠落検出・循環検出コード
    // ...

    RegistryAuditReport::new(schemas_guard.len(), issues)
}
```

**見積もり**: 2-3時間

---

#### 3. FieldPathDisplay 上限超過の検出（🟡優先度: 中）

**問題**:
```rust
// registry.rs の audit() メソッドに FieldPathDisplay 長さチェックが存在しない
// MAX_FIELD_PATH_BYTES (96バイト) を超えるケースを検出できない
```

**影響**:
- メモリオーバーフローのリスク
- Requirements R2.AC2 が部分的に未達

**推奨実装**:
```rust
pub fn audit(&self) -> RegistryAuditReport {
    // ...

    for schema in schemas_guard.values() {
        // FieldPathDisplay 長さチェックの追加
        if schema.root_display().as_bytes().len() > MAX_FIELD_PATH_BYTES {
            issues.push(RegistryAuditIssue {
                field_path: schema.root_display().as_str().to_string(),
                type_name: schema.root_type_name(),
                reason: "FieldPathDisplay exceeds maximum length",
            });
        }

        for field in schema.fields() {
            if field.display().as_bytes().len() > MAX_FIELD_PATH_BYTES {
                issues.push(RegistryAuditIssue {
                    field_path: field.display().as_str().to_string(),
                    type_name: field.type_name(),
                    reason: "FieldPathDisplay exceeds maximum length",
                });
            }

            // 既存の欠落検出コード
            // ...
        }
    }

    // ...
}
```

**見積もり**: 1時間

---

#### 4. Telemetry/DeadLetter/監視API への通知（🟡優先度: 中）

**問題**:
- EventStream への発行は実装済み
- しかし、Telemetry、DeadLetter、監視API への直接通知がない

**現状**:
```rust
// system/base.rs:348
self.publish_event(&EventStreamEvent::SerializationAudit(audit_event.clone()));
// ✅ EventStream のみ発行
```

**要求**:
- Telemetry Service への直接通知
- DeadLetter への記録
- 監視API への通知

**状況**: EventStreamサブスクライバーで対応可能だが、直接通知はない

**推奨**: EventStreamサブスクライバーとして実装するか、または明示的なドキュメント化

**見積もり**: 2時間（実装） または 30分（ドキュメント化）

---

#### 5. 監査フラグによる制御（🟢優先度: 低）

**問題**:
- 監査は常に実行される
- フラグによる on/off 制御がない

**要求**:
```
監査フラグが有効なときに EventStream/Telemetry/DeadLetter/監視 API へ検証レポートを発行
```

**現状**: フラグなしで常に発行

**推奨実装**:
```rust
// システム設定に監査フラグを追加
pub struct SystemConfig {
    pub enable_serialization_audit: bool,  // デフォルト: true
}

fn bootstrap<F>(...) -> Result<(), SpawnError> {
    // ...
    if self.config().enable_serialization_audit {
        let audit_report = serialization.registry().audit();
        // ...
    }
    // ...
}
```

**見積もり**: 1-2時間

---

### ⚠️ テストカバレッジ不足（優先度: 高）

**問題**:
- 欠落検出のユニットテストは存在
- しかし、統合テスト（ActorSystem起動失敗）がない
- EventStream発行のテストがない

**要求**:
```
起動時検証をカバーする統合テストが追加され、エラー時のイベント内容がアサートされる
```

**推奨追加テスト**:
```rust
// system/base/tests.rs に追加

#[test]
fn bootstrap_fails_when_serialization_audit_reports_issues() {
    // 1. シリアライザなしでスキーマを登録
    let registry = SerializerRegistry::<NoStdToolbox>::new();
    let mut builder = AggregateSchemaBuilder::<Parent>::new(...);
    builder.add_value_field::<Child>(..., false).expect("add");
    let schema = builder.finish().expect("schema");
    registry.register_aggregate_schema(schema).expect("register");

    // 2. ActorSystemのブートストラップを試行
    let result = ActorSystem::new_with(&user_guardian_props, |system| {
        // configure でレジストリにスキーマを登録済みとする
        Ok(())
    });

    // 3. 監査失敗により起動が失敗することを確認
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, SpawnError::InvalidProps(_)));
}

#[test]
fn bootstrap_publishes_serialization_audit_event() {
    // 1. EventStream サブスクライバーを設定
    let events = Arc::new(Mutex::new(Vec::new()));
    let subscriber = TestSubscriber::new(events.clone());

    // 2. ActorSystem を起動
    let system = ActorSystem::new(&user_guardian_props).expect("bootstrap");
    system.subscribe_event_stream(&subscriber);

    // 3. SerializationAudit イベントが発行されたことを確認
    let captured = events.lock();
    let audit_events: Vec<_> = captured.iter()
        .filter_map(|e| match e {
            EventStreamEvent::SerializationAudit(ae) => Some(ae),
            _ => None,
        })
        .collect();

    assert_eq!(audit_events.len(), 1);
    assert!(audit_events[0].success());
}
```

**見積もり**: 3-4時間

---

## 評価まとめ

| 検出項目 | 実装状況 | テスト | スコア |
|---------|---------|--------|--------|
| 欠落バインディング | ✅ 完全実装 | ✅ ユニット | 100% |
| 循環参照 | ❌ 未実装 | ❌ なし | 0% |
| manifest 衝突 | ❌ 未実装 | ❌ なし | 0% |
| FieldPathDisplay超過 | ❌ 未実装 | ❌ なし | 0% |
| EventStream 発行 | ✅ 完全実装 | ⚠️ 統合なし | 70% |
| 起動停止フロー | ✅ 完全実装 | ⚠️ 統合なし | 70% |
| Telemetry通知 | ⚠️ 間接的 | ❌ なし | 30% |
| DeadLetter通知 | ❌ 未実装 | ❌ なし | 0% |
| 監視API通知 | ❌ 未実装 | ❌ なし | 0% |
| 監査フラグ制御 | ❌ 未実装 | ❌ なし | 0% |

**総合スコア**: (100+0+0+0+70+70+30+0+0+0) / 10 = **27%**

しかし、基盤部分（欠落検出、EventStream、起動停止）が実装済みなので、**50%完了**と評価します。

---

## 推奨アクション

### 🔴 即時対応（タスク1.1完了に必須）
1. **循環参照検出の実装**（3-4時間）
   - DFS/スタックベースの循環検出アルゴリズム
   - 対応するテストケース追加

2. **manifest 衝突検出の実装**（2-3時間）
   - manifest → 型名のマッピング構築
   - 重複検出とエラー報告

3. **統合テストの追加**（3-4時間）
   - ActorSystem起動失敗テスト
   - EventStream発行テスト
   - 各種検出シナリオテスト

### 🟡 短期対応（タスク1.1完了後に推奨）
4. **FieldPathDisplay 長さ検出**（1時間）
5. **Telemetry/DeadLetter/監視API 通知の明確化**（2時間 or ドキュメント化30分）

### 🟢 長期対応（タスク3以降）
6. **監査フラグ制御の追加**（1-2時間）

---

## 結論

タスク1.1は **50%完了** です。

**✅ 実装済みの基盤**:
- ActorSystem ブートストラップでの監査実行
- EventStream への SerializationAuditEvent 発行
- 欠落バインディング検出
- 起動停止フロー
- 基本的なユニットテスト

**❌ 未実装の重要機能**:
- **循環参照検出**（セキュリティ/安定性リスク）
- **manifest 衝突検出**（正確性リスク）
- **統合テスト**（品質保証不足）
- Telemetry/DeadLetter/監視API への直接通知

**推奨**: 循環検出、manifest衝突検出、統合テストの3つを完了させてからタスク2に進むべき。特に**循環検出**と**manifest衝突検出**は実装リスクが高いため優先度が高い。

**実装完了までの見積もり**: 8-11時間（1-2日）

---

**レビュー完了**
