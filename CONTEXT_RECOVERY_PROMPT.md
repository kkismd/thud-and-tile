# プロジェクト状況回復用プロンプト

**作成日**: 2025年10月7日  
**対象**: thud-and-tile WASM統合プロジェクト  
**用途**: AIエージェントのコンテキストロスト時の状況回復

---

## 🚨 **緊急時回復用プロンプト**

```
私はthud-and-tileテトリスゲームのWASM統合プロジェクトに取り組んでいます。
AIエージェントとしてのコンテキストが失われたため、現在の状況を回復してください。

### 📍 現在の正確な状況
- **プロジェクト**: Rustで実装されたテトリスゲームのCLI→WASM統合
- **現在地**: /Users/kshimada/Documents/move_folder/src/Rust/thud-and-tile/
- **実装段階**: Phase 1実装中（Vec to fixed-array変換）
- **最新コミット**: phase1-implementation-start タグ付き
- **設計方針**: CLI_WASM_INTEGRATION_REDESIGN.md の3層アーキテクチャ

### 🎯 **Phase 1の具体的タスク**
1. AnimationState.lines: Vec<usize> → [usize; 4] + lines_count
2. CoreGameState.animations: Vec<AnimationState> → [AnimationState; 20] + animations_count

### 📊 **確認すべき状況**
1. `cargo test --lib` でテスト状況確認（目標: 92/92 passed）
2. `git status` で現在の変更状況確認
3. src/core/animation_logic.rs の AnimationState 構造体確認
4. src/core/game_state.rs の CoreGameState 構造体確認

### 📖 **重要文書**
- IMPLEMENTATION_ROADMAP.md: 実装手順詳細
- PHASE1_CORE_MODULE_COMPATIBILITY.md: Phase 1設計仕様
- CLI_WASM_INTEGRATION_REDESIGN.md: 全体アーキテクチャ

次に何をすべきか教えてください。
```

---

## 🔧 **技術的回復情報**

### **プロジェクト構造**
```
thud-and-tile/
├── src/core/              # Layer 1: WASM互換コア
│   ├── animation_logic.rs # ← 変更対象
│   └── game_state.rs     # ← 変更対象
├── src/                  # Layer 2: CLI実装
└── pkg/                  # Layer 3: WASM出力
```

### **変更対象コード**
```rust
// 変更前 (src/core/animation_logic.rs)
pub struct AnimationState {
    pub lines: Vec<usize>,        // ← これを変更
    // ... 他フィールド
}

// 変更後（目標）
pub struct AnimationState {
    pub lines: [usize; 4],       // テトリス最大4ライン
    pub lines_count: usize,      // 実際の使用数
    // ... 他フィールド
}
```

### **テスト状況の確認コマンド**
```bash
# テスト実行
cargo test --lib

# 変更状況確認
git status
git diff

# 最新タグ確認
git tag --list | tail -5
```

---

## 📋 **回復後の作業手順**

### **Step 1: 状況確認（必須）**
1. 現在のディレクトリ確認
2. テスト状況確認（目標: 92/92）
3. git状況確認
4. 設計文書の最新状況確認

### **Step 2: 実装続行**
1. AnimationState構造体変更
2. 関連関数の修正
3. テスト修正
4. 動作確認

### **Step 3: 完了確認**
1. 全テスト通過確認
2. コミット作成
3. Phase 1完了タグ作成

---

## 🎯 **成功条件**
- [ ] cargo test --lib: 92/92 passed
- [ ] AnimationState.lines が [usize; 4] + count パターン
- [ ] CoreGameState.animations が [AnimationState; 20] + count パターン  
- [ ] 既存機能の動作維持
- [ ] WASM境界安全性向上

---

## ⚠️ **注意事項**
1. **データ破損防止**: 必ずテスト確認後に変更
2. **段階的変更**: 一度に全て変更せず、構造体→関数→テストの順
3. **ロールバック準備**: git tagでセーフポイント確保済み
4. **設計準拠**: CLI_WASM_INTEGRATION_REDESIGN.md の Layer 1要件準拠

---

## 📞 **エスカレーション**
もし回復が困難な場合：
1. `git reset --hard phase1-implementation-start` で安全地点に戻る
2. IMPLEMENTATION_ROADMAP.md の Phase 1手順を再確認
3. 段階的に再開始