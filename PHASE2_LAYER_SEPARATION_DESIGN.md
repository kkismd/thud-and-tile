# Phase 2: Layer分離アーキテクチャ設計

## 📊 設計概要

**設計日**: 2024年12月
**基準文書**: CLI_WASM_INTEGRATION_REDESIGN.md 3-layer設計
**Phase 1結果**: Core Module 95%適合（Layer 1として活用可能）

---

## 🎯 3-Layer分離アーキテクチャ

### **現在の構造** vs **提案構造**

```
[現在: 2層構造]
src/core/ (共通モジュール)
├── game_state.rs
├── erase_line_logic.rs
├── animation_logic.rs
└── board_logic.rs

src/ (CLI専用 + WASM API混在)
├── lib.rs (WASM API)
├── main.rs (CLI)
├── render.rs (CLI)
└── その他

[提案: 3層構造]
Layer 1: 共通コアロジック (src/core/)
├── 純粋関数群
├── 固定サイズデータ構造
└── 借用チェッカー安全

Layer 2: CLI専用レイヤー (src/cli/)
├── CLI特化機能
├── Rust native処理
└── 既存実装活用

Layer 3: WASM専用レイヤー (src/wasm/)
├── JavaScript安全API
├── データコピー徹底
└── 時間管理分離
```

---

## 🔧 Layer 1: 共通コアロジック（✅ 既存活用）

### **利用方針**
Phase 1で95%適合確認済みの既存Core Moduleを基盤として活用

### **既存モジュール活用**
- ✅ `src/core/game_state.rs` → `CoreGameState`
- ✅ `src/core/erase_line_logic.rs` → 純粋関数群
- ✅ `src/core/animation_logic.rs` → `AnimationState`処理
- ✅ `src/core/board_logic.rs` → `FixedBoard`操作

### **軽微調整項目**
```rust
// 現在
pub struct AnimationState {
    pub lines: Vec<usize>,  // ⚠ 動的配列
}

// 推奨調整
pub struct AnimationState {
    pub lines: [Option<usize>; 4],  // 固定サイズ化
}
```

---

## 🔧 Layer 2: CLI専用レイヤー（新規設計）

### **設計方針**
- 既存CLI実装を最大限活用
- Layer 1の純粋関数を組み合わせ
- Rust nativeの利点活用

### **新規モジュール構造**
```
src/cli/
├── mod.rs                    # CLI層エクスポート
├── cli_game_state.rs         # CLI版ゲーム状態
├── cli_animation.rs          # CLI版アニメーション処理
├── cli_input_handler.rs      # CLI版入力処理
└── cli_renderer.rs           # CLI版描画処理
```

### **設計例: cli_game_state.rs**
```rust
//! CLI専用ゲーム状態管理
//! Layer 1の共通ロジックを使用したCLI特化実装

use crate::core::{CoreGameState, AnimationState};
use crate::render::Renderer;
use std::time::Instant;

/// CLI版ゲーム状態ラッパー
pub struct CliGameState {
    pub core: CoreGameState,           // Layer 1活用
    pub time_provider: TimeProvider,   // CLI特化時間管理
    pub renderer_state: RendererState, // CLI特化描画状態
}

impl CliGameState {
    /// CLI版アニメーション更新
    pub fn update_animations(&mut self) {
        let current_time_ms = self.time_provider.now_ms();
        
        // Layer 1純粋関数使用
        self.core.animations = crate::core::animation_logic::update_animation_states(
            &self.core.animations,
            current_time_ms,
        );
        
        // CLI特化後処理
        self.handle_cli_specific_updates();
    }
}
```

---

## 🔧 Layer 3: WASM専用レイヤー（新規設計）

### **設計方針**
- JavaScript安全なAPI設計
- データコピーパターン徹底
- 借用チェッカー競合完全回避

### **新規モジュール構造**
```
src/wasm/
├── mod.rs                    # WASM層エクスポート
├── wasm_game_engine.rs       # WASM APIエンジン
├── wasm_types.rs             # JavaScript互換型定義
├── wasm_animation.rs         # WASM版アニメーション
└── wasm_bridge.rs            # Layer 1→WASM変換
```

### **設計例: wasm_game_engine.rs**
```rust
//! WASM境界専用ゲームエンジン
//! JavaScript安全なインターフェース

use wasm_bindgen::prelude::*;
use crate::core::CoreGameState;

#[wasm_bindgen]
pub struct WasmGameEngine {
    core_snapshot: CoreGameState,  // データコピー保持
    last_update_ms: u64,
}

#[wasm_bindgen]
impl WasmGameEngine {
    /// JavaScript時間管理でアニメーション更新
    #[wasm_bindgen]
    pub fn update_with_time(&mut self, js_time_ms: f64) -> JsValue {
        let time_ms = js_time_ms as u64;
        
        // Layer 1純粋関数使用（借用チェッカー安全）
        self.core_snapshot.animations = crate::core::animation_logic::update_animation_states(
            &self.core_snapshot.animations,
            time_ms,
        );
        
        // データコピーでJavaScript返却
        self.create_render_info_js()
    }
    
    /// EraseLineアニメーション開始
    #[wasm_bindgen]
    pub fn start_erase_line_animation(&mut self) -> bool {
        // Layer 1純粋関数使用
        let solid_lines = crate::core::erase_line_logic::count_solid_lines_from_bottom(
            self.core_snapshot.board
        );
        
        let erase_count = crate::core::erase_line_logic::determine_erase_line_count(
            self.core_snapshot.chain_bonus,
            solid_lines,
        );
        
        // 安全なデータコピー更新
        if erase_count > 0 {
            self.create_erase_line_animation(erase_count);
            true
        } else {
            false
        }
    }
}
```

---

## 📋 Layer分離実装計画

### **Phase 2.1: Layer 1軽微調整** (0.5日)
- [ ] `AnimationState.lines`の固定サイズ化
- [ ] Core Module微調整とテスト更新

### **Phase 2.2: Layer 2 CLI専用設計** (1日)
- [ ] `src/cli/`ディレクトリ作成
- [ ] CLI専用モジュール設計
- [ ] 既存CLI実装の移行計画

### **Phase 2.3: Layer 3 WASM専用設計** (1日)
- [ ] `src/wasm/`ディレクトリ作成
- [ ] WASM APIエンジン基盤設計
- [ ] JavaScript境界安全性確保

### **Phase 2.4: 統合テスト設計** (0.5日)
- [ ] 3-layer統合テスト計画
- [ ] Layer間インターフェース検証
- [ ] 既存テストの移行計画

---

## 🎯 **Phase 2完了基準**

- ✅ 3-layer分離設計の完成
- ✅ 各Layerの責務明確化
- ✅ Phase 3（WASM境界実装）への基盤確立
- ✅ 既存機能への影響最小化

---

**Next**: Phase 3 - WASM境界再設計（Layer 3詳細実装）