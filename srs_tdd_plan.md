# SRS実装 TDD計画書

## 🎯 目標
現在の独自回転システムから標準的なSuper Rotation System (SRS)への移行を、TDD手法で安全かつ確実に実装する。

## 📋 全体ロードマップ

### Phase 1: 回転状態管理 (Week 1)
**目標**: Tetrominoに回転状態(0,1,2,3)を追加し、回転時に正しく更新

#### 🔴 Red Phase
1. **回転状態フィールドのテスト**
   - 新しく作成されたテトリミノの初期回転状態が0
   - 時計回り回転で状態が0→1→2→3→0と循環
   - 反時計回り回転で状態が0→3→2→1→0と循環

2. **回転状態取得メソッドのテスト**
   - `get_rotation_state()` メソッドの動作確認

#### 🟢 Green Phase
1. **Tetrominoに`rotation_state: u8`フィールド追加**
2. **`get_rotation_state()`メソッド実装**
3. **`rotated()`と`rotated_counter_clockwise()`での状態更新**

#### 🔵 Refactor Phase
1. **コードの整理と最適化**
2. **既存テストの互換性確認**

### Phase 2: SRS標準回転中心 (Week 2)
**目標**: 各テトリミノの回転中心をSRS標準に変更

#### 🔴 Red Phase
1. **SRS標準回転のテストケース**
   - Tミノ: (1,1)中心の回転動作
   - L,J,S,Zミノ: SRS準拠の回転パターン
   - Iミノ: SRS標準の回転中心

#### 🟢 Green Phase
1. **各テトリミノの回転ロジックをSRS準拠に修正**
2. **回転中心の調整**

#### 🔵 Refactor Phase
1. **回転ロジックの統一化**
2. **既存テストの更新**

### Phase 3: Wall Kickシステム基盤 (Week 3)
**目標**: Wall Kickの基本機能を実装

#### 🔴 Red Phase
1. **基本的なWall Kickテスト**
   - 壁際での回転試行
   - 単純なオフセット試行

#### 🟢 Green Phase
1. **基本的なWall Kick試行ロジック**
2. **簡単なオフセットテーブル**

#### 🔵 Refactor Phase
1. **Wall Kickロジックの最適化**

### Phase 4: 完全なSRSオフセットテーブル (Week 4)
**目標**: 標準SRSのオフセットテーブルを完全実装

#### 🔴 Red Phase
1. **SRS標準のWall Kickテスト**
   - 各テトリミノの標準的なWall Kickパターン
   - 複雑なWall Kick状況のテスト

#### 🟢 Green Phase
1. **完全なSRSオフセットテーブル実装**
2. **5段階試行ロジック実装**

#### 🔵 Refactor Phase
1. **パフォーマンス最適化**
2. **コードの整理**

### Phase 5: 統合とバリデーション (Week 5)
**目標**: 色の一貫性確保と全体的な品質保証

#### 🔴 Red Phase
1. **色の一貫性テスト**
   - SRS回転後の色位置確認
   - 複雑な回転シーケンスでの色保持

#### 🟢 Green Phase
1. **色ローテーション最適化**
2. **統合機能の調整**

#### 🔵 Refactor Phase
1. **最終的なコード整理**
2. **パフォーマンス調整**

## 🔧 実装の詳細ガイドライン

### テスト作成時の注意点
1. **色の検証を必須に**: 各テストで色の位置を厳密にチェック
2. **境界条件の考慮**: 壁際、他のブロック近くでの動作
3. **既存テスト構造の活用**: `assert_piece_state`関数を積極的に使用

### コード品質基準
1. **毎フェーズ後にcargo test実行**: 46テスト + 新規テスト全て通過
2. **cargo clippyでリント**: 警告ゼロを維持
3. **cargo fmtでフォーマット**: 一貫したコードスタイル

### 進行管理
1. **各フェーズ完了後にコミット**: 意味のある単位でのバージョン管理
2. **game_spec.mdの更新**: 進捗状況を記録
3. **回帰テストの実行**: 既存機能に影響がないことを確認

## 📊 成功基準

### 機能面
- [ ] 全7種のテトリミノでSRS標準回転動作
- [ ] Wall Kickが標準的な状況で正常動作
- [ ] 色の一貫性が全回転パターンで保持
- [ ] 既存ゲーム機能に影響なし

### 品質面  
- [ ] 全テスト通過 (既存46 + 新規約20テスト)
- [ ] コードカバレッジ80%以上
- [ ] パフォーマンス劣化なし
- [ ] メモリリーク等の問題なし

## 🚀 Phase 1 開始準備

### 最初のテスト仕様
```rust
#[test]
fn test_tetromino_initial_rotation_state() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let piece = Tetromino::from_shape(TetrominoShape::T, colors);
    assert_eq!(piece.get_rotation_state(), 0);
}

#[test]
fn test_clockwise_rotation_state_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::T, colors);
    
    assert_eq!(piece.get_rotation_state(), 0);
    piece = piece.rotated();
    assert_eq!(piece.get_rotation_state(), 1);
    piece = piece.rotated();
    assert_eq!(piece.get_rotation_state(), 2);
    piece = piece.rotated();
    assert_eq!(piece.get_rotation_state(), 3);
    piece = piece.rotated();
    assert_eq!(piece.get_rotation_state(), 0);
}
```

Phase 1の実装開始準備完了！