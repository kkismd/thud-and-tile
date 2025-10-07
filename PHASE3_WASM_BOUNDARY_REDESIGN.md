# Phase 3: WASM境界設計の根本見直し

## 📊 設計概要

**設計日**: 2025年10月7日
**基準文書**: CLI_WASM_INTEGRATION_REDESIGN.md
**前提条件**: Phase 1（95%適合）、Phase 2（3層コンセプト確定）完了
**目標**: データコピー最優先原則による安全なWASM境界設計

---

## 🚨 **過去のWASMインシデント教訓の徹底適用**

### **インシデント回避策**
1. **借用チェッカー競合の完全回避**
   - 解決策: データコピーパターンの徹底
   - 実装: 全WASM API関数で値渡し・戻り値返却

2. **メモリアクセス違反の防止**
   - 解決策: 固定サイズ配列の使用
   - 実装: JavaScript側には単純なプリミティブ型のみ返却

3. **アーキテクチャ競合の解消**
   - 解決策: 3層分離による責任の明確化
   - 実装: Layer 3でWASM境界処理を完全分離

---

## 🎯 **Layer 3: WASM境界層の完全設計**

### **設計原則**

#### **1. データコピー最優先**
```rust
// ❌ 過去の問題パターン（借用チェッカーリスク）
#[wasm_bindgen]
impl WasmGame {
    pub fn update_animations(&mut self) -> JsValue {
        // Core Moduleの可変借用 → 競合リスク
        let result = self.core_state.update_animations();
        // JavaScript側での状態共有 → メモリ違反リスク
    }
}

// ✅ 新設計（データコピーパターン）
#[wasm_bindgen]
impl WasmGameEngine {
    pub fn update_with_time(&mut self, js_time_ms: f64) -> JsValue {
        let time_ms = js_time_ms as u64;
        
        // 1. Layer 1の純粋関数使用（借用なし）
        let updated_animations = crate::core::animation_logic::update_animation_states(
            &self.snapshot.animations,  // 読み取り専用借用のみ
            time_ms,
        );
        
        // 2. データコピーで状態更新
        self.snapshot.animations = updated_animations;
        
        // 3. JavaScript安全な戻り値（プリミティブのみ）
        self.create_render_info_js()  // データコピー返却
    }
}
```

#### **2. JavaScript時間管理への移行**
```rust
// ❌ 過去の問題（Rust側時間取得）
pub fn update_animations(&mut self) {
    let now = SystemTime::now();  // Rust側時間 → WASM境界問題
}

// ✅ 新設計（JavaScript時間管理）
#[wasm_bindgen]
pub fn update_with_time(&mut self, js_time_ms: f64) -> JsValue {
    // JavaScript側から時間を受け取り
    // Rust側では時間取得しない
}
```

#### **3. 固定サイズ・プリミティブ型インターフェース**
```rust
/// JavaScript安全な戻り値構造
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmRenderInfo {
    // 固定サイズ配列をJavaScript側に適した形式で返却
    pub score: u64,
    pub lines_cleared: u32,
    pub chain_bonus: u32,
    pub animation_count: usize,
}

#[wasm_bindgen]
impl WasmRenderInfo {
    /// ボード状態の安全な取得（行ごと）
    #[wasm_bindgen(getter)]
    pub fn get_board_row(&self, row: usize) -> Vec<u8> {
        // 固定サイズ配列を安全にVecでコピー返却
        if row >= BOARD_HEIGHT {
            return vec![0; BOARD_WIDTH];
        }
        self.board_data[row].to_vec()
    }
}
```

---

## 🔧 **具体的WASM API設計**

### **WasmGameEngine構造**
```rust
//! src/wasm/wasm_game_engine.rs

use wasm_bindgen::prelude::*;
use crate::core::{CoreGameState, AnimationState};

#[wasm_bindgen]
pub struct WasmGameEngine {
    // Layer 1のスナップショットをデータコピーで保持
    core_snapshot: CoreGameState,
    last_update_ms: u64,
    // JavaScript側エラー情報（シンプルな型のみ）
    last_error_code: u32,
}

#[wasm_bindgen]
impl WasmGameEngine {
    /// 安全なコンストラクタ
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGameEngine {
        WasmGameEngine {
            core_snapshot: CoreGameState::new_default(),
            last_update_ms: 0,
            last_error_code: 0,
        }
    }
    
    /// メインアニメーション更新（データコピーパターン）
    #[wasm_bindgen]
    pub fn update_with_time(&mut self, js_time_ms: f64) -> WasmRenderInfo {
        let time_ms = js_time_ms as u64;
        self.last_update_ms = time_ms;
        
        // Layer 1純粋関数使用（借用チェッカー安全）
        self.core_snapshot.animations = crate::core::animation_logic::update_all_animation_states(
            self.core_snapshot.animations.clone(),  // データコピー
            time_ms,
        ).updated_animations;
        
        // 安全なデータコピー返却
        self.create_render_info()
    }
    
    /// EraseLineアニメーション開始（安全なAPI）
    #[wasm_bindgen]
    pub fn start_erase_line_animation(&mut self) -> bool {
        // Layer 1純粋関数でSolidライン数取得
        let solid_lines = crate::core::erase_line_logic::count_solid_lines_from_bottom(
            self.core_snapshot.board
        );
        
        let erase_count = crate::core::erase_line_logic::determine_erase_line_count(
            self.core_snapshot.chain_bonus,
            solid_lines,
        );
        
        if erase_count > 0 {
            // データコピーでアニメーション追加
            let new_animation = crate::core::animation_logic::create_erase_line_animation(
                (0..erase_count).collect(),  // 底辺からerase_count行
                self.last_update_ms,
            );
            self.core_snapshot.animations.push(new_animation);
            true
        } else {
            false
        }
    }
    
    /// 入力処理（値渡しパターン）
    #[wasm_bindgen]
    pub fn handle_input(&mut self, input_code: u8) -> bool {
        // Layer 1純粋関数でゲーム状態更新
        match crate::core::input_handler::process_input(
            self.core_snapshot.clone(),  // データコピー
            input_code,
            self.last_update_ms,
        ) {
            Ok(new_state) => {
                self.core_snapshot = new_state;  // データコピー反映
                true
            }
            Err(_) => {
                self.last_error_code = 1;
                false
            }
        }
    }
    
    /// エラー状態取得
    #[wasm_bindgen]
    pub fn get_last_error(&self) -> u32 {
        self.last_error_code
    }
    
    /// 安全なレンダリング情報作成
    fn create_render_info(&self) -> WasmRenderInfo {
        // Layer 1純粋関数でボード状態計算
        let rendered_board = crate::core::animation_logic::apply_all_animations_to_board(
            self.core_snapshot.board,
            &self.core_snapshot.animations,
            self.last_update_ms,
        );
        
        WasmRenderInfo::from_core_state(&self.core_snapshot, rendered_board)
    }
}
```

### **JavaScript安全な戻り値型**
```rust
//! src/wasm/wasm_types.rs

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmRenderInfo {
    board_data: [[u8; BOARD_WIDTH]; BOARD_HEIGHT],  // 内部用
    score: u64,
    lines_cleared: u32,
    chain_bonus: u32,
    animation_count: usize,
    game_mode: u8,  // enum → u8変換
}

#[wasm_bindgen]
impl WasmRenderInfo {
    /// JavaScript側でのボード状態取得
    #[wasm_bindgen]
    pub fn get_board_data(&self) -> js_sys::Uint8Array {
        // 固定サイズ配列を安全にJavaScript配列として返却
        let flat_data: Vec<u8> = self.board_data
            .iter()
            .flat_map(|row| row.iter())
            .cloned()
            .collect();
        js_sys::Uint8Array::from(&flat_data[..])
    }
    
    /// 個別フィールド取得（プリミティブ型のみ）
    #[wasm_bindgen(getter)]
    pub fn score(&self) -> u64 { self.score }
    
    #[wasm_bindgen(getter)]
    pub fn lines_cleared(&self) -> u32 { self.lines_cleared }
    
    #[wasm_bindgen(getter)]
    pub fn chain_bonus(&self) -> u32 { self.chain_bonus }
    
    #[wasm_bindgen(getter)]
    pub fn animation_count(&self) -> usize { self.animation_count }
    
    #[wasm_bindgen(getter)]
    pub fn game_mode(&self) -> u8 { self.game_mode }
}

impl WasmRenderInfo {
    /// Core Moduleからの安全な変換
    pub fn from_core_state(core: &CoreGameState, rendered_board: FixedBoard) -> Self {
        let mut board_data = [[0u8; BOARD_WIDTH]; BOARD_HEIGHT];
        
        // セル状態をu8にマッピング（JavaScript安全）
        for (y, row) in rendered_board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                board_data[y][x] = match cell {
                    Cell::Empty => 0,
                    Cell::Occupied(color) => color.to_u8() + 1,
                    Cell::Connected(color) => color.to_u8() + 10,
                    Cell::Solid(color) => color.to_u8() + 20,
                };
            }
        }
        
        Self {
            board_data,
            score: core.score,
            lines_cleared: core.lines_cleared,
            chain_bonus: core.chain_bonus,
            animation_count: core.animations.len(),
            game_mode: core.game_mode as u8,
        }
    }
}
```

---

## 🧪 **安全性検証計画**

### **1. 借用チェッカー安全性テスト**
```rust
#[cfg(test)]
mod wasm_safety_tests {
    use super::*;
    
    #[test]
    fn test_no_borrow_checker_conflicts() {
        let mut engine = WasmGameEngine::new();
        
        // 並行呼び出しテスト（借用競合があれば失敗）
        let _result1 = engine.update_with_time(100.0);
        let _result2 = engine.handle_input(32);  // Spaceキー
        let _result3 = engine.get_render_info();
        
        // 全て成功すれば借用チェッカー安全
    }
}
```

### **2. メモリ安全性テスト**
```rust
#[test]
fn test_memory_safety() {
    let mut engine = WasmGameEngine::new();
    
    // 大量データ処理テスト
    for i in 0..1000 {
        let result = engine.update_with_time(i as f64);
        // JavaScript側戻り値が有効であることを確認
        assert!(result.score() >= 0);
    }
}
```

### **3. JavaScript統合テスト**
```typescript
// TypeScript側での安全性確認
describe('WASM境界安全性', () => {
    let engine: WasmGameEngine;
    
    beforeEach(() => {
        engine = new WasmGameEngine();
    });
    
    test('連続呼び出し安全性', () => {
        // 高頻度呼び出しでメモリ問題が発生しないことを確認
        for (let i = 0; i < 1000; i++) {
            const result = engine.update_with_time(Date.now());
            expect(result.score).toBeGreaterThanOrEqual(0);
        }
    });
});
```

---

## 📋 **Phase 3実装計画**

### **3.1: WASM境界基盤実装** (1日)
- [ ] `src/wasm/`ディレクトリ作成
- [ ] `WasmGameEngine`基本構造実装
- [ ] `WasmRenderInfo`安全型定義
- [ ] 基本的なデータコピーパターン実装

### **3.2: EraseLineアニメーション統合** (1日)
- [ ] `start_erase_line_animation` API実装
- [ ] Layer 1純粋関数との安全な統合
- [ ] アニメーション状態のWASM境界対応
- [ ] JavaScript時間管理統合

### **3.3: 安全性検証とテスト** (0.5日)
- [ ] 借用チェッカー安全性テスト
- [ ] メモリ安全性テスト  
- [ ] JavaScript統合テスト
- [ ] パフォーマンス検証

---

## 🎯 **Phase 3完了基準**

- ✅ データコピー最優先原則の完全実装
- ✅ 過去のWASMインシデント回避策の実装
- ✅ JavaScript安全なAPI境界の確立
- ✅ Layer 1-3の完全統合
- ✅ 借用チェッカー競合ゼロの確認
- ✅ Phase 4（統合計画再構築）への基盤完成

---

**Next**: Phase 4 - 統合計画再構築（WASM_CORE_INTEGRATION文書の全面改訂）