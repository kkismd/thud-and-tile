# Phase 2 CLI移行戦略: 安全な段階的移行計画

**作成日**: 2025年10月7日  
**特化対象**: フェーズ 2「Layer 2 CLI実装」専門戦略  
**目的**: 既存CLI機能の安全な移行とテスト保護

**📋 関連文書**: 
- 包括的リスク分析: `MIGRATION_RISK_COMPREHENSIVE_ANALYSIS.md`
- フェーズ2計画改訂: `PHASE2_PLAN_REVISION.md`

---

## 🚨 **過去のインシデント分析**

### **典型的な失敗パターン**
1. **新API作成 → 既存実装一括置換 → テスト大量失敗 → ロールバック**
2. **根本原因**: 既存の依存関係とテストケースを考慮せずに一括変更
3. **影響範囲**: 現在92個のテストが全て通過中 → 一括変更で大量失敗のリスク

---

## 🎯 **CLI移行戦略の特化範囲**

### **対象範囲（CLI Layer専門）**
```rust
main.rs (エントリーポイント)
├── crossterm (terminal制御) ← 移行対象
├── render.rs (描画システム) ← 移行対象  
├── game_input.rs (入力処理) ← 移行対象
├── scheduler.rs (時間管理) ← 移行対象
├── cli_bridge.rs (CLIブリッジ) ← 移行対象
└── core/ (共通ロジック) ← Phase 1完了、Layer 1適合済み
```

### **対象外（他文書で管理）**
- **WASM境界リスク** → `MIGRATION_RISK_COMPREHENSIVE_ANALYSIS.md`
- **JavaScript統合** → `MIGRATION_RISK_COMPREHENSIVE_ANALYSIS.md`  
- **Feature Flag管理** → `MIGRATION_RISK_COMPREHENSIVE_ANALYSIS.md`
- **Core Module変更** → Phase 1完了済み

---

## 🛡️ **リスク対応戦略**

### **戦略 1: 段階的移行（Strangler Fig Pattern）**
新しいLayer 2を既存CLIと**並行稼働**させ、段階的に置換

```
フェーズ 2A: Layer 2基盤作成（リスクなし）
├── src/cli/ ディレクトリ作成
├── Layer 2基本構造実装
└── 既存CLI機能は完全保持（変更なし）

フェーズ 2B: 機能別段階移行（低リスク）
├── 1機能ずつ移行
├── 各移行後に全テスト実行
└── 失敗時は該当機能のみロールバック

フェーズ 2C: 最終統合（確認済み）
├── 全機能移行完了後の統合
└── 旧実装削除
```

### **戦略 2: テスト駆動移行（Test-First Migration）**
移行前に新Layer 2用のテストを作成し、既存テストとの整合性確保

```
手順:
1. 移行対象機能の既存テストを複製
2. Layer 2 API向けに調整
3. Layer 2実装 → 新テスト通過確認
4. 既存実装を段階的に新実装に置換
5. 全テスト（旧+新）通過確認
```

### **戦略 3: 後方互換API（Backward Compatibility）**
既存APIを維持しつつ、内部実装のみLayer 2に移行

```rust
// 既存API保持（テスト変更なし）
pub fn render_game_state(state: &GameState) {
    // 内部でLayer 2呼び出し
    cli::renderer::render(state);
}

// 新Layer 2 API（段階的導入）
pub mod cli {
    pub mod renderer {
        pub fn render(state: &GameState) {
            // Layer 2実装
        }
    }
}
```

---

## 📋 **詳細移行計画**

### **フェーズ 2A: 安全な基盤構築（0.3日）**
```
目的: 既存機能に影響を与えずにLayer 2基盤を準備

実装:
1. src/cli/ ディレクトリ作成
2. CLI Layer基本構造定義
3. Layer 1との接続インターフェース作成

リスク: ゼロ（既存コードに変更なし）
検証: cargo test → 92/92 passed 維持
```

### **フェーズ 2B-1: 描画機能移行（0.2日）**
```
対象: render.rs の描画機能
理由: 他機能への依存が少ない独立性の高い機能

手順:
1. src/cli/renderer.rs 作成
2. render.rs の機能をLayer 2に実装
3. 後方互換APIで既存呼び出しを維持
4. テスト実行確認 (92/92 passed)

ロールバック: render.rs復元のみ（影響局所化）
```

### **フェーズ 2B-2: 入力機能移行（0.2日）**
```
対象: game_input.rs の入力処理
前提: フェーズ 2B-1 完了確認済み

手順:
1. src/cli/input.rs 作成
2. 既存InputProviderをLayer 2で再実装
3. 後方互換APIで既存呼び出しを維持
4. テスト実行確認

ロールバック: game_input.rs復元
```

### **フェーズ 2B-3: スケジューリング機能移行（0.2日）**
```
対象: scheduler.rs の時間管理
前提: フェーズ 2B-1,2 完了確認済み

手順:
1. src/cli/scheduler.rs 作成
2. 時間管理ロジックをLayer 2で再実装
3. 後方互換APIで維持
4. テスト実行確認

ロールバック: scheduler.rs復元
```

### **フェーズ 2C: 統合と最適化（0.1日）**
```
前提: 全コンポーネント移行完了、92テスト通過確認済み

手順:
1. main.rs でLayer 2 APIを直接呼び出しに変更
2. 旧ファイル削除
3. 最終テスト実行確認

ロールバック: gitによる全変更取り消し
```

---

## 🧪 **テスト保護戦略**

### **移行前テスト確保**
```bash
# 移行前ベースライン確保
cargo test > baseline_test_results.txt
git tag phase2-migration-start

# 各移行ステップでの検証
function verify_migration_step() {
    cargo test || {
        echo "テスト失敗 - ロールバック実行"
        git reset --hard HEAD~1
        return 1
    }
}
```

### **並行テスト実行**
```rust
// 旧実装テスト保持
#[cfg(test)]
mod legacy_tests {
    // 既存の77個のCLIテスト
}

// 新Layer 2テスト段階追加
#[cfg(test)]
mod layer2_tests {
    // Layer 2実装用の新テスト
}
```

### **回帰テスト自動化**
```bash
# 各機能移行後の自動検証
for component in renderer input scheduler; do
    echo "移行中: $component"
    # 移行実装
    cargo test || {
        echo "失敗: $component移行をロールバック"
        git checkout -- src/$component.rs
        continue
    }
    echo "成功: $component移行完了"
done
```

---

## ⚡ **緊急ロールバック計画**

### **段階別ロールバック**
```
レベル 1: 個別機能ロールバック
├── 対象: 単一コンポーネント移行失敗
├── 方法: git checkout -- src/cli/component.rs
└── 影響: 該当機能のみ

レベル 2: フェーズ全体ロールバック  
├── 対象: フェーズ内複数失敗
├── 方法: git reset --hard phase2-migration-start
└── 影響: フェーズ 2全体

レベル 3: 完全ロールバック
├── 対象: 予期せぬシステム全体影響
├── 方法: git reset --hard origin/main
└── 影響: 全変更取り消し
```

---

## 📊 **成功指標と検証**

### **移行完了条件**
1. ✅ **全テスト通過**: 92/92 passed 維持
2. ✅ **機能等価性**: CLI版のユーザビリティ完全保持
3. ✅ **コード品質**: Layer分離による依存関係整理
4. ✅ **保守性向上**: Layer 2独立モジュール化

### **品質メトリクス**
```rust
// テスト通過率
assert_eq!(test_pass_rate, 100%);

// Layer分離品質
assert!(layer1_dependencies.is_empty_of_cli_specific());
assert!(layer2_dependencies.is_empty_of_wasm_specific());

// 後方互換性
assert_eq!(cli_user_experience, original_experience);
```

---

## 🎯 **実装順序最適化**

### **依存関係最小化順序**
1. **renderer** (独立性高) → リスク最小
2. **input** (renderer依存なし) → リスク低
3. **scheduler** (全体調整必要) → リスク中
4. **統合** (全依存関係調整) → リスク管理済み

### **各ステップでの検証ポイント**
```
ステップ 1完了 → 描画テスト確認
ステップ 2完了 → 入力テスト確認
ステップ 3完了 → 統合テスト確認
最終完了 → 全機能テスト確認
```

**結論**: 段階的移行 + テスト駆動 + 後方互換性により、テスト失敗リスクを最小化したフェーズ 2実装計画を確立。