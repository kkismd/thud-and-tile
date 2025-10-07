# WASM統合マイグレーション: 包括的リスク管理マスター文書

**作成日**: 2025年10月7日  
**位置づけ**: WASM統合全体のリスク管理統括文書  
**分析範囲**: CLI Layer、Core Module、WASM境界、JavaScript統合、テスト構造、ビルド環境

**📋 専門文書との関係**:
- CLI移行詳細: `PHASE2_CLI_MIGRATION_STRATEGY.md`
- フェーズ2計画: `PHASE2_PLAN_REVISION.md`
- 技術仕様: `WASM_CORE_INTEGRATION_TECHNICAL.md`

---

## 🚨 **発見された主要リスク**

### **0. CLI移行リスク（フェーズ2特化）**
**詳細**: `PHASE2_CLI_MIGRATION_STRATEGY.md` 参照

#### **概要**
- **現状**: 92個のテスト全通過 → 一括変更で大量失敗リスク
- **対策**: 段階的移行（Strangler Fig Pattern）+ 後方互換API保持
- **ロールバック**: 個別機能レベル → フェーズレベル → 完全復元

### **1. WASMテスト構造の複雑性リスク**

#### **現状分析**
```bash
# 現在のテスト実行状況
cargo test --lib         # 92 tests passed (Core + CLI)
cargo test --features wasm-test  # 171 tests passed (全テスト)
```

#### **リスク要因**
- **複数テストモード**: native/wasm/nodejs-test の3つのテスト環境
- **feature依存**: `--features wasm-test` でのみ一部テスト実行
- **WASM専用テスト**: 171個中79個がWASM特化テスト → CLI移行時に影響

#### **失敗シナリオ**
```
Layer 2移行 → WASM feature依存テスト失敗 → 
統合テスト大量失敗 → デバッグ困難 → ロールバック
```

### **2. Feature Flag依存性リスク**

#### **現状のFeature構造**
```toml
[features]
default = ["native-bin"]
native-bin = []           # CLI専用
wasm = ["wasm-bindgen", "web-sys", "js-sys"]  # WASM専用
wasm-test = ["wasm", "wasm-bindgen-test"]     # WASMテスト
nodejs-test = ["wasm-bindgen", "js-sys"]      # Node.js環境
```

#### **リスク要因**
- **相互依存**: CLI移行時にfeature flagsの整合性が必要
- **ビルド複雑性**: 4つの異なるビルド構成（native/wasm/wasm-test/nodejs-test）
- **テスト分岐**: feature依存でテスト内容が変化

#### **失敗シナリオ**
```
Layer分離実装 → feature flags不整合 → 
ビルド失敗 → 統合テスト実行不可 → 開発停止
```

### **3. WASM境界APIの一括変更リスク**

#### **現在のWASM API構造**
```rust
// src/lib.rs: 1657行の大きなファイル
pub struct WasmGameState { ... }          // 380行目
impl WasmGameState { ... }                // 400行目〜992行目
// 合計600行以上のWASM実装
```

#### **リスク要因**
- **大きなファイル**: src/lib.rs が1657行（単一ファイルに集中）
- **密結合**: WasmGameState がCore Moduleと直接結合
- **API数**: 20個以上のwasm_bindgen関数 → 一括変更で大量影響

#### **失敗シナリオ**
```
Layer 3実装 → 20+ WASM API一括変更 → 
JavaScript側エラー大量発生 → Web版動作停止 → 統合失敗
```

### **4. JavaScript統合コードの脆弱性**

#### **Web版の現状**
```typescript
// thud-and-tile-web/ の依存
- TypeScript環境での複雑なWASM import
- Vite ビルド設定での WASM handling
- WASM package 依存: ../thud-and-tile/pkg/
```

#### **リスク要因**
- **ビルド依存**: Web版が Rust側 pkg/ ディレクトリに依存
- **型定義**: TypeScript型定義とRust型の不整合リスク
- **API契約**: JavaScript側でのWASM API呼び出し規約変更影響

#### **失敗シナリオ**
```
WASM API変更 → TypeScript型定義エラー → 
Web版ビルド失敗 → 統合テスト不可 → 全体開発停止
```

### **5. Core Module内部変更の波及リスク**

#### **現在のCore Module使用状況**
```rust
// CLI側使用
use crate::core::game_state::CoreGameState;

// WASM側使用（lib.rs内）
impl WasmGameState {
    // Core Moduleを直接操作
    board: Vec<Vec<Cell>>,
    // ... Core構造に依存
}
```

#### **リスク要因**
- **Layer 1調整**: Phase 1でのCore Module軽微調整がWASM側に波及
- **データ構造変更**: `Vec<AnimationState>` → 固定サイズ配列変更の影響
- **API整合性**: Core ModuleのAPI変更がCLI/WASM両方に影響

#### **失敗シナリオ**
```
Layer 1軽微調整 → WASM API型不整合 → 
コンパイルエラー → テスト大量失敗 → Layer 1調整ロールバック
```

---

## 📊 **リスク影響度評価**

| リスク要因 | 影響度 | 発生確率 | 総合リスク | 対策優先度 |
|------------|--------|----------|------------|------------|
| **WASMテスト構造複雑性** | 🔴 高 | 🟡 中 | 🔴 高 | 1位 |
| **Feature Flag依存性** | 🟡 中 | 🔴 高 | 🔴 高 | 2位 |
| **WASM境界API一括変更** | 🔴 高 | 🟡 中 | 🔴 高 | 3位 |
| **JavaScript統合脆弱性** | 🟡 中 | 🟡 中 | 🟡 中 | 4位 |
| **Core Module波及** | 🟡 中 | 🟡 中 | 🟡 中 | 5位 |

---

## 🛡️ **リスク対策戦略**

### **対策 1: WASMテスト保護戦略**

#### **段階的テスト移行**
```bash
# Phase 2.1: CLI基盤作成時
cargo test --lib                    # 92 tests → 維持必須
cargo test --features wasm-test     # 171 tests → 影響監視

# Phase 2.2: 各CLI機能移行時  
function verify_wasm_tests() {
    cargo test --features wasm-test || {
        echo "WASMテスト失敗 - 移行ロールバック"
        return 1
    }
}
```

#### **テスト分離保護**
```rust
// WASMテストを独立実行可能に
#[cfg(all(feature = "wasm-test", target_arch = "wasm32"))]
mod wasm_integration_tests {
    // WASM専用テストを分離保護
}
```

### **対策 2: Feature Flag安定化**

#### **段階的Feature移行**
```toml
# Phase 2での新しいfeature構成
[features]
default = ["native-bin"]
native-bin = []                    # CLI Layer（既存保持）
cli-layer = ["native-bin"]         # 新CLI Layer 
wasm = ["wasm-bindgen", "web-sys", "js-sys"]  # WASM Layer（既存保持）
wasm-layer = ["wasm"]              # 新WASM Layer
unified-test = ["cli-layer", "wasm-layer"]  # 統合テスト
```

#### **Feature互換性検証**
```bash
# 全Feature組み合わせテスト
for feature in native-bin wasm wasm-test nodejs-test; do
    cargo test --features $feature || exit 1
done
```

### **対策 3: WASM APIの段階的移行**

#### **後方互換API保持**
```rust
// Phase 3での段階移行戦略
#[wasm_bindgen]
impl WasmGameState {
    // 既存API保持（内部実装のみLayer 3化）
    #[wasm_bindgen]
    pub fn update_with_input(&mut self, input_code: u8) -> WasmRenderInfo {
        // 内部でLayer 3呼び出し
        wasm::engine::process_input(input_code)
    }
}

// 新Layer 3 API（段階導入）
pub mod wasm {
    pub mod engine {
        pub fn process_input(input_code: u8) -> WasmRenderInfo {
            // Layer 3実装
        }
    }
}
```

#### **API変更影響最小化**
```rust
// 型定義の後方互換性
#[wasm_bindgen]
pub struct WasmRenderInfo {
    // 既存フィールド保持
    score: u32,
    // 新フィールド段階追加
    #[wasm_bindgen(getter)]
    pub fn get_chain_bonus(&self) -> u32 { ... }
}
```

### **対策 4: JavaScript統合保護**

#### **TypeScript型定義の保護**
```typescript
// Phase 3実装中の型安全性確保
interface WasmApiCompat {
    // 既存API型定義保持
    update_with_input(input: number): WasmRenderInfo;
    // 新API段階的追加
    update_with_layer3_input?(input: number): WasmRenderInfo;
}
```

#### **Web版ビルド保護**
```bash
# Phase 3での統合テスト
function test_web_integration() {
    # WASM package生成
    wasm-pack build --target web --features wasm
    
    # Web版ビルドテスト
    cd ../thud-and-tile-web
    npm run build || {
        echo "Web版ビルド失敗 - WASM API変更ロールバック"
        return 1
    }
}
```

### **対策 5: Core Module変更の影響制御**

#### **Layer 1変更の段階的適用**
```rust
// Phase 1での軽微調整戦略
#[cfg(feature = "layer1-migration")]
pub struct CoreGameState {
    // 新設計（固定サイズ配列）
    pub animations: [Option<AnimationState>; MAX_ANIMATIONS],
}

#[cfg(not(feature = "layer1-migration"))]
pub struct CoreGameState {
    // 既存設計保持（互換性確保）
    pub animations: Vec<AnimationState>,
}
```

#### **WASM境界での変換レイヤー**
```rust
// Core Module変更の影響吸収
impl WasmGameState {
    fn convert_core_state(&self, core: &CoreGameState) -> WasmRenderInfo {
        // Core変更の影響をWASM境界で吸収
        WasmRenderInfo {
            // 型変換で互換性確保
        }
    }
}
```

---

## 📋 **フェーズ別リスク対策統合**

### **Phase 1補強: Core Module変更保護**
- Core Module変更時のWASM API影響テスト追加
- 型変換レイヤーでの互換性確保
- feature flag切り替えでの段階移行

### **Phase 2補強: CLI移行時のWASMテスト保護**
- CLI移行各段階でのWASMテスト実行確認
- Feature flag整合性検証の自動化
- WASMテスト失敗時の個別ロールバック

### **Phase 3補強: WASM境界変更の段階実行**
- WASM API後方互換性確保
- JavaScript型定義との整合性確認
- Web版ビルドテストの統合

### **Phase 4補強: 統合テスト時の全環境検証**
- 全Feature組み合わせでの統合テスト
- CLI/WASM両環境での機能等価性確認
- Web版を含む完全統合テスト

---

## ⚡ **緊急時ロールバック拡張**

### **レベル別ロールバック戦略**
```
レベル 1: 個別機能ロールバック
├── CLI機能: git checkout -- src/cli/component.rs
├── WASM API: git checkout -- src/lib.rs (特定関数)
└── Feature: feature flag切り替え

レベル 2: Layer別ロールバック  
├── Layer 1: git checkout -- src/core/ 
├── Layer 2: git checkout -- src/cli/
└── Layer 3: git checkout -- src/lib.rs

レベル 3: Phase別ロールバック
├── Phase 1: git reset --hard phase1-complete
├── Phase 2: git reset --hard phase2-complete
└── Phase 3: git reset --hard phase3-complete

レベル 4: 完全ロールバック
└── git reset --hard main
```

### **環境別復旧戦略**
```bash
# CLI環境復旧
function restore_cli() {
    cargo test --lib || git checkout -- src/main.rs src/cli/
}

# WASM環境復旧  
function restore_wasm() {
    cargo test --features wasm-test || git checkout -- src/lib.rs
}

# Web版統合復旧
function restore_web() {
    cd ../thud-and-tile-web && npm run build || {
        cd ../thud-and-tile
        git checkout -- src/lib.rs pkg/
    }
}
```

---

## 🎯 **成功指標拡張**

### **CLI以外の成功条件**
1. **WASMテスト**: 171/171 passed 維持
2. **Feature互換**: 全feature組み合わせビルド成功  
3. **WASM API**: JavaScript型定義との完全整合性
4. **Web版統合**: thud-and-tile-web正常ビルド・動作
5. **環境統合**: CLI/WASM完全機能等価性

### **リスク指標**
- ❌ WASMテスト失敗率: 0% 維持必須
- ❌ Feature依存ビルド失敗: 0件許容
- ❌ JavaScript型不整合: 0件許容  
- ❌ Web版ビルド失敗: 0件許容
- ❌ 環境間機能差異: 0件許容

**結論**: CLI移行と並行してWASM環境の複雑性リスクを段階的に管理し、全環境での安定性を確保。