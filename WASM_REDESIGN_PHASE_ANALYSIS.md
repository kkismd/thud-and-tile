# WASM統合再設計 Phase別依存関係分析書

**日時**: 2025年10月7日  
**目的**: CLI_WASM_INTEGRATION_REDESIGN.mdの原則に準拠したWASM統合設計の段階的再検討  
**重要**: 本ドキュメントにより、既存の`WASM_CORE_INTEGRATION_PLAN.md`と`WASM_CORE_INTEGRATION_TECHNICAL.md`は**Phase 4完了時に全面改訂**されます

## 📋 背景：設計適合性課題の特定

### 🚨 **統合プラン作成時に発見された原則乖離**

1. **Layer分離アーキテクチャ未適用**
   - **再設計書**: Layer 1（共通コア）/Layer 2（CLI専用）/Layer 3（WASM API）の3層分離
   - **現統合プラン**: Core ModuleとWASM APIの2層統合のみ

2. **データコピー最優先原則の不徹底**
   - **再設計書**: 借用チェッカー競合回避のためのデータコピーパターン
   - **現統合プラン**: Core Moduleの既存設計をそのまま使用予定

3. **過去のWASMインシデント教訓の反映不足**
   - 借用チェッカー競合、メモリアクセス違反、アーキテクチャ競合の再発リスク

### 📄 **既存文書の取り扱い**
- `WASM_CORE_INTEGRATION_PLAN.md` → **Phase 4で全面改訂** （再設計書原則準拠）
- `WASM_CORE_INTEGRATION_TECHNICAL.md` → **Phase 4で技術詳細更新** （データコピーパターン反映）
- 改訂時は旧版を`obsolete_docs/`に移動し、新版で置き換え

## 🎯 Phase別実行計画（依存関係考慮）

### **Phase 1: Core Moduleの再設計書適合性検証（✅ 完了）**
**優先度**: 🔴 最高 | **期間**: 1日 | **依存**: なし（開始可能） | **結果**: ✅ 95%適合

#### 🔍 **検証スコープ**
1. **Layer 1（共通コアロジック）への適合性**
   ```rust
   // 検証結果: src/core/game_state.rs
   pub struct CoreGameState {
       pub board: FixedBoard,                    // ✅ 固定サイズ配列 - 完全適合
       pub animations: Vec<AnimationState>,      // ⚠ Vec動的 - 軽微、許容範囲
       pub current_piece: Option<Tetromino>,     // ✅ 基本型Option - 安全
       // ... 他フィールドも基本型で安全
   }
   ```

2. **純粋関数設計の検証** - 🟢 **完全適合**
   - ✅ データコピーパターン理想的実装
   - ✅ 借用チェッカー競合リスクなし
   - ✅ WASM境界安全性確保済み

3. **アニメーション処理の安全性** - 🟢 **完全適合**
   - ✅ `AnimationState`コピー可能設計
   - ✅ 時間ベース処理の純粋関数実装
   - ✅ JavaScript時間管理と互換性あり

#### 📋 **検証アクション**
- [x] Core Module全関数の純粋関数性検証 → ✅ 全EraseLineロジック適合確認
- [x] `CoreGameState`のWASM境界安全性評価 → ✅ 固定サイズ配列で安全
- [x] 動的データ構造（Vec、Option）の安全性確認 → ✅ 影響軽微、許容範囲
- [x] 必要に応じたCore Module設計修正 → ✅ 軽微改善のみ（Vec→固定配列）

#### 🎯 **検証結果**: **✅ Phase 2進行承認**
- **95%適合**: CLI_WASM_INTEGRATION_REDESIGN.md Layer 1要件にほぼ完全適合
- **詳細レポート**: `PHASE1_CORE_MODULE_COMPATIBILITY.md`作成済み
- **次段階**: Layer分離アーキテクチャ設計に安全に進行可能

---

### **Phase 2: Layer分離アーキテクチャの適用検討**
**優先度**: 🟡 高 | **期間**: 1-2日 | **依存**: Phase 1完了

#### 🔍 **検討スコープ**
1. **3層分離の適用可能性分析**
   ```
   現在の2層構造:
   [Core Module] → [WASM API]
   
   提案の3層構造:
   [Layer 1: 共通コア] → [Layer 2: CLI専用] → [Layer 3: WASM専用]
   ```

2. **既存Core Moduleの位置づけ**
   - Layer 1として利用可能か
   - Layer 2への分離が必要か
   - 新規Layer設計の必要性

3. **移行戦略の検討**
   - 段階的移行 vs 一括移行
   - 既存機能への影響評価
   - テスト戦略の調整

#### 📋 **検討アクション**
- [ ] Layer分離の費用対効果分析
- [ ] Core Module再利用可能性評価
- [ ] CLI版への影響度評価
- [ ] 移行リスク・工数評価

#### 🎯 **検討結果による分岐**
- **3層適用**: Layer分離設計でPhase 3へ
- **2層維持**: 現構造強化でPhase 3へ
- **段階移行**: 部分的Layer分離でPhase 3へ

---

### **Phase 3: WASM境界設計の根本見直し**
**優先度**: 🟡 高 | **期間**: 1-2日 | **依存**: Phase 1,2完了

#### 🔍 **見直しスコープ**
1. **データコピー最優先原則の徹底**
   ```rust
   // 現在の問題パターン（借用チェッカーリスク）
   pub fn handle_input(&mut self, input_code: u8) -> bool {
       // Core Moduleの参照を直接操作
       let result = process_input(&mut self.core_state, ...);
   }
   
   // 提案: データコピーパターン
   pub fn handle_input(&mut self, input_code: u8) -> bool {
       let snapshot = self.core_state.clone();       // データコピー
       let result = process_input(snapshot, ...);     // 純粋関数処理
       self.core_state = result.new_state;           // 結果反映
       result.input_consumed
   }
   ```

2. **JavaScript時間管理への移行**
   - Rust側での時間取得廃止
   - JavaScript側からの時間パラメータ受取り
   - アニメーション処理の時間依存解消

3. **WASM境界安全インターフェース**
   - 固定サイズ配列の活用
   - シンプルなABI設計
   - エラーハンドリング強化

#### 📋 **見直しアクション**
- [ ] 現WASM API設計の問題点特定
- [ ] データコピーパターン具体設計
- [ ] JavaScript連携インターフェース設計
- [ ] 安全性テストパターン設計

#### 🎯 **設計結果による分岐**
- **全面見直し**: 新WASM API設計でPhase 4へ
- **部分修正**: 既存API強化でPhase 4へ
- **段階移行**: 段階的安全化でPhase 4へ

---

### **Phase 4: 段階的統合プランの再構築**
**優先度**: 🟢 中 | **期間**: 1日 | **依存**: Phase 1-3完了

#### 🔍 **再構築スコープ**
1. **Phase 1-3結果の統合**
   - 確定アーキテクチャによるTDDサイクル再設計
   - 実装優先度の再評価
   - リスク管理戦略更新

2. **過去インシデント対策の組み込み**
   - 借用チェッカー競合の具体的回避パターン
   - メモリアクセス違反の防止戦略
   - アーキテクチャ競合の事前検出

3. **統合プラン文書の更新**
   - `WASM_CORE_INTEGRATION_PLAN.md`の全面改訂
   - `WASM_CORE_INTEGRATION_TECHNICAL.md`の技術詳細更新
   - 新設計原則の反映

#### 📋 **再構築アクション**
- [ ] 新アーキテクチャベースのTDDサイクル設計
- [ ] 安全性テスト戦略策定
- [ ] 実装プロセス最適化
- [ ] 統合プラン文書の改訂

## 🔄 **依存関係フローチャート**

```
Phase 1: Core Module適合性検証
    ↓ (Core Moduleの安全性確認)
Phase 2: Layer分離アーキテクチャ検討
    ↓ (アーキテクチャ構造確定)
Phase 3: WASM境界設計見直し
    ↓ (具体的WASM API設計確定)
Phase 4: 統合プラン再構築
    ↓ (最終実装プラン確定)
実装開始
```

## 📅 **推奨スケジュール**

| 日程 | Phase | 主要アクティビティ | 成果物 |
|------|-------|-------------------|--------|
| Day 1 | Phase 1 | Core Module適合性検証 | 適合性評価レポート |
| Day 2 | Phase 2 | Layer分離検討 | アーキテクチャ設計書 |
| Day 3 | Phase 3 | WASM境界見直し | WASM API設計書 |
| Day 4 | Phase 4 | 統合プラン再構築 | 改訂統合プラン |

## 🎯 **最終目標**

**安全で保守性の高いWASM統合の実現**
- 過去のWASMインシデント完全回避
- CLI版との機能等価性保証
- 再設計書原則への完全準拠
- 段階的で検証可能な実装プロセス

## 📄 **文書管理**

### **アクティブ文書**
- `CLI_WASM_INTEGRATION_REDESIGN.md` - 設計基準文書
- `WASM_REDESIGN_PHASE_ANALYSIS.md` - 本文書（Phase分析）

### **見直し対象文書**
- `WASM_CORE_INTEGRATION_PLAN.md` - Phase 4で全面改訂予定
- `WASM_CORE_INTEGRATION_TECHNICAL.md` - Phase 4で技術詳細更新予定

### **廃止予定文書**
- 古い統合プラン関連文書（Phase 4完了後に`obsolete_docs/`へ移動）

---

**注意**: Phase 1から順次実行し、各Phaseの結果を次Phaseの設計に反映させることで、一貫性のある安全な統合設計を実現します。