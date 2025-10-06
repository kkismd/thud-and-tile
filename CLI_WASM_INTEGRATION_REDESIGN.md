# CLI-WASM統合再設計書

**日時:** 2025年10月6日  
**背景:** CLI版メカニクス完成後のWASM統合再挑戦  
**目的:** 過去のWASMインシデント知見を活かした安全なAPI設計

## 📋 現状分析

### ✅ CLI版で完成した機能
1. **LineBlink アニメーション**
   - TDD Cycle 1-4 完了 (18/18 テスト成功)
   - ヘルパー関数: `is_line_visible`, `animation_affects_line`, `is_animation_completed`, `should_render_cell`
   - 完全なCLI描画統合

2. **PushDown アニメーション**
   - 基本構造とボード更新ロジック実装済み
   - 段階的押し下げとgray_line_y管理

3. **EraseLine アニメーション**
   - 段階的ライン消去とCHAIN-BONUS連携
   - 時間ベース処理とステップ管理

4. **統合アニメーション処理**
   - `update_animations` 関数によるマルチアニメーション管理
   - CLI版での完全な統合確認済み

### 🚨 過去のWASMインシデント分析

#### インシデント1: 借用チェッカー競合
- **問題**: `update_animation_with_time` での再帰借用エラー
- **原因**: JavaScript → Rust → JavaScript 間での状態共有
- **教訓**: 借用分離とデータコピーパターンの必要性

#### インシデント2: メモリアクセス違反
- **問題**: `unreachable` WASM実行時エラー
- **原因**: 複雑なアニメーション状態の境界越えアクセス
- **教訓**: シンプルな値渡しAPIの重要性

#### インシデント3: アーキテクチャ競合
- **問題**: Rust借用チェッカーとWASMバインディングの根本的不整合
- **原因**: 多層アーキテクチャでの状態管理複雑性
- **教訓**: 明確な責任分離の必要性

## 🎯 新設計コンセプト

### 核心原則
1. **データコピー最優先**: 借用チェッカー競合を完全回避
2. **責任の明確分離**: Rust側とJavaScript側の役割分担
3. **段階的統合**: 機能別の安全な統合手順
4. **検証可能性**: 各段階での完全テスト

## 🏗️ 統合アーキテクチャ設計

### Layer 1: 共通コアロジック (core_logic.rs)
```rust
//! WASM境界を越えない純粋なRustロジック
//! CLI版とWASM版の共通基盤

/// アニメーション状態のスナップショット (Copy/Clone可能)
#[derive(Debug, Clone, Copy)]
pub struct AnimationState {
    pub animation_type: AnimationType,
    pub lines: [Option<usize>; 4], // 固定サイズでヒープ回避
    pub current_step: usize,
    pub elapsed_ms: u64,
    pub is_completed: bool,
}

/// ゲーム状態のスナップショット (WASM安全)
#[derive(Debug, Clone)]
pub struct GameStateSnapshot {
    pub board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT], // 固定サイズ配列
    pub current_board_height: usize,
    pub animations: Vec<AnimationState>, // 単純な状態のみ
    pub score: u64,
    pub lines_cleared: u32,
}

/// 核心ロジック関数群 (借用チェッカー競合なし)
pub fn update_animation_states(
    animations: &[AnimationState],
    current_time_ms: u64,
) -> Vec<AnimationState> {
    // 純粋関数: 入力コピー → 処理 → 新しい状態コピー返却
}

pub fn apply_animation_to_board(
    board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
    animation: AnimationState,
    current_time_ms: u64,
) -> [[Cell; BOARD_WIDTH]; BOARD_HEIGHT] {
    // 純粋関数: ボードコピー → アニメーション適用 → 新ボード返却
}

pub fn calculate_line_clears(
    board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
    board_height: usize,
) -> Vec<usize> {
    // 純粋関数: ボード分析 → クリア対象ライン検出
}
```

### Layer 2: CLI専用レイヤー (cli_integration.rs)
```rust
//! CLI版での統合処理
//! 共通コアロジックを使用したCLI特化機能

use crate::core_logic::*;

/// CLI版ゲーム状態 (Rust native)
pub struct CliGameState {
    pub snapshot: GameStateSnapshot,
    pub time_provider: Box<dyn TimeProvider>,
    // CLI特化のフィールド
}

impl CliGameState {
    /// CLI版でのアニメーション処理 (既存実装維持)
    pub fn update_animations(&mut self) {
        let current_time_ms = self.time_provider.now().as_millis() as u64;
        
        // 共通ロジック使用
        self.snapshot.animations = update_animation_states(
            &self.snapshot.animations,
            current_time_ms,
        );
        
        // CLI特化処理
        self.handle_cli_specific_updates();
    }
    
    /// CLI描画処理 (既存実装活用)
    pub fn render(&self, renderer: &mut dyn Renderer) {
        // 共通ロジックでボード状態計算
        let rendered_board = self.calculate_rendered_board();
        
        // CLI特化描画
        renderer.render_board(&rendered_board);
    }
}
```

### Layer 3: WASM APIレイヤー (wasm_api.rs)
```rust
//! WASM境界専用API
//! JavaScript安全なインターフェース

use wasm_bindgen::prelude::*;
use crate::core_logic::*;

#[wasm_bindgen]
pub struct WasmGameEngine {
    snapshot: GameStateSnapshot,
    last_update_ms: u64,
}

#[wasm_bindgen]
impl WasmGameEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGameEngine {
        WasmGameEngine {
            snapshot: GameStateSnapshot::new(),
            last_update_ms: 0,
        }
    }
    
    /// JavaScript時間管理によるアニメーション更新
    #[wasm_bindgen]
    pub fn update_with_time(&mut self, current_time_ms: f64) -> JsValue {
        let time_ms = current_time_ms as u64;
        
        // 共通ロジック使用 (借用チェッカー安全)
        self.snapshot.animations = update_animation_states(
            &self.snapshot.animations,
            time_ms,
        );
        
        self.last_update_ms = time_ms;
        
        // JavaScript側返却用のシンプルな状態
        serde_wasm_bindgen::to_value(&self.get_render_info()).unwrap()
    }
    
    /// レンダリング情報の取得 (データコピー)
    #[wasm_bindgen]
    pub fn get_render_info(&self) -> RenderInfo {
        // 共通ロジックでレンダリング状態計算
        let rendered_board = apply_all_animations_to_board(
            self.snapshot.board,
            &self.snapshot.animations,
            self.last_update_ms,
        );
        
        RenderInfo {
            board: rendered_board,
            score: self.snapshot.score,
            lines_cleared: self.snapshot.lines_cleared,
        }
    }
    
    /// ユーザー入力処理 (値渡し)
    #[wasm_bindgen]
    pub fn handle_input(&mut self, input_type: u32, input_data: JsValue) -> bool {
        // 共通ロジックでゲーム状態更新
        // 戻り値: 更新成功/失敗
    }
}

/// JavaScript側へのシンプルなデータ構造
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct RenderInfo {
    board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
    score: u64,
    lines_cleared: u32,
}
```

## 🔄 段階的統合プラン

### Phase 1: 共通コアロジック抽出 (2-3日)
1. **core_logic.rs作成**
   - 既存CLI実装から純粋関数抽出
   - 借用チェッカー競合ゼロの設計
   - 包括的単体テスト

2. **CLI版リファクタリング**
   - 共通ロジック使用への移行
   - 既存テスト互換性確保
   - パフォーマンス検証

### Phase 2: WASM APIレイヤー実装 (3-4日)
1. **基本WASM構造作成**
   - WasmGameEngine骨組み
   - JavaScript時間管理インターフェース
   - シンプルな状態取得API

2. **段階的機能追加**
   - LineBlink統合 (最も安全)
   - PushDown統合
   - EraseLine統合 (最も複雑)

### Phase 3: Web統合とテスト (2-3日)
1. **TypeScript統合**
   - アニメーション描画ループ
   - WASM API呼び出し実装
   - エラーハンドリング強化

2. **包括的テスト**
   - CLI版とWASM版の動作同等性検証
   - パフォーマンステスト
   - 長時間実行安定性テスト

## 🛡️ 安全性確保戦略

### 借用チェッカー対策
1. **データコピーパターン**: すべてのWASM境界でコピー渡し
2. **固定サイズ配列**: 動的メモリ確保を最小化
3. **純粋関数設計**: 副作用のない関数による状態変更

### WASM境界安全性
1. **シンプルなABI**: 複雑な構造体をWASM境界で回避
2. **JavaScript時間管理**: Rust側での時間取得を廃止
3. **明確なエラー処理**: WASM panicsの完全防止

### テスト戦略
1. **core_logic単体テスト**: 共通ロジックの完全検証
2. **CLI-WASM同等性テスト**: 同じ入力での同じ出力保証
3. **境界値ストレステスト**: WASM境界での異常処理検証

## 🎯 実装優先順位

### 最高優先 (今週)
1. **共通コアロジック設計**: 借用チェッカー競合ゼロの純粋関数
2. **CLI版リファクタリング**: 既存機能の共通ロジック化

### 高優先 (来週)
1. **基本WASM API**: LineBlink統合まで
2. **TypeScript統合**: 基本的なWeb描画

### 中優先 (翌週)
1. **全アニメーション統合**: PushDown, EraseLine
2. **パフォーマンス最適化**: 描画頻度とメモリ使用量

## 📋 次のアクション

1. **core_logic.rs作成開始**: 純粋関数アニメーションロジック
2. **CLI版テスト実行**: 現状の安定性確認
3. **段階的リファクタリング**: 共通化可能な部分の特定

---

**この設計により、過去のWASMインシデントを回避しつつ、CLI版で完成した機能を安全にWeb版に統合できます。**