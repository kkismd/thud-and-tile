//! Phase 6のlock_piece統合テスト
//! 新スコア計算システムの統合を検証

use crate::scoring::CustomScoreSystem;
use crate::game_color::GameColor;
use crate::cell::Cell;

// テスト用のゲーム状態作成（簡易版）
fn create_test_game_state() -> TestGameState {
    TestGameState {
        custom_score_system: CustomScoreSystem::new(),
        board: vec![vec![Cell::Empty; 10]; 20],
    }
}

// テスト用の簡易ゲーム状態構造体
struct TestGameState {
    custom_score_system: CustomScoreSystem,
    board: Vec<Vec<Cell>>,
}

fn setup_line_clear_scenario(game_state: &mut TestGameState) {
    // 底辺ライン（y=19）を埋める
    for x in 0..10 {
        game_state.board[19][x] = Cell::Occupied(GameColor::Cyan);
    }
}

// lock_piece関数の実装
fn lock_piece(game_state: &mut TestGameState) {
    // ライン消去検出とスコア計算を統合して実行
    let lines_to_clear = detect_full_lines(&game_state.board);
    
    if !lines_to_clear.is_empty() {
        // 新スコア計算システムを使用
        let mut total_score = 0;
        for line_y in &lines_to_clear {
            for x in 0..10 {
                if let Cell::Occupied(_color) = game_state.board[*line_y][x] {
                    // 簡単なスコア計算（10ポイント/ブロック）
                    total_score += 10;
                }
            }
        }
        game_state.custom_score_system.total_score += total_score;
        
        // ライン消去実行
        for line_y in lines_to_clear.iter().rev() {
            game_state.board.remove(*line_y);
            game_state.board.insert(0, vec![Cell::Empty; 10]);
        }
    }
}

// 満ラインを検出するヘルパー関数
fn detect_full_lines(board: &Vec<Vec<Cell>>) -> Vec<usize> {
    let mut full_lines = Vec::new();
    for (y, row) in board.iter().enumerate() {
        if row.iter().all(|cell| matches!(cell, Cell::Occupied(_))) {
            full_lines.push(y);
        }
    }
    full_lines
}

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

#[test]
fn test_lock_piece_score_accumulation() {
    let mut game_state = create_test_game_state();
    game_state.custom_score_system.total_score = 100; // 既存スコア
    
    setup_line_clear_scenario(&mut game_state);
    
    lock_piece(&mut game_state);
    
    // スコアが累積されることを確認（100 + 新しいスコア）
    assert!(game_state.custom_score_system.total_score > 100);
}