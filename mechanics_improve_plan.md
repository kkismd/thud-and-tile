# Thud&Tile メカニクス改善計画

## スコアシステムの改修計画

### 既存の仕組みの変更点
* SCOREを色別に表示するのをやめて、合計値ひとつにする
  * ColorScoresを単一の total_score: u32 に変更
* 色別のMAX-CHAINだけを表示して最大値は表示しない

* スコア計算方式の変更
  * 従来：色別に加算後、total()で合計計算
  * 改善後：直接合計値(total_score)に加算

* CustomScoreSystem構造体の変更
  * scores: ColorScores → total_score: u32
  * max_chains: ColorMaxChains (chain_bonusメンバ追加)

### 追加要素
* CHAIN-BONUSという新しい値を作成する
  * ColorMaxChains構造体に新しく chain_bonus というメンバを追加する
* CHAIN−BONUSはピースが着地したタイミングで、色別のMAX-CHAIN値が増えたら、その増えた数が加算される
  * 例：CYAN: 2→3, MAGENTA: 4→4, YELLOW: 5→6の場合、CHAIN-BONUSには1+0+1=2が加算される
* CHAIN-BONUSは後述するSolidラインの相殺で1ラインにつき1ポイント消費される
  * 着地→連結数字計算→MAX-CHAIN更新→CHAIN-BONUS加算の順序で実行

### 変更後のUIイメージ

```
SCORE:    1120

MAX-CHAIN:
  CYAN:    2
  MAGENTA: 4
  YELLOW:  5

CHAIN-BONUS: 11
```

## Solidラインシステムの改修計画

フィールドの高さを狭めるSolidラインは、ライン消去のたびに揃ったラインの数だけ積み上がるが、積み上がった後に、CHAIN-BONUSが1以上の場合はCHAIN-BONUSの数値を消費してSolidラインの相殺（EraseLine）が起こる。
CHAIN-BONUSが0の場合は相殺が起こらない。

相殺のアニメーションは
- CHAIN-BONUSが1減らされる
- Solidラインが1列消し込まれる
- CHAIN-BONUSが0になるか、Solidラインがすべて消えるまで、これが繰り返される

アニメーションシーケンスとして、LineBlink, PushDownのあとにEraseLineというシーケンスを新しく追加して対応する。

- LineBlink -- ラインが揃った後に点滅する演出
- PushDown -- Solidラインが降下していく演出
- EraseLine -- Solidラインが相殺される演出

PushDownは「残っているブロックを1段ずつ消しながら一番下のSolidラインの上に下がっていく」という演出を実装するシーケンス

PushDown完了後にEraseLineを開始するという順序になる

アニメーションの間隔はまず120ミリ秒で実装し、動いているのを見て調整したい

## ゲームバランスに関する考察

現状のメカニクスでは、1ライン揃うごとに、ボトムでないかぎりSolidラインが1段ずつ増えていくため、着実にフィールドが狭くなっていくく。

MAX-CHAINを更新するごとにSolidラインを相殺できれば、ゲームを継続させるというメリットが生まれるため、色を揃える行為が単にスコアを稼ぐこと以上の価値とリターンにつながる。

難易度設定のポイントとしては、MAX-CHAINの合計値=相殺できるライン数なので、ラインを揃えるたびに相殺がおきるわけではないということ。
MAX-CHAINを伸ばすことには限界があるため、必ず何処かで頭打ちとなり、相殺はそこでストップする。そのあとはフィールドが迫ってきてゲームオーバーとなる。

## 既存テストへの影響

ColorScoresを廃止すると、既存のテストコード（約15個）の修正が必要になります。

## TDD実装計画（Red-Green-Refactor）

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
    
    let result = process_erase_line_step(&mut animation, Duration::from_millis(100));
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

### Phase 8: UI/表示系更新とCHAIN-BONUS連携

#### TDD Cycle 8-1: スコア表示UI更新
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
    assert!(!display_text.contains("CYAN:"), "個別色スコアは非表示");
}
```

**GREEN**: 
- `CustomScoreSystem`の`Display`実装をtotal_score中心に変更
- CHAIN-BONUS表示の追加
- 色別スコアから統合スコア表示への変更

**REFACTOR**: 
- 表示フォーマットの最適化

#### TDD Cycle 8-2: render.rs更新（CLI版）
**RED**: 
```rust
#[test]
fn test_render_score_display_cli() {
    let mut game_state = create_test_game_state();
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

### Phase 9: EraseLineアニメーション連携システム

#### TDD Cycle 9-1: EraseLineアニメーション更新処理
**RED**: 
```rust
#[test]
fn test_erase_line_animation_updates() {
    let mut animations = vec![Animation::EraseLine { 
        lines_remaining: 2,
        last_update: Duration::from_millis(0)
    }];
    let current_time = Duration::from_millis(120);
    
    let result = update_animations(&mut animations, current_time);
    
    // 1つのラインが削除されることを確認
    if let Animation::EraseLine { lines_remaining, .. } = &animations[0] {
        assert_eq!(*lines_remaining, 1);
    }
}
```

**GREEN**: 
- `update_animations()`にEraseLine処理追加

**REFACTOR**: 
- 120ミリ秒間隔の調整可能性

#### TDD Cycle 8-2: CHAIN-BONUS消費ロジック統合
**RED**: 
```rust
#[test]
fn test_chain_bonus_creates_erase_line_animation() {
    let mut game_state = create_test_game_state();
    game_state.custom_score_system.max_chains.chain_bonus = 3;
    add_solid_lines_to_board(&mut game_state, 2);
    
    trigger_push_down_completion(&mut game_state);
    
    // EraseLineアニメーションが作成されることを確認
    assert!(has_erase_line_animation(&game_state.animations));
    assert_eq!(get_erase_line_count(&game_state.animations), 2); // min(3, 2)
}
```

**GREEN**: 
- PushDownアニメーション完了時のCHAIN-BONUS消費処理
- EraseLineアニメーション生成ロジック

**REFACTOR**: 
- CHAIN-BONUS管理の最適化

### Phase 9: 高度機能実装

#### TDD Cycle 9-1: 複数アニメーション同時実行
**RED**: 
```rust
#[test]
fn test_multiple_animations_simultaneously() {
    let mut game_state = create_test_game_state();
    
    // LineBlink + EraseLineの同時実行
    game_state.animations.push(Animation::LineBlink { /* ... */ });
    game_state.animations.push(Animation::EraseLine { /* ... */ });
    
    update_animations(&mut game_state.animations, Duration::from_millis(120));
    
    // 両方のアニメーションが正常に更新されることを確認
    assert_eq!(game_state.animations.len(), 2);
}
```

**GREEN**: 
- 複数アニメーション処理の並列実行

**REFACTOR**: 
- パフォーマンス最適化

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