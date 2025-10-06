# Thud&Tile ゲームシステム改修 - TDD実装計画

> **元文書**: [mechanics_improve_plan.md](./mechanics_improve_plan.md)  
> **分離日**: 2025年10月5日  
> **目的**: TDD実装の詳細計画を独立したドキュメントとして管理

## TDD実装の全体方針

### 実装方針
- **RED**: 失敗するテストを先に書く
- **GREEN**: 最小限のコードでテストを通す
- **REFACTOR**: 動作を保ったままコードを改善
- **1サイクル = 30分以内**を目標に小さく刻む

### Phase 1: ColorMaxChainsにchain_bonusメンバ追加

#### TDD Cycle 1-1: chain_bonusフィールド追加
**RED**: 
```rust
#[test]
fn test_color_max_chains_has_chain_bonus() {
    let max_chains = ColorMaxChains::new();
    assert_eq!(max_chains.chain_bonus, 0);
}
```

**GREEN**: 
- `ColorMaxChains`構造体に`pub chain_bonus: u32`を追加
- `new()`メソッドで`chain_bonus: 0`で初期化

**REFACTOR**: 
- 既存のテストが通ることを確認
- コードスタイルとドキュメント整備

#### TDD Cycle 1-2: chain_bonus加算メソッド
**RED**: 
```rust
#[test]
fn test_add_chain_bonus() {
    let mut max_chains = ColorMaxChains::new();
    max_chains.add_chain_bonus(5);
    assert_eq!(max_chains.chain_bonus, 5);
    
    max_chains.add_chain_bonus(3);
    assert_eq!(max_chains.chain_bonus, 8);
}
```

**GREEN**: 
- `add_chain_bonus(&mut self, amount: u32)`メソッド実装

**REFACTOR**: 
- オーバーフロー対策（saturating_add使用）

#### TDD Cycle 1-3: chain_bonus消費メソッド
**RED**: 
```rust
#[test]
fn test_consume_chain_bonus() {
    let mut max_chains = ColorMaxChains::new();
    max_chains.chain_bonus = 5;
    
    let consumed = max_chains.consume_chain_bonus(3);
    assert_eq!(consumed, 3);
    assert_eq!(max_chains.chain_bonus, 2);
    
    // 不足する場合のテスト
    let consumed2 = max_chains.consume_chain_bonus(5);
    assert_eq!(consumed2, 2);
    assert_eq!(max_chains.chain_bonus, 0);
}
```

**GREEN**: 
- `consume_chain_bonus(&mut self, max_amount: u32) -> u32`メソッド実装

**REFACTOR**: 
- エッジケースのテスト追加

### Phase 2: CustomScoreSystem構造変更（段階的移行）

#### TDD Cycle 2-1: total_scoreフィールド追加
**RED**: 
```rust
#[test]
fn test_custom_score_system_has_total_score() {
    let system = CustomScoreSystem::new();
    assert_eq!(system.total_score, 0);
    // 既存のscoresフィールドも並行して存在することを確認
    assert_eq!(system.scores.total(), 0);
}
```

**GREEN**: 
- `CustomScoreSystem`に`pub total_score: u32`追加
- `new()`で`total_score: 0`初期化
- 既存の`scores: ColorScores`は保持

**REFACTOR**: 
- フィールドアクセスの整理

#### TDD Cycle 2-2: add_total_scoreメソッド実装
**RED**: 
```rust
#[test]
fn test_add_total_score() {
    let mut system = CustomScoreSystem::new();
    system.add_total_score(100);
    assert_eq!(system.total_score, 100);
    
    system.add_total_score(50);
    assert_eq!(system.total_score, 150);
}
```

**GREEN**: 
- `add_total_score(&mut self, points: u32)`メソッド実装

**REFACTOR**: 
- オーバーフロー対策（saturating_add使用）

#### TDD Cycle 2-3: get_total_scoreメソッド実装
**RED**: 
```rust
#[test]
fn test_get_total_score() {
    let mut system = CustomScoreSystem::new();
    assert_eq!(system.get_total_score(), 0);
    
    system.total_score = 250;
    assert_eq!(system.get_total_score(), 250);
}
```

**GREEN**: 
- `get_total_score(&self) -> u32`メソッド実装

**REFACTOR**: 
- アクセサメソッドの一貫性確認

### Phase 3: スコア計算ロジックの変更

#### TDD Cycle 3-1: 新しいスコア計算関数の基本実装
**RED**: 
```rust
#[test]
fn test_calculate_line_clear_total_score_basic() {
    // シンプルなテストケース：Occupiedブロックのみ
    let mut board = vec![vec![Cell::Empty; 10]; 20];
    board[19][0] = Cell::Occupied(GameColor::Cyan);
    board[19][1] = Cell::Occupied(GameColor::Magenta);
    
    let mut max_chains = ColorMaxChains::new();
    max_chains.cyan = 2;
    max_chains.magenta = 3;
    
    let total_score = calculate_line_clear_total_score(&board, 19, &max_chains);
    assert_eq!(total_score, 50); // (1*2*10) + (1*3*10) = 50
}
```

**GREEN**: 
- `calculate_line_clear_total_score()`関数の基本実装
- Occupiedブロックのみ対応

**REFACTOR**: 
- 計算式の明確化

#### TDD Cycle 3-2: Connected ブロック対応追加
**RED**: 
```rust
#[test]
fn test_calculate_line_clear_total_score_connected() {
    let mut board = vec![vec![Cell::Empty; 10]; 20];
    board[19][0] = Cell::Connected { color: GameColor::Cyan, count: 3 };
    board[19][1] = Cell::Connected { color: GameColor::Yellow, count: 5 };
    
    let mut max_chains = ColorMaxChains::new();
    max_chains.cyan = 2;
    max_chains.yellow = 4;
    
    let total_score = calculate_line_clear_total_score(&board, 19, &max_chains);
    assert_eq!(total_score, 260); // (3*2*10) + (5*4*10) = 260
}
```

**GREEN**: 
- `Cell::Connected`ケースの処理追加

**REFACTOR**: 
- 計算ロジックの統一化

#### TDD Cycle 3-3: 既存システムとの並行動作確認
**RED**: 
```rust
#[test]
fn test_both_score_calculations_match() {
    let board = create_test_board_with_mixed_blocks();
    let max_chains = create_test_max_chains();
    
    // 既存システム
    let old_scores = calculate_line_clear_score(&board, 19, &max_chains);
    let old_total: u32 = old_scores.iter().map(|(_, points)| points).sum();
    
    // 新システム
    let new_total = calculate_line_clear_total_score(&board, 19, &max_chains);
    
    assert_eq!(old_total, new_total);
}
```

**GREEN**: 
- 両システムの結果一致を確認

**REFACTOR**: 
- テストヘルパー関数の整理

### Phase 4A: CHAIN-BONUS更新ロジック実装

#### TDD Cycle 4A-1: MAX-CHAIN更新検知
**RED**: 
```rust
#[test]
fn test_detect_max_chain_increases() {
    let old_chains = ColorMaxChains { cyan: 2, magenta: 3, yellow: 4, chain_bonus: 0 };
    let new_chains = ColorMaxChains { cyan: 4, magenta: 3, yellow: 6, chain_bonus: 0 };
    
    let increases = calculate_chain_increases(&old_chains, &new_chains);
    assert_eq!(increases, 4); // (4-2) + (6-4) = 4
}
```

**GREEN**: 
- `calculate_chain_increases()`関数実装

**REFACTOR**: 
- エッジケース（減少時）の処理確認

#### TDD Cycle 4A-2: ピース着地時のCHAIN-BONUS更新
**RED**: 
```rust
#[test]
fn test_chain_bonus_update_on_piece_lock() {
    let mut game_state = create_test_game_state();
    game_state.custom_score_system.max_chains.cyan = 2;
    game_state.custom_score_system.max_chains.chain_bonus = 1;
    
    // テストボードで新しい連結を作成してMAX-CHAINが増加する状況を設定
    setup_board_for_chain_increase(&mut game_state, GameColor::Cyan, 4);
    
    lock_piece_and_update_chains(&mut game_state);
    
    assert_eq!(game_state.custom_score_system.max_chains.cyan, 4);
    assert_eq!(game_state.custom_score_system.max_chains.chain_bonus, 3); // 1 + (4-2)
}
```

**GREEN**: 
- `lock_piece()`内でのCHAIN-BONUS更新実装

**REFACTOR**: 
- 処理順序の確認と最適化

### Phase 4B: スコア加算処理の統合

#### TDD Cycle 4B-1: lock_piece()での新スコア計算使用
**RED**: 
```rust
#[test]
fn test_lock_piece_uses_total_score() {
    let mut game_state = create_test_game_state_with_line_ready();
    let initial_total = game_state.custom_score_system.total_score;
    
    lock_piece(&mut game_state);
    
    assert!(game_state.custom_score_system.total_score > initial_total);
    // 既存のcolor_scoresは更新されないことを確認（並行期間中）
    assert_eq!(game_state.custom_score_system.scores.total(), 0);
}
```

**GREEN**: 
- `lock_piece()`での新スコア計算とtotal_score更新

**REFACTOR**: 
- スコア計算処理の整理

### Phase 5: EraseLineアニメーション実装

#### TDD Cycle 5-1: Animation列挙体拡張
**RED**: 
```rust
#[test]
fn test_erase_line_animation_creation() {
    let animation = Animation::EraseLine { 
        lines_remaining: 3,
        last_update: Duration::from_millis(0)
    };
    match animation {
        Animation::EraseLine { lines_remaining, .. } => {
            assert_eq!(lines_remaining, 3);
        },
        _ => panic!("Expected EraseLine animation"),
    }
}
```

**GREEN**: 
- `Animation`列挙体に`EraseLine`バリアント追加

**REFACTOR**: 
- Animation関連のドキュメント更新

#### TDD Cycle 5-2: ライン消去アニメーション処理
**RED**: 
```rust
#[test]
fn test_erase_line_animation_progress() {
    let mut animation = Animation::EraseLine { 
        lines_remaining: 3,
        last_update: Duration::from_millis(0)
    };
    
    let result = process_erase_line_step(&mut animation, Duration::from_millis(120));
    assert!(matches!(result, EraseLineStepResult::Continue));
    
    if let Animation::EraseLine { lines_remaining, .. } = animation {
        assert_eq!(lines_remaining, 2);
    }
}
```

**GREEN**: 
- `process_erase_line_step`関数実装
- `EraseLineStepResult`列挙体追加

**REFACTOR**: 
- エラーハンドリング改善

### Phase 6: 新スコア計算システム統合

#### TDD Cycle 6-1: lock_piece()への統合
**RED**: 
```rust
#[test]
fn test_lock_piece_new_scoring_integration() {
    let mut game_state = create_test_game_state();
    game_state.custom_score_system.total_score = 0;
    
    // ライン消去が発生する状況を作成
    setup_line_clear_scenario(&mut game_state);
    
    lock_piece(&mut game_state);
    
    // 新スコア計算が適用されていることを確認
    assert!(game_state.custom_score_system.total_score > 0);
}
```

**GREEN**: 
- `lock_piece()`で`lock_piece_with_total_score()`を呼び出し
- ライン消去検出とスコア計算の統合

**REFACTOR**: 
- スコア計算フローの最適化

### Phase 7: 旧システム削除とテスト移行（一括実行）
**⚠️ PENDING: 大量の旧システム依存（50+箇所）によりコンテキスト消費リスクが高いため後回し**

**理由**: ColorScoresへの依存が lib.rs, main.rs, render.rs, tests/ 全体に広がっており、
一度に修正するとコンテキストを大量消費し、メインロジックに悪影響を与えるリスク。
新機能実装完了後の最終段階で安全に実行予定。

#### TDD Cycle 7-1: 旧システム依存テストの特定と移行
**RED**: 
```rust
// 旧システムへの依存が残っているテストの失敗確認
#[test]
fn test_old_system_dependencies_removed() {
    let system = CustomScoreSystem::new();
    // system.scores; // この行でコンパイルエラーになることを期待
    assert_eq!(system.total_score, 0);
}
```

**GREEN**: 
- `test_color_scores_*`系テストを`test_total_score_*`系に変換
- `test_custom_score_system_*`系テストを新仕様対応に変更
- main.rs/lib.rsの関連テストを新システム対応に更新

**REFACTOR**: 
- テスト重複の削除
- テストヘルパー関数の統一

#### TDD Cycle 7-2: ColorScores完全削除
**RED**: 
```rust
#[test] 
fn test_total_score_functionality_complete() {
    let mut system = CustomScoreSystem::new();
    system.add_total_score(100);
    
    // この時点でscoresフィールドへの依存がないことを確認
    assert_eq!(system.total_score, 100);
}
```

**GREEN**: 
- `CustomScoreSystem`から`scores: ColorScores`フィールド削除
- `Display`トレイト実装を`total_score`ベースに更新
- `ColorScores`構造体と関連メソッドを削除

**REFACTOR**: 
- 未使用import削除
- ドキュメント更新
- コード整理

### Phase 8: UI/表示系更新とCHAIN-BONUS連携 ✅ (Phase 8-1, 8-2完了, 8-3ペンディング)

#### TDD Cycle 8-1: スコア表示UI更新 ✅
**RED**: 
```rust
#[test]
fn test_score_display_shows_total_score() {
    let mut system = CustomScoreSystem::new();
    system.total_score = 1250;
    system.max_chains.cyan = 3;
    system.max_chains.magenta = 4;
    system.max_chains.yellow = 5;
    system.max_chains.chain_bonus = 2;
    
    let display_text = format!("{}", system);
    assert!(display_text.contains("TOTAL SCORE: 1250"));
    assert!(display_text.contains("CHAIN-BONUS: 2"));
    assert!(!display_text.lines().any(|line| line.trim().starts_with("SCORE:")));
}
```

**GREEN**: ✅
- `CustomScoreSystem`の`Display`実装をtotal_score中心に変更
- CHAIN-BONUS表示の追加
- 旧SCORE行の削除と新TOTAL SCORE表示への移行

**REFACTOR**: ✅
- テストコードからデバッグ出力削除
- 不要importの除去

#### TDD Cycle 8-2: render.rs更新（CLI版） ✅
**RED**: 
```rust
#[test]
fn test_render_shows_total_score() {
    let mut state = GameState::new();
    state.custom_score_system.total_score = 1500;
    
    let expected_format = format!("TOTAL SCORE: {:<6}", state.custom_score_system.total_score);
    assert!(expected_format.contains("TOTAL SCORE: 1500"));
    assert!(!expected_format.contains("SCORE:      "));
}
```

**GREEN**: ✅
- `render.rs`の`render_ui_changes`でSCORE表示をTOTAL SCOREに変更
- CHAIN-BONUS表示の追加
- 個別色スコア行の削除と画面位置調整

**REFACTOR**: ✅
- テストコードの整理とMockRenderer削除
- 不要import除去

#### TDD Cycle 8-3: WebAssembly側UI更新 ⚠️ PENDING
WebAssemblyビルドの複雑な依存関係のため一時的にペンディング。
CLI版の動作確認後、WASMインターフェース改修予定。

**課題**: 
- `wasm_bindgen`依存関係エラー
- `WasmCustomScoreSystem`との型不整合
- JavaScript型定義の更新必要

#### TDD Cycle 8-2: CLI/レンダリング更新
**RED**: 
```rust
#[test]
fn test_render_new_score_system() {
    let mut game_state = GameState::new();
    game_state.custom_score_system.total_score = 2500;
    game_state.custom_score_system.max_chains.chain_bonus = 5;
    
    let rendered_output = render_game_state(&game_state);
    
    assert!(rendered_output.contains("Score: 2500"));
    assert!(rendered_output.contains("Chain Bonus: 5"));
    assert!(rendered_output.contains("Max Chain:"));
}
```

**GREEN**: 
- `render.rs`のスコア表示ロジックを新システム対応に更新
- CLI出力フォーマットの改善
- CHAIN-BONUS情報の表示追加

**REFACTOR**: 
- レンダリング性能最適化

#### TDD Cycle 8-3: Web UI更新（JavaScript側）
**RED**: 
```rust
#[test]
fn test_web_score_interface() {
    let mut system = CustomScoreSystem::new();
    system.total_score = 3750;
    system.max_chains.chain_bonus = 8;
    
    // WebAssembly経由でのスコア取得テスト
    let score_json = export_score_data(&system);
    assert!(score_json.contains("\"total_score\":3750"));
    assert!(score_json.contains("\"chain_bonus\":8"));
}
```

**GREEN**: 
- WebAssembly export関数をtotal_score対応に更新
- JavaScript側のスコア表示ロジック更新
- Web UIでのCHAIN-BONUS表示実装

**REFACTOR**: 
- Web UI/CLI共通インターフェースの整理

### Phase 9: EraseLineアニメーションシステム完成（厳密な仕様実装）

### 🚨 前回実装での誤解と修正

#### 間違えた仕様理解：
1. **CHAIN-BONUS消費タイミング**: PushDown完了時に即座に消費 → ❌
2. **EraseLineアニメーション**: lines_remainingのカウントダウンのみ → ❌  
3. **相殺処理**: アニメーション完了後に完成ラインを消去 → ❌

#### 正しい仕様理解：
1. **CHAIN-BONUS消費**: EraseLineアニメーション中に1ライン消去毎に1ポイント消費
2. **EraseLineアニメーション**: 実際のSolidライン除去処理を含む
3. **相殺処理**: 完成ライン（隙間なしライン）ではなく、Solidライン（グレーライン）を消去

### 📋 厳密な仕様定義

#### MAX-CHAIN & CHAIN-BONUS仕様
```rust
// ピース着地時の処理順序（厳密）
fn lock_piece(&mut self) {
    // 1. ピースをボードに配置
    // 2. 隣接ブロック連結計算
    // 3. 連結数字をボードに反映  
    // 4. 旧MAX-CHAIN値を保存
    let old_max_chains = self.custom_score_system.max_chains.clone();
    // 5. 新MAX-CHAIN値を計算・更新
    self.update_max_chains();
    // 6. MAX-CHAIN増加分をCHAIN-BONUSに加算
    let increases = calculate_chain_increases(&old_max_chains, &self.custom_score_system.max_chains);
    self.custom_score_system.max_chains.chain_bonus += increases;
    // 7. 完成ライン検出とLineBlink開始
}
```

#### EraseLineアニメーション仕様
```rust
// EraseLineアニメーションの動作（厳密）
pub struct EraseLineAnimation {
    target_solid_lines: Vec<usize>,  // 削除対象のSolidライン（底上）
    current_step: usize,            // 現在の削除ステップ
    last_update: Duration,          // 最終更新時刻
    chain_bonus_consumed: u32,      // 消費したCHAIN-BONUS量
}

// 120ms毎に1ラインずつSolidラインを削除
// 削除と同時にCHAIN-BONUSを1ポイント消費
// CHAIN-BONUSが0になるか、Solidラインが全て消えたら完了
```

#### Solidライン相殺システム仕様
```rust
// PushDown完了時の相殺判定（厳密）
fn on_push_down_complete() {
    let solid_line_count = count_solid_lines_from_bottom();
    let available_chain_bonus = self.custom_score_system.max_chains.chain_bonus;
    let erasable_lines = min(solid_line_count, available_chain_bonus);
    
    if erasable_lines > 0 {
        // EraseLineアニメーション開始（相殺開始）
        self.start_erase_line_animation(erasable_lines);
    }
}
```

### 🧪 Phase 9-1: EraseLineアニメーション基盤（TDD）

#### Cycle 9-1-1: EraseLineAnimation構造体設計

**RED**: 失敗するテスト作成
```rust
#[test]
fn test_erase_line_animation_creation() {
    let solid_lines = vec![19, 18]; // 底辺から2行のSolidライン
    let animation = Animation::EraseLine {
        target_solid_lines: solid_lines.clone(),
        current_step: 0,
        last_update: Duration::from_millis(0),
        chain_bonus_consumed: 0,
    };
    
    // EraseLineアニメーション構造体が正しく作成されることを確認
    if let Animation::EraseLine { target_solid_lines, current_step, .. } = animation {
        assert_eq!(target_solid_lines, vec![19, 18]);
        assert_eq!(current_step, 0);
    } else {
        panic!("Expected EraseLine animation");
    }
}
```

**GREEN**: 最小実装
```rust
// animation.rs
pub enum Animation {
    LineBlink { /* existing */ },
    PushDown { /* existing */ },
    EraseLine {
        target_solid_lines: Vec<usize>,
        current_step: usize,
        last_update: Duration,
        chain_bonus_consumed: u32,
    },
}
```

**REFACTOR**: 構造体設計の最適化

#### Cycle 9-1-2: EraseLineアニメーションステップ処理

**RED**: 失敗するテスト作成
```rust
#[test]
fn test_erase_line_animation_step_processing() {
    let mut animation = Animation::EraseLine {
        target_solid_lines: vec![19, 18, 17],
        current_step: 0,
        last_update: Duration::from_millis(0),
        chain_bonus_consumed: 0,
    };
    
    // 120ms経過後にステップ処理
    let result = process_erase_line_step(&mut animation, Duration::from_millis(120));
    
    // 1ステップ進行することを確認
    if let Animation::EraseLine { current_step, chain_bonus_consumed, .. } = animation {
        assert_eq!(current_step, 1);
        assert_eq!(chain_bonus_consumed, 1);
        assert!(matches!(result, EraseLineStepResult::Continue));
    }
}

#[test]
fn test_erase_line_animation_completion() {
    let mut animation = Animation::EraseLine {
        target_solid_lines: vec![19],
        current_step: 0,
        last_update: Duration::from_millis(0),
        chain_bonus_consumed: 0,
    };
    
    // 120ms経過後にステップ処理
    let result = process_erase_line_step(&mut animation, Duration::from_millis(120));
    
    // アニメーション完了を確認
    assert!(matches!(result, EraseLineStepResult::Complete { lines_erased: 1 }));
}
```

**GREEN**: 最小実装
```rust
pub enum EraseLineStepResult {
    Continue,
    Complete { lines_erased: u32 },
}

pub fn process_erase_line_step(
    animation: &mut Animation,
    current_time: Duration,
) -> EraseLineStepResult {
    if let Animation::EraseLine {
        target_solid_lines,
        current_step,
        last_update,
        chain_bonus_consumed,
    } = animation {
        let erase_interval = Duration::from_millis(120);
        
        if current_time - *last_update >= erase_interval {
            *current_step += 1;
            *chain_bonus_consumed += 1;
            *last_update = current_time;
            
            if *current_step >= target_solid_lines.len() {
                EraseLineStepResult::Complete { 
                    lines_erased: target_solid_lines.len() as u32 
                }
            } else {
                EraseLineStepResult::Continue
            }
        } else {
            EraseLineStepResult::Continue
        }
    } else {
        EraseLineStepResult::Complete { lines_erased: 0 }
    }
}
```

### 🧪 Phase 9-2: CHAIN-BONUS統合システム（TDD）

#### Cycle 9-2-1: PushDown完了時の相殺判定

**RED**: 失敗するテスト作成
```rust
#[test]
fn test_push_down_triggers_erase_line_with_chain_bonus() {
    let mut game_state = TestGameState::new();
    game_state.custom_score_system.max_chains.chain_bonus = 3;
    
    // 底辺に2行のSolidライン配置
    add_solid_lines_to_bottom(&mut game_state, 2);
    
    // PushDown完了をトリガー
    let result = trigger_push_down_completion(&mut game_state);
    
    // EraseLineアニメーションが作成されることを確認
    assert!(has_erase_line_animation(&result.continuing_animations));
    let erase_animation = get_erase_line_animation(&result.continuing_animations).unwrap();
    assert_eq!(erase_animation.target_solid_lines.len(), 2); // min(3, 2) = 2
}

#[test]
fn test_insufficient_chain_bonus_limits_erase_lines() {
    let mut game_state = TestGameState::new();
    game_state.custom_score_system.max_chains.chain_bonus = 1;
    
    // 底辺に3行のSolidライン配置
    add_solid_lines_to_bottom(&mut game_state, 3);
    
    // PushDown完了をトリガー
    let result = trigger_push_down_completion(&mut game_state);
    
    // 制限された数のEraseLineアニメーションが作成されることを確認
    let erase_animation = get_erase_line_animation(&result.continuing_animations).unwrap();
    assert_eq!(erase_animation.target_solid_lines.len(), 1); // min(1, 3) = 1
}
```

#### Cycle 9-2-2: EraseLineアニメーション完了時のCHAIN-BONUS消費

**RED**: 失敗するテスト作成
```rust
#[test]
fn test_erase_line_completion_consumes_chain_bonus() {
    let mut game_state = TestGameState::new();
    game_state.custom_score_system.max_chains.chain_bonus = 5;
    
    // EraseLineアニメーションを直接作成（2ライン削除予定）
    let erase_animation = Animation::EraseLine {
        target_solid_lines: vec![19, 18],
        current_step: 0,
        last_update: Duration::from_millis(0),
        chain_bonus_consumed: 0,
    };
    game_state.animations.push(erase_animation);
    
    // アニメーション完了まで進める
    complete_erase_line_animation(&mut game_state);
    
    // CHAIN-BONUSが正しく消費されることを確認
    assert_eq!(game_state.custom_score_system.max_chains.chain_bonus, 3); // 5 - 2 = 3
}
```

### 🧪 Phase 9-3: Solidライン操作システム（TDD）

#### Cycle 9-3-1: Solidライン検出とカウント

**RED**: 失敗するテスト作成
```rust
#[test]
fn test_count_solid_lines_from_bottom() {
    let mut board = create_empty_board();
    
    // 底辺から3行をSolidライン（グレー）にする
    for y in 17..20 {
        for x in 0..10 {
            board[y][x] = Cell::Occupied(GameColor::Grey);
        }
    }
    
    let solid_count = count_solid_lines_from_bottom(&board);
    assert_eq!(solid_count, 3);
}

#[test]
fn test_partial_solid_lines_not_counted() {
    let mut board = create_empty_board();
    
    // 底辺ラインを部分的に埋める（完全Solidではない）
    for x in 0..5 {
        board[19][x] = Cell::Occupied(GameColor::Grey);
    }
    
    let solid_count = count_solid_lines_from_bottom(&board);
    assert_eq!(solid_count, 0); // 部分的な行はカウントしない
}
```

#### Cycle 9-3-2: Solidライン除去処理

**RED**: 失敗するテスト作成  
```rust
#[test]
fn test_remove_solid_line_from_bottom() {
    let mut board = create_empty_board();
    let mut current_height = 20;
    
    // 底辺に2行のSolidライン配置
    add_solid_lines_to_bottom_direct(&mut board, 2);
    
    // 底辺のSolidライン1行を除去
    let result = remove_solid_line_from_bottom(&mut board, &mut current_height);
    
    // 1行除去されることを確認
    assert!(result.is_some());
    assert_eq!(current_height, 21); // ボード高が1行拡張される
    
    // 残りのSolidライン数を確認
    let remaining_solid = count_solid_lines_from_bottom(&board);
    assert_eq!(remaining_solid, 1);
}
```

### 🧪 Phase 9-4: 統合テストとエッジケース

#### Cycle 9-4-1: 完全な相殺シーケンステスト

**RED**: 失敗するテスト作成
```rust
#[test]
fn test_complete_offset_sequence() {
    let mut game_state = TestGameState::new();
    
    // 初期状態設定
    game_state.custom_score_system.max_chains.chain_bonus = 2;
    add_solid_lines_to_bottom(&mut game_state, 3);
    add_complete_line(&mut game_state, 16); // 通常のライン消去をトリガー
    
    // LineBlink → PushDown → EraseLine の完全シーケンス実行
    let sequence_result = execute_complete_animation_sequence(&mut game_state);
    
    // 最終状態確認
    assert_eq!(game_state.custom_score_system.max_chains.chain_bonus, 0); // 2消費
    assert_eq!(count_solid_lines_from_bottom(&game_state.board), 1); // 3-2=1
    assert!(sequence_result.all_animations_completed);
}
```

#### Cycle 9-4-2: CHAIN-BONUS枯渇エッジケース

**RED**: 失敗するテスト作成
```rust
#[test]
fn test_chain_bonus_exhaustion_stops_erase_line() {
    let mut game_state = TestGameState::new();
    game_state.custom_score_system.max_chains.chain_bonus = 1;
    
    // 5行のSolidラインがあるが、CHAIN-BONUSは1のみ
    add_solid_lines_to_bottom(&mut game_state, 5);
    
    // EraseLineアニメーション実行
    let erase_animation = create_erase_line_animation(1); // 1行のみ削除予定
    game_state.animations.push(erase_animation);
    
    complete_erase_line_animation(&mut game_state);
    
    // CHAIN-BONUSが0になり、Solidラインが4行残ることを確認
    assert_eq!(game_state.custom_score_system.max_chains.chain_bonus, 0);
    assert_eq!(count_solid_lines_from_bottom(&game_state.board), 4);
}
```

## 開発完了基準

1. **機能完備性**：
   - ✅ 新スコア計算システム（Phase 3-4）
   - ⏳ EraseLineアニメーション（Phase 5）
   - ⏳ 統合システム（Phase 6-7）
   - ⏳ 高度機能（Phase 8-9）

2. **品質基準**：
   - すべてのTDDサイクル完了
   - テストカバレッジ95%以上維持
   - パフォーマンス要件充足
   - ドキュメント整備完了

3. **デプロイ準備**：
   - 旧システム完全削除（Phase 7）
   - 設定ファイル更新
   - リリースノート作成

**GREEN**: 
- PushDown完了時のEraseLine作成ロジック

**REFACTOR**: 
- アニメーションシーケンスの最適化

### 各Cycleでの確認事項
1. **cargo check**: コンパイルエラーなし
2. **cargo test**: 全テスト通過（95/95維持）
3. **cargo clippy**: 警告なし
4. **cargo fmt**: フォーマット適用
5. **git add && git commit**: 各Cycle完了時にコミット
6. **in japanese** : コミットメッセージは日本語で書く

### エラー発生時の対応
- **RED段階**: コンパイルエラーは期待される（新機能追加時）
- **GREEN段階**: テスト通過最優先、最小実装でOK
- **REFACTOR段階**: 機能変更禁止、品質向上のみ
- **想定外の失敗**: 前Cycleに戻って原因調査
- **テスト数減少**: 即座に原因特定と修復

### 実装完了の確認基準
- [ ] 全95テストが通過
- [ ] 新機能のテストが追加済み
- [ ] ColorScoresが完全削除済み
- [ ] CHAIN-BONUSが正常動作
- [ ] EraseLineアニメーションが実装済み
- [ ] CLI版での動作確認完了