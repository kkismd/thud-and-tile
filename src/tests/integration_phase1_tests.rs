//! Integration Phase I テスト
//! TDD Cycle I-1: mainループの新スコア計算切り替え

use crate::cell::{Board, Cell};
use crate::config::BOARD_WIDTH;
use crate::game_color::GameColor;
use crate::{GameState, MockTimeProvider};

/// テストヘルパー: ライン完成状況をセットアップ
fn setup_line_clear_scenario(state: &mut GameState) {
    // 底辺ライン（y=19）を完成状況にセットアップ
    for x in 0..BOARD_WIDTH {
        state.board[19][x] = Cell::Occupied(GameColor::Cyan);
    }
}

/// TDD Cycle I-1 RED: 新旧システム並行動作の統合テスト
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
    assert!(
        state.custom_score_system.total_score > initial_total,
        "新システム（total_score）が更新されるべき"
    );
    
    // ⚠️ 並行期間中は旧システムも更新される
    assert!(
        state.custom_score_system.scores.total() > initial_old_total,
        "並行期間中は旧システムも更新されるべき"
    );
    
    // 🔍 新旧システムの結果一致を確認（整合性チェック）
    let old_total = state.custom_score_system.scores.total() - initial_old_total;
    let new_total = state.custom_score_system.total_score - initial_total;
    assert_eq!(old_total, new_total, "新旧スコア計算結果は一致するべき");
}

/// 追加テスト: 新スコア計算関数の単体動作確認
#[test]
fn test_new_score_calculation_function() {
    let mut board: Board = vec![vec![Cell::Empty; 10]; 20];
    
    // テストライン作成: Cyanブロック5個、Magentaブロック3個、Connected 2個
    board[19][0] = Cell::Occupied(GameColor::Cyan);
    board[19][1] = Cell::Occupied(GameColor::Cyan);
    board[19][2] = Cell::Occupied(GameColor::Cyan);
    board[19][3] = Cell::Occupied(GameColor::Magenta);
    board[19][4] = Cell::Occupied(GameColor::Magenta);
    board[19][5] = Cell::Connected { color: GameColor::Yellow, count: 2 };
    // 残りは空
    
    let mut max_chains = crate::scoring::ColorMaxChains::new();
    max_chains.cyan = 3;
    max_chains.magenta = 2;
    max_chains.yellow = 4;
    
    // 新スコア計算関数を直接テスト
    let total_score = crate::scoring::calculate_line_clear_total_score(&board, 19, &max_chains);
    
    // 期待値: (3*3*10) + (2*2*10) + (2*4*10) = 90 + 40 + 80 = 210
    assert_eq!(total_score, 210);
}

/// 追加テスト: 空ライン・部分ラインでのスコア計算
#[test]
fn test_new_score_calculation_edge_cases() {
    let mut max_chains = crate::scoring::ColorMaxChains::new();
    max_chains.cyan = 2;
    
    // 空ボードでのスコア計算
    let empty_board: Board = vec![vec![Cell::Empty; 10]; 20];
    let score_empty = crate::scoring::calculate_line_clear_total_score(&empty_board, 19, &max_chains);
    assert_eq!(score_empty, 0);
    
    // 部分ラインでのスコア計算
    let mut partial_board: Board = vec![vec![Cell::Empty; 10]; 20];
    partial_board[19][0] = Cell::Occupied(GameColor::Cyan);
    partial_board[19][1] = Cell::Occupied(GameColor::Cyan);
    let score_partial = crate::scoring::calculate_line_clear_total_score(&partial_board, 19, &max_chains);
    assert_eq!(score_partial, 40); // 2 * 2 * 10 = 40
}