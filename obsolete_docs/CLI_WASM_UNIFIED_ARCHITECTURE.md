# CLI-WASM統合アーキテクチャ戦略

**日時:** 2025年10月6日  
**目標:** CLI版完成機能とWASM版の安全で効率的な統合アーキテクチャ  
**基盤:** Phase C1 TDD実装完了、WASM API設計完了

## 🎯 統合アーキテクチャ概要

### 3層分離設計
```
┌─────────────────────────────────────────────────┐
│                 Web Frontend                    │
│              (TypeScript/Canvas)                │
│    ┌─────────────────┐  ┌─────────────────┐    │
│    │  Animation Loop │  │   Game UI       │    │
│    │  Rendering      │  │   Controls      │    │
│    └─────────────────┘  └─────────────────┘    │
└─────────────────┬───────────────────────────────┘
                  │ WASM Bindings (safe API)
┌─────────────────▼───────────────────────────────┐
│              WASM API Layer                     │
│           (Rust + wasm-bindgen)                 │
│    ┌─────────────────┐  ┌─────────────────┐    │
│    │ WasmGameEngine  │  │  Safe Data      │    │
│    │ Error Handling  │  │  Structures     │    │
│    └─────────────────┘  └─────────────────┘    │
└─────────────────┬───────────────────────────────┘
                  │ Pure Functions (no borrowing)
┌─────────────────▼───────────────────────────────┐
│               Core Logic                        │
│           (Pure Rust Functions)                 │
│    ┌─────────────────┐  ┌─────────────────┐    │
│    │   Animation     │  │  Game Logic     │    │
│    │   Processing    │  │  Board Logic    │    │
│    └─────────────────┘  └─────────────────┘    │
└─────────────────┬───────────────────────────────┘
                  │ Direct Function Calls
┌─────────────────▼───────────────────────────────┐
│              CLI Interface                      │
│            (Rust + CrossTerm)                   │
│    ┌─────────────────┐  ┌─────────────────┐    │
│    │ CLI Rendering   │  │  Terminal I/O   │    │
│    │ Game Loop       │  │  Time Provider  │    │
│    └─────────────────┘  └─────────────────┘    │
└─────────────────────────────────────────────────┘
```

## 📚 モジュール構成詳細

### Core Logic Layer (src/core/)
```rust
//! 純粋なゲームロジック - プラットフォーム非依存
//! CLI版、WASM版共通の基盤

// core/animation_logic.rs
pub mod animation_logic {
    /// アニメーション状態の純粋関数処理
    pub fn update_animation_state(
        animation: AnimationState,
        current_time_ms: u64,
    ) -> AnimationUpdateResult {
        // 借用チェッカー競合なし - 値渡し・値返し
    }
    
    pub fn calculate_line_visibility(
        animation_type: AnimationType,
        elapsed_ms: u64,
        line_y: usize,
    ) -> bool {
        // LineBlink点滅計算
    }
    
    pub fn process_animation_completion(
        completed_animation: AnimationState,
        board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
    ) -> AnimationCompletionResult {
        // 完了時の状態変更計算
    }
}

// core/board_logic.rs
pub mod board_logic {
    pub fn detect_complete_lines(
        board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
        board_height: usize,
    ) -> Vec<usize> {
        // ライン完成検出
    }
    
    pub fn apply_push_down_step(
        board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
        gray_line_y: usize,
    ) -> PushDownResult {
        // Push Down 1ステップ計算
    }
    
    pub fn erase_line_step(
        board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
        target_lines: &[usize],
        current_step: usize,
    ) -> EraseLineResult {
        // EraseLine 1ステップ計算
    }
}

// core/game_state.rs
pub mod game_state {
    /// プラットフォーム非依存のゲーム状態
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
        /// 純粋関数による状態更新
        pub fn update_with_time(
            self,
            current_time_ms: u64,
        ) -> GameStateUpdateResult {
            // self を consume、新しい状態を返却
            // 借用チェッカー競合完全回避
        }
        
        /// アニメーション開始 (純粋関数)
        pub fn start_line_blink(
            mut self,
            lines: Vec<usize>,
            start_time_ms: u64,
        ) -> Self {
            // 新しいアニメーション追加して自分を返却
        }
    }
}
```

### CLI Integration Layer (src/cli/)
```rust
//! CLI特化機能 - 既存実装の活用

// cli/game_runner.rs
pub mod game_runner {
    use crate::core::game_state::CoreGameState;
    
    /// CLI版ゲーム実行器
    pub struct CliGameRunner {
        core_state: CoreGameState,
        time_provider: Box<dyn TimeProvider>,
        renderer: Box<dyn Renderer>,
    }
    
    impl CliGameRunner {
        pub fn update(&mut self) {
            let current_time_ms = self.time_provider.now().as_millis() as u64;
            
            // Core Logic使用
            let update_result = self.core_state
                .clone() // 明示的クローン
                .update_with_time(current_time_ms);
            
            self.core_state = update_result.new_state;
            
            // CLI特化処理
            self.handle_cli_specific_updates(&update_result);
        }
        
        pub fn render(&self) {
            // Core Logic使用
            let render_data = self.core_state.calculate_render_data(
                self.time_provider.now().as_millis() as u64
            );
            
            // CLI特化描画
            self.renderer.render(&render_data);
        }
    }
}

// cli/input_handler.rs - 既存実装活用
// cli/terminal_renderer.rs - 既存実装活用
```

### WASM API Layer (src/wasm/)
```rust
//! WASM境界API - JavaScript安全インターフェース

// wasm/game_engine.rs
use wasm_bindgen::prelude::*;
use crate::core::game_state::CoreGameState;

#[wasm_bindgen]
pub struct WasmGameEngine {
    core_state: CoreGameState,
    last_update_time_ms: u64,
}

#[wasm_bindgen]
impl WasmGameEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGameEngine {
        WasmGameEngine {
            core_state: CoreGameState::new(),
            last_update_time_ms: 0,
        }
    }
    
    /// JavaScript時間管理でのアップデート
    #[wasm_bindgen]
    pub fn update(&mut self, current_time_ms: f64) -> WasmUpdateResult {
        let time_ms = current_time_ms as u64;
        
        // Core Logic使用 (借用チェッカー安全)
        let update_result = self.core_state
            .clone() // WASM境界では常にクローン
            .update_with_time(time_ms);
        
        self.core_state = update_result.new_state;
        self.last_update_time_ms = time_ms;
        
        // WASM境界安全な結果変換
        WasmUpdateResult::from_core_result(update_result)
    }
    
    /// 状態取得 (スナップショット)
    #[wasm_bindgen]
    pub fn get_state(&self) -> WasmGameStateSnapshot {
        WasmGameStateSnapshot::from_core_state(&self.core_state)
    }
    
    /// ライン消去トリガー
    #[wasm_bindgen]
    pub fn trigger_line_clear(&mut self, lines_js: &[u32], start_time_ms: f64) -> bool {
        let lines: Vec<usize> = lines_js.iter()
            .filter_map(|&line| {
                if line < BOARD_HEIGHT as u32 {
                    Some(line as usize)
                } else {
                    None
                }
            })
            .collect();
        
        if lines.is_empty() || lines.len() > 4 {
            return false;
        }
        
        // Core Logic使用
        self.core_state = self.core_state
            .clone()
            .start_line_blink(lines, start_time_ms as u64);
        
        true
    }
}

// wasm/data_conversion.rs
impl WasmGameStateSnapshot {
    /// Core状態からWASM安全構造への変換
    pub fn from_core_state(core_state: &CoreGameState) -> Self {
        // 安全な変換処理
    }
}
```

## 🔄 データフロー設計

### CLI版データフロー
```
Time Provider → CLI Runner → Core Logic → Render Data → Terminal Renderer
     ↑              ↓
Input Handler ← Core Logic ← Game State Update ← Pure Functions
```

### WASM版データフロー
```
JavaScript Timer → WASM Engine → Core Logic → WASM Result → JavaScript
       ↑                ↓
Canvas Renderer ← WASM Snapshot ← Game State Clone ← Pure Functions
```

### 共通Core Logicフロー
```
Input: Game State + Time + Action
  ↓
Pure Function Processing (no borrowing)
  ↓
Output: New Game State + Events + Render Data
```

## 🛡️ 安全性保証メカニズム

### 1. 借用チェッカー競合回避
```rust
// ❌ 危険なパターン (過去のインシデント原因)
impl GameState {
    fn update(&mut self) -> &Vec<Animation> {
        self.animations.update();
        &self.animations // 借用返却 → 競合リスク
    }
}

// ✅ 安全なパターン (新設計)
impl CoreGameState {
    fn update_with_time(self, time_ms: u64) -> GameStateUpdateResult {
        // self を consume
        let new_animations = process_animations(self.animations, time_ms);
        
        GameStateUpdateResult {
            new_state: CoreGameState {
                animations: new_animations,
                ..self // 他フィールドをムーブ
            },
            events: vec![], // 新規作成
        }
    }
}
```

### 2. WASM境界メモリ安全性
```rust
// ✅ 固定サイズ配列使用
type SafeBoard = [[Cell; BOARD_WIDTH]; BOARD_HEIGHT];

// ✅ 制限付き動的配列
const MAX_ANIMATIONS: usize = 8;
type SafeAnimations = ArrayVec<AnimationState, MAX_ANIMATIONS>;

// ✅ コピー可能データ構造
#[derive(Debug, Clone, Copy)]
struct WasmSafeData {
    // プリミティブ型のみ
}
```

### 3. エラー境界設計
```rust
// wasm/error_handling.rs
pub fn safe_wasm_call<T, F>(operation: F, fallback: T) -> T
where
    F: FnOnce() -> T + std::panic::UnwindSafe,
    T: Clone,
{
    match std::panic::catch_unwind(operation) {
        Ok(result) => result,
        Err(_) => {
            web_sys::console::error_1(&"WASM operation failed, using fallback".into());
            fallback
        }
    }
}

// 使用例
#[wasm_bindgen]
pub fn update(&mut self, time_ms: f64) -> WasmUpdateResult {
    safe_wasm_call(
        || self.update_internal(time_ms),
        WasmUpdateResult::empty(), // フォールバック
    )
}
```

## 🔧 実装移行戦略

### Phase 1: Core Logic抽出 (現在のCLI実装から)
```bash
# 1. 既存animation.rsから純粋関数を抽出
src/core/animation_logic.rs

# 2. 既存board_logic.rsから純粋関数を抽出  
src/core/board_logic.rs

# 3. 統合ゲーム状態構造を作成
src/core/game_state.rs

# 4. CLI版を新アーキテクチャにリファクタリング
src/cli/game_runner.rs
```

### Phase 2: WASM API実装
```bash
# 5. WASM境界データ構造
src/wasm/data_structures.rs

# 6. メインWASMエンジン
src/wasm/game_engine.rs

# 7. JavaScript統合テスト
tests/wasm_integration_tests.rs
```

### Phase 3: テストと検証
```bash
# 8. CLI-WASM動作同等性テスト
tests/cli_wasm_equivalence_tests.rs

# 9. 長時間実行テスト
tests/stability_tests.rs

# 10. メモリリークテスト
tests/memory_tests.rs
```

## 📋 実装チェックリスト

### Core Logic Layer
- [ ] `src/core/animation_logic.rs` 純粋関数実装
- [ ] `src/core/board_logic.rs` ボード処理関数
- [ ] `src/core/game_state.rs` 統合状態構造
- [ ] 既存CLI実装からの関数抽出
- [ ] Core Logic単体テスト

### CLI Integration Layer  
- [ ] `src/cli/game_runner.rs` 新アーキテクチャ対応
- [ ] 既存CLI機能の互換性確保
- [ ] CLI層での Core Logic使用
- [ ] CLI版動作テスト

### WASM API Layer
- [ ] `src/wasm/game_engine.rs` メインエンジン
- [ ] `src/wasm/data_structures.rs` 境界安全構造
- [ ] `src/wasm/error_handling.rs` エラー処理
- [ ] TypeScript型定義
- [ ] JavaScript統合テスト

### Quality Assurance
- [ ] CLI-WASM同等性テスト
- [ ] 長時間実行安定性テスト
- [ ] メモリ使用量測定
- [ ] パフォーマンステスト
- [ ] エラー境界テスト

## 🎯 成功指標

1. **機能同等性**: CLI版とWASM版で同じ入力に対して同じ出力
2. **安全性**: WASM関連panic/エラーの完全回避
3. **パフォーマンス**: 60FPS描画での安定動作
4. **保守性**: 共通ロジックの重複排除
5. **拡張性**: 新機能追加時の両版同時対応

---

**この統合アーキテクチャにより、CLI版の完成した機能を安全にWASM版に統合し、過去のインシデントを完全に回避した堅牢なゲームシステムを構築できます。**