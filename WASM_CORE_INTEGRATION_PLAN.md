# WASM統合実装計画書（Phase 1-3再設計結果版）# WASM API Core Module統合 TDDサイクル計画書



**作成日**: 2025年10月7日  ## プロジェクト概要

**基準**: WASM_REDESIGN_PHASE_ANALYSIS.md完了結果  WASM APIレイヤーを完全にCore Moduleベースに移行し、EraseLineアニメーション機能を含む統一的なゲーム状態管理を実現する。

**目標**: 3層分離アーキテクチャによる安全なWASM統合実装

**⚠ 設計コンセプト適合性レビュー結果：**  

---この統合プランは過去のWASMインシデント教訓を活かした`CLI_WASM_INTEGRATION_REDESIGN.md`の新設計コンセプトに**部分的に適合**しているが、以下の重要な原則との乖離があります：



## 📚 **設計基盤（Phase 1-3完了結果）**### 🚨 原則乖離箇所と対応

1. **データコピー最優先**: ✅ 部分適合 - Core Moduleの`process_input`戻り値処理で実現予定

### **✅ Phase 1: Core Module適合性（95%適合確認済み）**2. **責任の明確分離**: ❌ 未適合 - Layer分離設計が統合プランに反映されていない

- **Layer 1基盤**: 既存Core Moduleを共通コアロジックとして活用3. **段階的統合**: ✅ 適合 - 6つのPhase設計で実現

- **純粋関数設計**: 借用チェッカー競合ゼロ確認済み4. **検証可能性**: ✅ 適合 - TDDサイクル設計で実現

- **固定サイズ配列**: WASM境界安全性確保済み

- **詳細**: `PHASE1_CORE_MODULE_COMPATIBILITY.md`参照## 現状分析

### 完了済み

### **✅ Phase 2: 3層分離アーキテクチャ（コンセプト確定済み）**- ✅ Core Module EraseLineアニメーション実装完了（15/15テスト通過）

```- ✅ WasmGameState構造のCore Module基盤への部分移行

Layer 1: 共通コアロジック (src/core/)

    ↑## 特定された改修事項

    ├── Layer 2: CLI専用レイヤー (src/cli/) 1. **入力処理の非統合**: `handle_input`関数が独自実装でCore Moduleの`process_input`未使用

    └── Layer 3: WASM専用レイヤー (src/wasm/)2. **ToggleEraseLine未対応**: `input_code: 8`（ToggleEraseLine）がWASM APIで未実装

```3. **アニメーション状態API不足**: EraseLineアニメーションの状態取得・制御API不在

- **依存関係**: Layer 2⊥Layer 3（相互独立）、両者ともLayer 1に依存4. **イベント処理未統合**: Core Moduleイベント（GameModeChanged等）のWASM境界での処理不足

- **詳細**: `PHASE2_LAYER_SEPARATION_DESIGN.md`参照5. **型変換問題**: WASM境界での安全な型変換とエラーハンドリング不統一

6. **chain_bonus増加機能の統合不足**: 既存の`add_chain_bonus` WASM APIはあるが、Core Moduleの自動chain_bonus増加ロジック（ピース配置時の隣接ブロック処理）がWASM側で未適用

### **✅ Phase 3: WASM境界安全設計（完成済み）**7. **🚨 アーキテクチャレイヤー不統合**: 再設計書のLayer分離（共通コアロジック/CLI専用/WASM API）が現在の統合アプローチに反映されていない

- **データコピー最優先**: 全API関数で借用チェッカー競合回避

- **JavaScript時間管理**: Rust側時間取得完全廃止## TDDサイクル実行計画

- **安全な戻り値型**: `WasmRenderInfo`（固定サイズ配列、プリミティブ型のみ）

- **詳細**: `PHASE3_WASM_BOUNDARY_REDESIGN.md`参照### Phase 1: 統合テストフレームワーク構築

**期間**: 1日  

---**目標**: WASM APIとCore Module間の統合テスト基盤確立

**⚠ 設計適合性**: 再設計書Phase 1（共通コアロジック抽出）との整合性要確認

## 🎯 **実装計画（Phase 4実行内容）**

#### Step 1.1: WASM統合テスト環境構築

### **実装フェーズ 1: Layer 1軽微調整** (0.5日)- WASM bindgenテスト環境セットアップ

**目標**: Phase 1で特定された軽微改善の実装- Core Module - WASM API統合テストケース骨格作成

- JavaScript側との通信テスト基盤構築

#### **1.1: AnimationState固定サイズ化**- **追加必要**: 再設計書Layer分離の適合性検証

```rust

// 現在: Vec<usize> → 改善: [Option<usize>; 4]#### Step 1.2: 基本統合テストケース作成

pub struct AnimationState {- WasmGameState初期化テスト

    pub lines: [Option<usize>; 4],  // 最大4ライン同時消去- Core Module状態同期テスト

    // ... 他フィールド- 基本入力処理統合テスト

}- **追加必要**: データコピーパターンの借用チェッカー競合回避テスト

```

### Phase 2: Core Module入力処理統合

#### **1.2: Core Module微調整****期間**: 1-2日  

- [ ] `src/core/animation_logic.rs`のVec使用箇所修正**目標**: WASMの`handle_input`をCore Moduleの`process_input`ベースに完全移行

- [ ] ヘルパー関数追加（`get_active_lines()`, `line_count()`）**⚠ 設計適合性**: 再設計書のWASM APIレイヤー（Layer 3）設計原則準拠要確認

- [ ] 既存テスト更新（15/15テスト → 完全互換維持）

#### Step 2.1: 入力処理統合テスト作成

---- 全GameInput種別のWASM API経由テスト

- ToggleEraseLine（input_code: 8）専用テスト

### **実装フェーズ 2: Layer 2（CLI専用レイヤー）実装** (1.2日)

**🚨 重要**: 過去のAPI変更でテスト大量失敗→ロールバック事象対策済み

**戦略**: 段階的移行（Strangler Fig Pattern）+ テスト駆動 + 後方互換性

#### **2.1: 安全なCLI Layer基盤作成（0.3日）**
**リスク**: ゼロ（既存コード無変更）

```rust
// src/cli/mod.rs - 新規作成
pub mod renderer;     // render.rs → Layer 2移行先  
pub mod input;        // game_input.rs → Layer 2移行先
pub mod scheduler;    // scheduler.rs → Layer 2移行先
pub mod bridge;       // CLI-Core統合管理
```

**検証**: `cargo test` → 92/92 passed 維持必須

#### **2.2: 段階的機能移行（0.8日）**

**2.2.1 描画機能移行（0.2日）**
```rust  
// 目標: render.rs → src/cli/renderer.rs
// 理由: 独立性高、依存関係最小でリスク最小

// 後方互換API保持（テスト変更なし）
pub fn draw_game_state(renderer: &mut CrosstermRenderer, state: &GameState) {
    // 内部でLayer 2呼び出し
    cli::renderer::draw(renderer, state);
}

// Layer 2新実装
pub mod cli {
    pub mod renderer {
        pub fn draw(renderer: &mut CrosstermRenderer, state: &GameState) {
            // Layer 2実装
        }
    }
}
```
**ロールバック**: `git checkout -- src/render.rs` (影響局所化)

**2.2.2 入力機能移行（0.3日）**
```rust
// 目標: game_input.rs → src/cli/input.rs  
// 前提: 2.2.1完了、テスト通過確認済み

// InputProviderトレイト保持（既存テスト維持）
impl InputProvider for CrosstermInputProvider {
    fn get_input(&mut self) -> GameInput {
        // 内部でLayer 2呼び出し
        cli::input::process()
    }
}
```
**ロールバック**: `git checkout -- src/game_input.rs`

**2.2.3 スケジューリング機能移行（0.3日）**
```rust
// 目標: scheduler.rs → src/cli/scheduler.rs
// 前提: 2.2.1, 2.2.2完了、テスト通過確認済み

// Schedulerトレイト保持（時間管理API維持）
pub fn create_default_scheduler() -> impl Scheduler {
    // 内部でLayer 2実装返却
    cli::scheduler::DefaultScheduler::new()
}
```
**ロールバック**: `git checkout -- src/scheduler.rs`

#### **2.3: 統合と最適化（0.1日）**
**前提**: 全移行完了、92テスト通過確認済み

1. main.rs でLayer 2 API直接呼び出し
2. 旧実装ファイル削除  
3. 最終統合テスト実行
4. **緊急時**: `git reset --hard phase2-start`でフェーズ全体復元

pub struct CliGameState {- Core Moduleの`lock_current_piece`機能をWASM APIに統合

    pub core: CoreGameState,           // Layer 1活用- ピース配置時の隣接ブロック処理とchain_bonus自動増加の実装

    pub time_provider: TimeProvider,   // CLI特化時間管理- 既存の`add_chain_bonus` API（手動用）との統合テスト

    pub renderer_state: RendererState, // CLI特化描画状態- **重要**: 共通コアロジック（再設計書Layer 1）からの純粋関数活用

}

### Phase 3: アニメーション状態API統合

impl CliGameState {**期間**: 1日  

    pub fn update_animations(&mut self) {**目標**: EraseLineアニメーション状態の完全なWASM API露出

        let current_time_ms = self.time_provider.now_ms();

        #### Step 3.1: アニメーション状態テスト作成

        // Layer 1純粋関数使用- `has_active_erase_line_animation()` APIテスト

        self.core.animations = crate::core::animation_logic::update_animation_states(- `get_erase_line_animation_progress()` APIテスト

            &self.core.animations,- アニメーション完了イベント処理テスト

            current_time_ms,

        );#### Step 3.2: アニメーション状態API実装

    }- Core Moduleアニメーション状態のWASM境界露出

}- JavaScript側で使いやすい形式での状態返却

```- アニメーション進行度情報の詳細提供



#### **2.3: 既存CLI実装の移行**### Phase 4: イベント処理統合

- [ ] `src/main.rs`をLayer 2使用に更新**期間**: 1日  

- [ ] `src/render.rs`をCLI Layer統合**目標**: Core ModuleイベントのWASM境界での統一処理

- [ ] CLI特化機能の分離（time_provider、terminal操作等）

#### Step 4.1: イベント処理テスト作成

---- GameModeChangedイベント処理テスト

- EraseLineAnimationStarted/Completedイベントテスト

### **実装フェーズ 3: Layer 3（WASM専用レイヤー）実装** (1.5日)

**🚨 重要**: CLI移行以外のマイグレーションリスク対策統合済み

**目標**: Phase 3設計に基づく安全なWASM境界実装

#### **3.1: WASM Layer基盤作成（0.3日）**
**リスク**: WASMテスト失敗による統合停止

```rust
// src/wasm/mod.rs
pub mod wasm_game_engine;
pub mod wasm_types;
pub mod wasm_animation;
pub mod wasm_bridge;
```

**検証**: WASMテスト保護戦略実行
```bash
# 基盤作成後の全環境テスト
cargo test --lib                    # 92 tests → 維持必須
cargo test --features wasm-test     # 171 tests → 影響監視
cargo test --features nodejs-test   # Node.js環境確認

# Feature Flag整合性確認
for feature in native-bin wasm wasm-test nodejs-test; do
    cargo test --features $feature || exit 1
done
```

#### **3.2: WasmGameEngine段階的実装（0.7日）**
**戦略**: 後方互換API保持 + 段階的内部移行

```rust
// src/wasm/wasm_game_engine.rs
#[wasm_bindgen]
pub struct WasmGameEngine {
    core_snapshot: CoreGameState,  // Layer 1データコピー保持
    last_update_ms: u64,
    last_error_code: u32,

}

### Phase 6: 総合統合テスト・性能最適化

#[wasm_bindgen]**期間**: 1日  

impl WasmGameEngine {**目標**: 完全統合テストとパフォーマンス検証

    /// JavaScript時間管理でアニメーション更新

    #[wasm_bindgen]#### Step 6.1: 総合統合テスト作成

    pub fn update_with_time(&mut self, js_time_ms: f64) -> WasmRenderInfo {- EraseLineアニメーション完全フローのWASM APIテスト

        let time_ms = js_time_ms as u64;- CLI版とWASM版の動作等価性確認テスト

        - 大量入力処理の性能テスト

        // Layer 1純粋関数使用（借用チェッカー安全）

        self.core_snapshot.animations = crate::core::animation_logic::update_animation_states(#### Step 6.2: 最終統合・最適化

            &self.core_snapshot.animations,- 全機能統合テスト実行・修正

            time_ms,- WASM境界パフォーマンス最適化

        );- 完全なCore Module - WASM API統合達成

        

        self.create_render_info()  // データコピー返却## 成功指標

    }1. **機能完全性**: CLI版と同等のEraseLineアニメーション機能をWASM APIで実現

    2. **テストカバレッジ**: 全WASM API機能の統合テスト100%通過

    /// EraseLineアニメーション開始3. **型安全性**: WASM境界での型変換エラー0件

    #[wasm_bindgen]4. **性能**: Core Module統合によるオーバーヘッド最小化

    pub fn start_erase_line_animation(&mut self) -> bool {5. **保守性**: 単一のCore Moduleでの統一状態管理

        // Layer 1純粋関数使用

        let solid_lines = crate::core::erase_line_logic::count_solid_lines_from_bottom(## リスク管理

            self.core_snapshot.board- **WASM制約**: wasm-bindgenの制約による実装制限の事前調査

        );- **型変換コスト**: WASM境界での型変換コスト最小化戦略

        - **デバッグ困難性**: WASM環境でのデバッグツール・ログ戦略確立

        let erase_count = crate::core::erase_line_logic::determine_erase_line_count(- **テスト環境**: ブラウザ環境でのテスト実行の安定性確保

            self.core_snapshot.chain_bonus,- **chain_bonus同期**: Core Moduleとの自動増加処理の確実な同期確保

            solid_lines,- **🚨 借用チェッカー競合**: 再設計書で特定された過去インシデントの再発防止（データコピー最優先の徹底）

        );- **アーキテクチャ不整合**: 現統合プランと再設計書Layer分離設計の整合性確保

        

        if erase_count > 0 {## 最終目標

            let new_animation = crate::core::animation_logic::create_erase_line_animation(Core ModuleベースのWASM APIにより、EraseLineアニメーション機能を含む完全な統一ゲーム状態管理を実現し、CLI版とWASM版の機能等価性を達成する。

                (0..erase_count).collect(),

                self.last_update_ms,**⚠ 重要な設計適合性課題:**  

            );現在の統合プランは再設計書の核心原則に部分的に適合していますが、Layer分離アーキテクチャ（共通コアロジック/CLI専用/WASM API）への準拠が不足しています。実装前に以下の対応が必要です：

            self.core_snapshot.animations.push(new_animation);

            true1. **Layer 1（共通コアロジック）の確立**: Core Moduleが既にこの役割を担っているかの検証

        } else {2. **Layer 3（WASM APIレイヤー）の設計見直し**: データコピー最優先原則の徹底

            false3. **借用チェッカー競合回避**: 過去のWASMインシデント再発防止の具体的実装パターン確立
        }
    }
    
    /// 入力処理（データコピーパターン）
    #[wasm_bindgen]
    pub fn handle_input(&mut self, input_code: u8) -> bool {
        match crate::core::input_handler::process_input(
            self.core_snapshot.clone(),  // データコピー
            input_code,
            self.last_update_ms,
        ) {
            Ok(new_state) => {
                self.core_snapshot = new_state;
                true
            }
            Err(_) => {
                self.last_error_code = 1;
                false
            }
        }
    }
}
```

#### **3.3: JavaScript安全な戻り値型実装**
```rust
// src/wasm/wasm_types.rs
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmRenderInfo {
    board_data: [[u8; BOARD_WIDTH]; BOARD_HEIGHT],
    score: u64,
    lines_cleared: u32,
    chain_bonus: u32,
    animation_count: usize,
}

#[wasm_bindgen]
impl WasmRenderInfo {
    #[wasm_bindgen]
    pub fn get_board_data(&self) -> js_sys::Uint8Array {
        let flat_data: Vec<u8> = self.board_data
            .iter()
            .flat_map(|row| row.iter())
            .cloned()
            .collect();
        js_sys::Uint8Array::from(&flat_data[..])
    }
    
    #[wasm_bindgen(getter)]
    pub fn score(&self) -> u64 { self.score }
    
    #[wasm_bindgen(getter)]  
    pub fn chain_bonus(&self) -> u32 { self.chain_bonus }
    
    // ... 他のgetter
}
```

---

### **実装フェーズ 4: 統合テストと検証** (1日)
**目標**: 3層統合の安全性検証

#### **4.1: Layer統合テスト**
- [ ] Layer 1単体テスト（既存15/15テスト拡張）
- [ ] Layer 2 CLI統合テスト（既存CLI機能同等性確認）
- [ ] Layer 3 WASM境界テスト（JavaScript統合確認）

#### **4.2: 安全性検証**
```rust
#[cfg(test)]
mod integration_safety_tests {
    #[test]
    fn test_no_borrow_checker_conflicts() {
        let mut engine = WasmGameEngine::new();
        
        // 並行呼び出しテスト
        let _result1 = engine.update_with_time(100.0);
        let _result2 = engine.handle_input(32);
        let _result3 = engine.start_erase_line_animation();
        
        // 全て成功すれば借用チェッカー安全
    }
    
    #[test]
    fn test_memory_safety() {
        let mut engine = WasmGameEngine::new();
        
        // 大量データ処理テスト
        for i in 0..1000 {
            let result = engine.update_with_time(i as f64);
            assert!(result.score() >= 0);
        }
    }
}
```

#### **4.3: TypeScript統合テスト**
```typescript
describe('WASM統合安全性', () => {
    let engine: WasmGameEngine;
    
    test('EraseLineアニメーション統合', () => {
        engine = new WasmGameEngine();
        
        // アニメーション開始
        const started = engine.start_erase_line_animation();
        
        // 時間経過シミュレーション
        for (let t = 0; t < 500; t += 16) {
            const result = engine.update_with_time(t);
            expect(result.score).toBeGreaterThanOrEqual(0);
        }
    });
});
```

---

## 📋 **マイルストーン・成果物**

### **フェーズ 1完了時**
- ✅ Layer 1軽微調整完了
- ✅ AnimationState固定サイズ化
- ✅ 既存テスト15/15継続通過

### **フェーズ 2完了時**
- ✅ CLI Layer実装完了
- ✅ Layer 1-2統合確認
- ✅ 既存CLI機能同等性確認

### **フェーズ 3完了時**
- ✅ WASM Layer実装完了
- ✅ JavaScript安全API実装
- ✅ データコピーパターン完全実装

### **フェーズ 4完了時**
- ✅ 3層統合検証完了
- ✅ 過去WASMインシデント回避確認
- ✅ 本番デプロイ準備完了

---

## 🚨 **リスク管理**

### **技術リスク**
1. **借用チェッカー競合**: データコピーパターン徹底で回避
2. **メモリ安全性**: 固定サイズ配列とプリミティブ型使用で回避
3. **パフォーマンス**: Layer分離オーバーヘッド（軽微と予想）

### **スケジュールリスク**
- **総期間**: 4日（各フェーズ並行実行可能部分あり）
- **クリティカルパス**: Layer 3実装（最も複雑）
- **緩和策**: Phase 3設計完了により実装リスク軽減済み

### **品質リスク**
- **回避策**: 段階的テスト（各Layer独立検証）
- **最終検証**: 統合テストによる全体動作確認
- **フォールバック**: 既存CLI版は影響なし（Layer 2独立）

---

## 🎯 **成功基準**

1. **✅ 機能同等性**: CLI版とWASM版の完全同等動作
2. **✅ 安全性**: 過去WASMインシデントゼロ
3. **✅ 保守性**: 3層分離による明確な責任分担
4. **✅ 拡張性**: 新機能追加時のLayer独立性確保

---

**実装開始準備完了**: このプランに基づき、フェーズ 1から段階的実装を開始できます。