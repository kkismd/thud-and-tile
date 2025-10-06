# 段階的移行実装計画

**日時:** 2025年10月6日  
**目標:** CLI版からWASM統合版への安全で段階的な移行  
**期間:** 2-3週間の段階的実装  
**リスク管理:** 各段階での検証とロールバック戦略

## 🎯 移行戦略概要

### 基本方針
1. **段階的実装**: 機能別に順次移行、各段階で完全検証
2. **リスク最小化**: 既存CLI版の安定性を損なわない
3. **検証完全性**: 各段階で CLI版とWASM版の動作同等性確認
4. **ロールバック準備**: 各段階で問題発生時の即座復旧
5. **品質保証**: 90%以上のテストカバレッジ維持

## 📅 3週間実装スケジュール

### 🚀 Week 1: Core Logic 抽出・統合 (7日間)

#### Day 1-2: 純粋関数抽出
**目標:** 既存CLI実装から純粋関数を抽出し、共通コアロジックを構築

**作業内容:**
```bash
# 1. Core Logic モジュール作成
mkdir src/core
touch src/core/mod.rs
touch src/core/animation_logic.rs
touch src/core/board_logic.rs  
touch src/core/game_state.rs

# 2. 既存実装から純粋関数抽出
# animation.rs → core/animation_logic.rs
# board_logic.rs → core/board_logic.rs
```

**実装詳細:**
```rust
// src/core/animation_logic.rs
pub fn update_line_blink_state(
    lines: Vec<usize>,
    start_time_ms: u64,
    current_time_ms: u64,
) -> LineBlinkUpdateResult {
    let elapsed = current_time_ms.saturating_sub(start_time_ms);
    let blink_step_ms = 120;
    let max_count = 6;
    let count = (elapsed / blink_step_ms) as usize;
    
    LineBlinkUpdateResult {
        is_completed: count >= max_count,
        current_count: count,
        is_visible: (count % 2) == 0,
    }
}

pub fn calculate_push_down_result(
    board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
    gray_line_y: usize,
    current_board_height: usize,
) -> PushDownCalculationResult {
    // 純粋関数でPushDown結果計算
}
```

**検証方法:**
- 抽出した純粋関数の単体テスト作成
- 既存CLI実装との結果同等性テスト
- パフォーマンス回帰テストの実行

#### Day 3-4: 統合ゲーム状態設計
**目標:** CLI版とWASM版で共有する統合ゲーム状態構造の実装

**実装詳細:**
```rust
// src/core/game_state.rs
#[derive(Debug, Clone)]
pub struct CoreGameState {
    pub board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
    pub current_board_height: usize,
    pub animations: Vec<AnimationState>,
    pub score: u64,
    pub lines_cleared: u32,
    pub game_mode: GameMode,
}

impl CoreGameState {
    /// 時間ベース状態更新 (純粋関数)
    pub fn update_with_time(self, current_time_ms: u64) -> CoreGameStateUpdateResult {
        // self を consume して新しい状態を生成
        // 借用チェッカー競合完全回避
    }
    
    /// アニメーション開始 (純粋関数)
    pub fn start_line_blink(mut self, lines: Vec<usize>, start_time_ms: u64) -> Self {
        self.animations.push(AnimationState::LineBlink {
            lines,
            start_time_ms,
            current_count: 0,
        });
        self
    }
}
```

**検証方法:**
- CoreGameState の状態遷移テスト
- メモリ使用量の測定とリーク検出
- 複数アニメーション同時処理テスト

#### Day 5-6: CLI版リファクタリング
**目標:** 既存CLI実装を新しいCore Logic使用に移行

**実装詳細:**
```rust
// src/cli/game_runner.rs (新設計版)
pub struct CliGameRunner {
    core_state: CoreGameState,
    time_provider: Box<dyn TimeProvider>,
    renderer: Box<dyn Renderer>,
}

impl CliGameRunner {
    pub fn update(&mut self) {
        let current_time_ms = self.time_provider.now().as_millis() as u64;
        
        // Core Logic使用 (借用チェッカー安全)
        let update_result = self.core_state
            .clone() // 明示的クローン
            .update_with_time(current_time_ms);
        
        self.core_state = update_result.new_state;
        
        // CLI特化処理
        self.handle_completed_animations(&update_result.completed_animations);
    }
}
```

**検証方法:**
- 既存CLI版との完全動作同等性テスト
- パフォーマンス比較 (レンダリング速度、メモリ使用量)
- 長時間実行安定性テスト (8時間連続実行)

#### Day 7: Week 1 統合テスト・検証
**目標:** Week 1実装の完全検証とWeek 2準備

**検証項目:**
1. Core Logic単体テストの完全パス
2. CLI版リファクタリング後の機能同等性確認
3. パフォーマンス回帰なしの確認
4. メモリリーク検出テストの実行

### 🌐 Week 2: WASM API 実装 (7日間)

#### Day 8-9: WASM データ構造実装
**目標:** WASM境界安全なデータ構造とAPI基盤の構築

**実装詳細:**
```rust
// src/wasm/data_structures.rs
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct WasmAnimationState {
    animation_type: u32,
    line_0: u32, // None = u32::MAX
    line_1: u32,
    line_2: u32, 
    line_3: u32,
    current_step: u32,
    start_time_ms: u64,
    is_completed: bool,
}

#[wasm_bindgen]
impl WasmAnimationState {
    #[wasm_bindgen(getter)]
    pub fn is_line_visible(&self, current_time_ms: u64) -> bool {
        // Core Logic使用
        use crate::core::animation_logic::calculate_line_visibility;
        calculate_line_visibility(
            self.animation_type,
            current_time_ms.saturating_sub(self.start_time_ms),
        )
    }
}

// Core State → WASM State変換
impl From<&CoreGameState> for WasmGameStateSnapshot {
    fn from(core_state: &CoreGameState) -> Self {
        // 安全な変換処理
    }
}
```

#### Day 10-11: WasmGameEngine 実装
**目標:** メインWASMエンジンとJavaScript統合APIの実装

**実装詳細:**
```rust
// src/wasm/game_engine.rs
#[wasm_bindgen]
pub struct WasmGameEngine {
    core_state: CoreGameState,
    last_update_time_ms: u64,
}

#[wasm_bindgen]
impl WasmGameEngine {
    #[wasm_bindgen]
    pub fn update(&mut self, current_time_ms: f64) -> WasmUpdateResult {
        let time_ms = current_time_ms as u64;
        
        // Core Logic使用 (借用チェッカー安全)
        let update_result = self.core_state
            .clone() // WASM境界では常にクローン
            .update_with_time(time_ms);
        
        self.core_state = update_result.new_state;
        self.last_update_time_ms = time_ms;
        
        // WASM境界安全な結果返却
        WasmUpdateResult::from_core_result(update_result)
    }
}
```

#### Day 12-13: JavaScript統合・TypeScript実装
**目標:** TypeScript型定義とアニメーションループの実装

**実装詳細:**
```typescript
// wasm_types.ts
export interface WasmGameEngine {
    new(): WasmGameEngine;
    update(currentTimeMs: number): WasmUpdateResult;
    get_state(): WasmGameStateSnapshot;
    trigger_line_clear(lines: number[], startTimeMs: number): boolean;
}

// animation_loop.ts  
export class GameAnimationLoop {
    private engine: WasmGameEngine;
    private isRunning: boolean = false;
    
    constructor() {
        this.engine = new WasmGameEngine();
    }
    
    start(): void {
        this.isRunning = true;
        this.loop();
    }
    
    private loop = (): void => {
        if (!this.isRunning) return;
        
        try {
            const currentTime = performance.now();
            const result = this.engine.update(currentTime);
            
            this.handleCompletedAnimations(result);
            this.render();
            
        } catch (error) {
            console.error('Animation loop error:', error);
            // エラー時も継続 (フェイルセーフ)
        }
        
        requestAnimationFrame(this.loop);
    };
}
```

#### Day 14: Week 2 統合テスト・検証
**目標:** WASM API完全動作確認とWeek 3準備

**検証項目:**
1. WASM API単体テストの完全パス
2. JavaScript統合動作の確認
3. CLI版とWASM版の動作同等性テスト
4. ブラウザでの基本動作確認

### ✅ Week 3: 統合テスト・完成 (7日間)

#### Day 15-16: CLI-WASM同等性テスト
**目標:** CLI版とWASM版の完全動作同等性検証

**テスト内容:**
```rust
// tests/cli_wasm_equivalence_tests.rs
#[test]
fn test_line_blink_equivalence() {
    let initial_state = create_test_state_with_complete_lines();
    
    // CLI版実行
    let mut cli_runner = CliGameRunner::new_with_state(initial_state.clone());
    let cli_results = run_cli_simulation(&mut cli_runner, 1000); // 1秒間
    
    // WASM版実行  
    let mut wasm_engine = WasmGameEngine::new_with_state(initial_state);
    let wasm_results = run_wasm_simulation(&mut wasm_engine, 1000);
    
    // 結果同等性検証
    assert_eq!(cli_results.final_score, wasm_results.final_score);
    assert_eq!(cli_results.final_board, wasm_results.final_board);
    assert_eq!(cli_results.animation_events, wasm_results.animation_events);
}
```

#### Day 17-18: パフォーマンス・安定性テスト
**目標:** 長時間実行とパフォーマンス特性の検証

**テスト内容:**
- 8時間連続実行テスト (メモリリーク検出)
- 60FPS描画安定性テスト
- 大量アニメーション同時処理テスト
- ブラウザメモリ使用量測定

#### Day 19-20: エラー処理・フェイルセーフテスト
**目標:** WASM境界でのエラー処理とフェイルセーフ機能の検証

**テスト内容:**
```rust
#[test]
fn test_wasm_error_resilience() {
    let mut engine = WasmGameEngine::new();
    
    // 異常入力テスト
    assert!(!engine.trigger_line_clear(&[999], 0.0)); // 無効ライン
    assert!(!engine.trigger_line_clear(&[], 0.0)); // 空配列
    
    // 異常時間テスト
    let _ = engine.update(-1.0); // 負の時間
    let _ = engine.update(f64::MAX); // 極大値
    
    // エンジンが安定状態を維持することを確認
    let state = engine.get_state();
    assert_eq!(state.score(), 0);
}
```

#### Day 21: 最終統合・ドキュメント完成
**目標:** 全体統合の最終確認とドキュメント整備

## 🛡️ リスク管理・ロールバック戦略

### 各段階でのロールバック条件
1. **テストカバレッジ90%未満**: 次段階進行中止
2. **パフォーマンス10%以上劣化**: 実装見直し
3. **メモリリーク検出**: 直ちに修正
4. **CLI版動作変更**: 即座にロールバック

### ロールバック手順
```bash
# 緊急ロールバック (各段階で実行可能)
git checkout main
git clean -fd
cargo test --all
./run_cli_integration_tests.sh

# 段階別ロールバック
git checkout week1-core-logic    # Week 1完了時点
git checkout week2-wasm-api      # Week 2完了時点
```

### 品質ゲート
各週の終了時に以下を満たすことを必須とする:
- [ ] 全テストパス (単体・統合)
- [ ] パフォーマンス回帰なし
- [ ] メモリリーク検出なし
- [ ] CLI版動作同等性確認

## 📋 実装チェックリスト

### Week 1: Core Logic
- [ ] `src/core/animation_logic.rs` 純粋関数実装
- [ ] `src/core/board_logic.rs` ボード処理抽出
- [ ] `src/core/game_state.rs` 統合状態構造
- [ ] CLI版リファクタリング完了
- [ ] 90%以上テストカバレッジ達成
- [ ] パフォーマンス回帰なし確認

### Week 2: WASM API
- [ ] `src/wasm/data_structures.rs` WASM安全構造
- [ ] `src/wasm/game_engine.rs` メインエンジン
- [ ] TypeScript型定義・統合
- [ ] JavaScript アニメーションループ
- [ ] 基本WASM動作確認
- [ ] エラー処理実装

### Week 3: 統合・完成
- [ ] CLI-WASM同等性テスト全パス
- [ ] 8時間安定性テスト成功
- [ ] 60FPS描画性能達成
- [ ] メモリリーク検出なし
- [ ] エラー処理・フェイルセーフ確認
- [ ] ドキュメント完成

## 🎯 成功基準

### 技術的成功基準
1. **機能同等性**: CLI版とWASM版で同じ入力に対して同じ出力
2. **安全性**: WASM関連panic/エラーの完全回避 (0件)
3. **パフォーマンス**: 60FPS描画での8時間安定動作
4. **品質**: 90%以上のテストカバレッジ
5. **保守性**: 共通ロジック重複排除率95%以上

### プロジェクト成功基準
1. **スケジュール**: 3週間以内での完了
2. **品質**: 本番環境での安定動作
3. **拡張性**: 新機能追加時の両版同時対応
4. **ドキュメント**: 保守・拡張のための完全ドキュメント
5. **知見**: 今後のRust-WASM開発のベストプラクティス確立

---

**この段階的移行計画により、CLI版の完成した機能を安全かつ確実にWASM版に統合し、過去のインシデントを完全に回避した高品質なゲームシステムを構築します。**