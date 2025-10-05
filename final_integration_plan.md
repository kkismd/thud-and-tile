# Thud&Tile 新メカニクス最終統合計画書

> **目的**: Phase 1-8で実装済みの新機能パーツをmainループに統合し、完全な新メカニクスを実現  
> **作成日**: 2025年10月5日  
> **前提**: [mechanics_tdd_implementation_plan.md](./mechanics_tdd_implementation_plan.md)の基盤実装完了

## 🎯 統合の目標

### 実現すべき新メカニクス
1. **スコアシステム統合**: 色別スコア → TOTAL SCORE + CHAIN-BONUS表示
2. **CHAIN-BONUS自動更新**: ピース着地時のMAX-CHAIN増加検知と加算
3. **Solidライン相殺システム**: CHAIN-BONUS消費によるEraseLineアニメーション
4. **完全なアニメーション統合**: LineBlink → PushDown → EraseLine順序実行

### 現在の実装状況
- ✅ **Phase 1-6**: 新機能パーツ個別実装完了
- ✅ **Phase 8-1/8-2**: UI表示システム更新完了
- ⚠️ **統合未完了**: mainループとの統合処理が欠如
- ⚠️ **新旧並存**: スコアシステムの二重構造が残存

## 📋 統合戦略とリスク管理

### TDD統合アプローチ
- **段階的統合**: 1機能ずつ順次統合（リスク最小化）
- **継続的検証**: 各ステップで全テスト通過確認
- **ロールバック対応**: 前ステップへの即座復帰可能性維持
- **前提修正対応**: 想定違い発覚時の計画見直し体制

### リスク要因と対策
1. **新旧システム競合**: 段階的移行でリスク軽減
2. **アニメーション統合複雑性**: 分割テストで問題局所化
3. **テスト失敗連鎖**: 各サイクル30分制限で早期発見
4. **想定外の依存関係**: 事前調査と柔軟な計画変更

## 🚀 統合実行フェーズ

### Integration Phase I: スコアシステム統合（高優先度）

#### TDD Cycle I-1: mainループの新スコア計算切り替え

**🔍 事前調査**:
```bash
# 現在のlock_piece()でのスコア計算箇所確認
grep -n "scores.add\|calculate_line_clear_score" src/main.rs
grep -n "total_score" src/main.rs
```

**🔴 RED**: 統合テスト作成
```rust
#[test]
fn test_main_loop_uses_total_score_system() {
    let mut state = GameState::new();
    state.custom_score_system.max_chains.cyan = 3;
    
    // ライン完成状況をセットアップ
    setup_line_clear_scenario(&mut state);
    
    let initial_total = state.custom_score_system.total_score;
    let initial_old_total = state.custom_score_system.scores.total();
    let time_provider = MockTimeProvider::new();
    
    // lock_piece()が新スコア計算を使用することを確認
    state.lock_piece(&time_provider);
    
    // 新システム（total_score）が更新されていることを確認
    assert!(state.custom_score_system.total_score > initial_total);
    
    // ⚠️ 並行期間中は旧システムも更新される
    assert!(state.custom_score_system.scores.total() > initial_old_total);
    
    // 🔍 新旧システムの結果一致を確認（整合性チェック）
    let old_total = state.custom_score_system.scores.total() - initial_old_total;
    let new_total = state.custom_score_system.total_score - initial_total;
    assert_eq!(old_total, new_total, "新旧スコア計算結果は一致するべき");
}
```

**🟢 GREEN**: main.rs のlock_piece()修正
```rust
// src/main.rs の lock_piece()内で変更
// ⚠️ 旧システムは並行動作のため保持
for (color, points) in scores {
    self.custom_score_system.scores.add(color, points);
}

// 🆕 新システム追加（並行実行）:
for &line_y in &lines_to_clear {
    let total_score = scoring::calculate_line_clear_total_score(
        &self.board,
        line_y,
        &self.custom_score_system.max_chains,
    );
    self.custom_score_system.add_total_score(total_score);
}
```

**🔵 REFACTOR**: 
- ⚠️ 旧スコア計算ロジック保持（Phase III-2で削除予定）
- 新旧システム並行動作の検証テスト追加
- import文整理（scoring関数追加）
- テストケース拡張

#### TDD Cycle I-2: CHAIN-BONUS自動更新統合

**🔴 RED**: CHAIN-BONUS更新テスト
```rust
#[test]
fn test_chain_bonus_auto_update_on_piece_lock() {
    let mut state = GameState::new();
    state.custom_score_system.max_chains.cyan = 2;
    state.custom_score_system.max_chains.chain_bonus = 1;
    
    // MAX-CHAINが増加する状況をセットアップ
    setup_chain_increase_scenario(&mut state, GameColor::Cyan, 5);
    
    let time_provider = MockTimeProvider::new();
    state.lock_piece(&time_provider);
    
    // MAX-CHAIN更新確認
    assert_eq!(state.custom_score_system.max_chains.cyan, 5);
    // CHAIN-BONUS増加確認（1 + (5-2) = 4）
    assert_eq!(state.custom_score_system.max_chains.chain_bonus, 4);
}
```

**🟢 GREEN**: lock_piece()にCHAIN-BONUS更新追加
```rust
// src/main.rs の lock_piece()内で追加
// MAX-CHAIN更新の前に旧値を保存
let old_max_chains = self.custom_score_system.max_chains.clone();

// 既存のMAX-CHAIN更新
self.update_max_chains();

// CHAIN-BONUS更新を追加
let increases = scoring::calculate_chain_increases(&old_max_chains, &self.custom_score_system.max_chains);
self.custom_score_system.max_chains.add_chain_bonus(increases);
```

**🔵 REFACTOR**: 
- 処理順序最適化
- エラーハンドリング追加

**⚠️ 前提修正チェックポイント**: 
- MAX-CHAIN更新タイミングが想定と違う場合は処理順序見直し
- 連結ブロック計算結果がMAX-CHAIN反映前の場合はupdate_connected_block_counts()順序調整

### Integration Phase II: EraseLineアニメーション統合（中優先度）

#### TDD Cycle II-1: PushDown完了時の相殺判定統合

**🔴 RED**: 相殺システム統合テスト
```rust
#[test]
fn test_pushdown_triggers_erase_line_animation() {
    let mut state = GameState::new();
    state.custom_score_system.max_chains.chain_bonus = 3;
    
    // 底辺にSolidライン配置
    setup_solid_lines_at_bottom(&mut state, 2);
    
    // PushDownアニメーション完了をシミュレート
    let time_provider = MockTimeProvider::new();
    simulate_pushdown_completion(&mut state, &time_provider);
    
    // EraseLineアニメーションが作成されることを確認
    let erase_animation = find_erase_line_animation(&state.animation);
    assert!(erase_animation.is_some());
    
    let erase = erase_animation.unwrap();
    assert_eq!(erase.target_solid_lines.len(), 2); // min(3, 2) = 2
}
```

**🟢 GREEN**: handle_animation()の拡張
```rust
// src/main.rs の handle_animation()内で追加
// Handle completed push downs の部分で拡張
for gray_line_y in result.completed_push_downs {
    match process_push_down_step(/*...*/) {
        PushDownStepResult::Completed => {
            state.update_all_connected_block_counts();

            // 🆕 相殺判定を追加
            let solid_count = animation::count_solid_lines_from_bottom(&state.board);
            let chain_bonus = state.custom_score_system.max_chains.chain_bonus;
            let erasable_lines = animation::determine_erase_line_count(chain_bonus, solid_count);
            
            if erasable_lines > 0 {
                // EraseLineアニメーション開始
                let target_lines = (0..erasable_lines).map(|i| state.board.len() - 1 - i).collect();
                state.animation.push(Animation::EraseLine {
                    target_solid_lines: target_lines,
                    current_step: 0,
                    last_update: current_time,
                    chain_bonus_consumed: 0,
                });
            } else if state.animation.is_empty() {
                state.spawn_piece();
            }
        }
        // ...existing code...
    }
}
```

**🔵 REFACTOR**: 
- 相殺判定ロジックの関数化
- エラーケース処理追加

#### TDD Cycle II-2: EraseLineアニメーション実行統合

**🔴 RED**: EraseLineアニメーション統合テスト
```rust
#[test]
fn test_erase_line_animation_integration() {
    let mut state = GameState::new();
    state.custom_score_system.max_chains.chain_bonus = 5;
    
    // EraseLineアニメーションを手動作成
    let erase_animation = Animation::EraseLine {
        target_solid_lines: vec![19, 18],
        current_step: 0,
        last_update: Duration::from_millis(0),
        chain_bonus_consumed: 0,
    };
    state.animation.push(erase_animation);
    
    let time_provider = MockTimeProvider::new();
    time_provider.advance(Duration::from_millis(120));
    
    // アニメーション処理実行
    handle_animation(&mut state, &time_provider);
    
    // 1ステップ進行確認
    if let Some(Animation::EraseLine { current_step, .. }) = state.animation.first() {
        assert_eq!(*current_step, 1);
    }
    
    // CHAIN-BONUS未消費確認（アニメーション中）
    assert_eq!(state.custom_score_system.max_chains.chain_bonus, 5);
}
```

**🟢 GREEN**: handle_animation()にEraseLine処理追加
```rust
// src/main.rs の handle_animation()内で追加
// アニメーション更新後に追加処理
let mut completed_erase_lines = Vec::new();

// EraseLineアニメーション個別処理
for i in 0..state.animation.len() {
    if let Animation::EraseLine { .. } = &state.animation[i] {
        let mut animation = state.animation[i].clone();
        let result = animation::process_erase_line_step(
            &mut animation,
            current_time,
            &mut state.board,
            &mut state.current_board_height,
        );
        
        match result {
            animation::EraseLineStepResult::Continue => {
                state.animation[i] = animation;
            }
            animation::EraseLineStepResult::Complete { lines_erased } => {
                completed_erase_lines.push((i, lines_erased));
            }
        }
    }
}

// 完了したEraseLineアニメーション処理
for (index, lines_erased) in completed_erase_lines.into_iter().rev() {
    state.animation.remove(index);
    
    // CHAIN-BONUS消費
    let consumed = animation::consume_chain_bonus_for_erase_line(
        &mut state.custom_score_system.max_chains.chain_bonus,
        lines_erased,
    );
    
    // 接続ブロック更新
    state.update_all_connected_block_counts();
}
```

**🔵 REFACTOR**: 
- アニメーション処理の統合整理
- パフォーマンス最適化

**⚠️ 前提修正チェックポイント**: 
- EraseLineアニメーション処理順序が想定と違う場合はupdate_animations()との統合見直し
- ボード操作タイミングで競合が発生する場合は排他制御検討

### Integration Phase III: 統合完成とクリーンアップ（低優先度）

#### TDD Cycle III-1: 全システム統合検証

**🔴 RED**: エンドツーエンド統合テスト
```rust
#[test]
fn test_complete_new_mechanics_integration() {
    let mut state = GameState::new();
    let time_provider = MockTimeProvider::new();
    
    // 完全なゲームシナリオのシミュレーション
    // 1. ピース着地 → MAX-CHAIN増加 → CHAIN-BONUS加算
    simulate_piece_lock_with_chain_increase(&mut state, &time_provider);
    
    // 2. ライン完成 → LineBlink → PushDown
    simulate_line_clear_sequence(&mut state, &time_provider);
    
    // 3. PushDown完了 → EraseLine相殺
    simulate_erase_line_sequence(&mut state, &time_provider);
    
    // 最終状態検証
    assert!(state.custom_score_system.total_score > 0); // 新スコア計算
    assert!(state.custom_score_system.max_chains.chain_bonus < initial_chain_bonus); // CHAIN-BONUS消費
    assert!(count_solid_lines_from_bottom(&state.board) < initial_solid_count); // Solidライン相殺
}
```

**🟢 GREEN**: 統合不具合修正
- 発見された統合問題の個別修正
- タイミング競合の解決
- エッジケース対応

**🔵 REFACTOR**: 
- 全体最適化
- 不要コード削除
- ドキュメント更新

#### TDD Cycle III-2: 旧システム削除（Phase 7統合）

**⚠️ 注意**: この段階は慎重に実行。大量の依存関係修正が必要。

**� 事前準備**: 影響テストサーベイとチェックリスト作成
```bash
# 旧システム依存テスト特定
grep -r "scores\\.add\|scores\\.get\|scores\\.total\|ColorScores" src/tests/
grep -r "test.*score.*system\|test.*color.*score" src/tests/

# テスト一覧作成（テスト名と確認内容）
find src/tests/ -name "*.rs" -exec grep -l "scores\|ColorScores" {} \;
```

**📋 テスト分類チェックリスト作成**:
```
影響テスト分析シート:
┌─────────────────────────────────────────────────────────────┐
│ テスト関数名 │ 確認内容 │ 対応分類 │ 修正状況 │ 備考 │
├─────────────────────────────────────────────────────────────┤
│ test_xxx     │ 何を確認 │ 1/2/3   │ ✓/✗   │ 注意点│
└─────────────────────────────────────────────────────────────┘

対応分類:
1. 削除してそのままで大丈夫
2. 削除して新システム向けの同じ観点のテストが必要  
3. 保持して動作するように修正が必要
```

**�🔴 RED**: 旧システム削除テスト
```rust
#[test]
fn test_old_score_system_removed() {
    let system = CustomScoreSystem::new();
    
    // 旧システムフィールドが削除されていることを確認
    // system.scores; // <- コンパイルエラーになることを期待
    
    // 新システムのみ動作確認
    assert_eq!(system.total_score, 0);
}
```

**🟢 GREEN**: 段階的旧システム削除
```
実行手順:
1. 事前準備フェーズ:
   a. 影響テスト完全サーベイ
   b. テスト分類チェックリスト作成
   c. 新システム対応テスト事前作成

2. 削除実行フェーズ:
   a. `ColorScores`構造体削除
   b. `CustomScoreSystem.scores`フィールド削除
   c. チェックリストに基づく段階的テスト修正
   
3. 検証フェーズ:
   a. 全テスト通過確認
   b. 機能回帰テスト実行
   c. チェックリスト完了確認
```

**🔵 REFACTOR**: 
- 最終クリーンアップ
- パフォーマンス検証
- 完全テストスイート実行
- チェックリスト完了報告

## 🚨 リスク対応とトラブルシューティング

### 想定される問題と対策

#### 1. **アニメーション処理競合**
**症状**: EraseLineアニメーション中にother animationsと競合  
**対策**: アニメーション排他制御の追加  
**回避**: 一時的にEraseLineアニメーション無効化

#### 2. **スコア計算不整合**
**症状**: 新旧スコア計算結果が一致しない  
**対策**: 並行期間中の比較テスト追加  
**回避**: 段階的移行での検証強化

#### 3. **旧システム早期削除リスク** 🆕
**症状**: TDD Cycle I-1で旧システム削除により大量テスト失敗  
**対策**: Phase III-2まで旧システム保持、新旧並行動作  
**回避**: 各サイクルで旧システム依存の確認と段階的移行

#### 3. **テスト失敗連鎖**
**症状**: 1つの変更が複数テストに影響  
**対策**: 30分制限での早期ロールバック  
**回避**: より小さなサイクルでの分割実装

#### 4. **旧システム依存テスト**
**症状**: 旧システム削除時の大量テスト失敗  
**対策**: Phase III-2での一括対応、依存テスト事前特定  
**回避**: 新旧並行期間での整合性維持

#### 5. **大量テスト修正時の作業ミス** 🆕
**症状**: Phase III-2での大量テスト修正時の編集ミス、見落とし  
**対策**: 事前サーベイ、3分類チェックリスト、段階的修正  
**回避**: 自動化可能な部分の事前準備、小分割作業

#### 4. **ボード状態破損**
**症状**: EraseLineアニメーション後のボード状態異常  
**対策**: ボード状態バリデーション追加  
**回避**: 手動ボード操作の一時停止

#### 5. **新旧システム並行期間の問題**
**症状**: 新旧システム同期不整合、メモリ使用量増加  
**対策**: 整合性検証テスト、早期Phase III移行  
**回避**: 並行期間最小化、定期的整合性チェック

### 計画修正プロトコル

#### レベル1: 軽微な修正（継続）
- **判定基準**: 1-2個のテスト失敗、想定内の小問題
- **対応**: 該当サイクル内での修正継続
- **期限**: サイクル制限時間内（30分）

#### レベル2: 中規模修正（ロールバック）
- **判定基準**: 5個以上のテスト失敗、設計前提の部分的修正必要
- **対応**: 前TDDサイクルへのロールバック
- **再計画**: 問題箇所の分析と計画細分化

#### レベル3: 大規模修正（計画見直し）
- **判定基準**: 統合アプローチの根本的問題、アーキテクチャ変更必要
- **対応**: 全Integration Phase の一時停止
- **再設計**: 統合戦略の根本的見直し

#### 特別対応: 大量テスト修正作業（Phase III-2専用）
- **判定基準**: チェックリスト対応率50%以下、修正ミス多発
- **対応**: 作業分割、自動化検討、段階的実行
- **品質保証**: 中間検証ポイント設定、ペアレビュー導入

## ✅ 各フェーズ完了基準

### Integration Phase I 完了基準
- [ ] 全テストが通過（95/95維持）
- [ ] 新スコア計算システムが稼働
- [ ] CHAIN-BONUS自動更新が動作
- [ ] ⚠️ 新旧システム並行動作確認（整合性検証）
- [ ] 旧スコア計算ロジック保持（Phase III-2で削除予定）

### Integration Phase II 完了基準
- [ ] EraseLineアニメーション統合完了
- [ ] PushDown→EraseLine順序実行確認
- [ ] CHAIN-BONUS相殺システム動作
- [ ] 全アニメーションシーケンス統合

### Integration Phase III 完了基準
- [ ] エンドツーエンドテスト通過
- [ ] 旧システム完全削除
- [ ] テスト分類チェックリスト100%完了
- [ ] 全テスト通過（95/95維持）
- [ ] パフォーマンス要件充足
- [ ] 新メカニクス仕様100%実現

## 🎯 最終統合スケジュール

### Week 1: Integration Phase I
- **Day 1-2**: TDD Cycle I-1 (スコア計算統合)
- **Day 3-4**: TDD Cycle I-2 (CHAIN-BONUS統合)
- **Day 5**: Phase I 統合検証とリファクタリング

### Week 2: Integration Phase II  
- **Day 1-3**: TDD Cycle II-1 (相殺判定統合)
- **Day 4-5**: TDD Cycle II-2 (EraseLineアニメーション統合)

### Week 3: Integration Phase III
- **Day 1**: TDD Cycle III-2 事前準備（テストサーベイ、チェックリスト作成）
- **Day 2-3**: TDD Cycle III-1 (全システム統合検証)
- **Day 4**: TDD Cycle III-2 (旧システム削除実行)
- **Day 5**: 最終検証とドキュメント更新

**⚠️ 重要**: 各フェーズで想定外の問題が発覚した場合は、無理に進めず計画見直しを優先する。品質と安定性が最優先事項。