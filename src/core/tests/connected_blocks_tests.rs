//! 隣接ブロックConnected変換のテスト

use crate::core::board_logic::{find_connected_components, apply_connected_components};
use crate::core::game_state::CoreGameState;
use crate::cell::Cell;
use crate::game_color::GameColor;
use crate::config::{BOARD_WIDTH, BOARD_HEIGHT};

/// 空のボードを作成
fn create_empty_board() -> [[Cell; BOARD_WIDTH]; BOARD_HEIGHT] {
    [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT]
}

#[test]
fn test_single_block_remains_occupied() {
    // 単一ブロックは隣接がないのでConnectedにならない
    let mut board = create_empty_board();
    board[19][5] = Cell::Occupied(GameColor::Cyan);
    
    let components = find_connected_components(board, &[]);
    let result_board = apply_connected_components(board, &components);
    
    // 単一ブロックもコンポーネントとして検出されConnectedになる
    assert!(matches!(result_board[19][5], Cell::Connected { color: GameColor::Cyan, count: 1 }));
}

#[test]
fn test_horizontal_adjacent_blocks_become_connected() {
    // 水平隣接ブロックのテスト
    let mut board = create_empty_board();
    board[19][5] = Cell::Occupied(GameColor::Cyan);
    board[19][6] = Cell::Occupied(GameColor::Cyan);
    board[19][7] = Cell::Occupied(GameColor::Cyan);
    
    let components = find_connected_components(board, &[]);
    
    // 3つのブロックが1つのコンポーネントになる
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].size, 3);
    assert_eq!(components[0].color, GameColor::Cyan);
    
    let result_board = apply_connected_components(board, &components);
    
    // すべてのブロックがConnectedになり、正しいカウントが設定される
    assert!(matches!(result_board[19][5], Cell::Connected { color: GameColor::Cyan, .. }));
    assert!(matches!(result_board[19][6], Cell::Connected { color: GameColor::Cyan, .. }));
    assert!(matches!(result_board[19][7], Cell::Connected { color: GameColor::Cyan, .. }));
}

#[test]
fn test_vertical_adjacent_blocks_become_connected() {
    // 垂直隣接ブロックのテスト
    let mut board = create_empty_board();
    board[17][5] = Cell::Occupied(GameColor::Magenta);
    board[18][5] = Cell::Occupied(GameColor::Magenta);
    board[19][5] = Cell::Occupied(GameColor::Magenta);
    
    let components = find_connected_components(board, &[]);
    
    // 3つのブロックが1つのコンポーネントになる
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].size, 3);
    assert_eq!(components[0].color, GameColor::Magenta);
    
    let result_board = apply_connected_components(board, &components);
    
    assert!(matches!(result_board[17][5], Cell::Connected { color: GameColor::Magenta, .. }));
    assert!(matches!(result_board[18][5], Cell::Connected { color: GameColor::Magenta, .. }));
    assert!(matches!(result_board[19][5], Cell::Connected { color: GameColor::Magenta, .. }));
}

#[test]
fn test_different_colors_not_connected() {
    // 異なる色は隣接していてもConnectedにならない（別コンポーネント）
    let mut board = create_empty_board();
    board[19][5] = Cell::Occupied(GameColor::Cyan);
    board[19][6] = Cell::Occupied(GameColor::Magenta);  // 隣接だが異色
    
    let components = find_connected_components(board, &[]);
    
    // 2つの別々のコンポーネント
    assert_eq!(components.len(), 2);
    
    // 各コンポーネントのサイズは1
    for component in &components {
        assert_eq!(component.size, 1);
    }
}

#[test]
fn test_l_shaped_connected_blocks() {
    // L字型の隣接ブロック
    let mut board = create_empty_board();
    board[18][5] = Cell::Occupied(GameColor::Yellow);
    board[19][5] = Cell::Occupied(GameColor::Yellow);
    board[19][6] = Cell::Occupied(GameColor::Yellow);
    board[19][7] = Cell::Occupied(GameColor::Yellow);
    
    let components = find_connected_components(board, &[]);
    
    // 4つのブロックが1つのコンポーネント
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].size, 4);
    assert_eq!(components[0].color, GameColor::Yellow);
}

#[test]
fn test_exclude_lines_are_ignored() {
    // 除外ライン内のブロックは処理されない
    let mut board = create_empty_board();
    board[19][5] = Cell::Occupied(GameColor::Cyan);
    board[19][6] = Cell::Occupied(GameColor::Cyan);
    
    // ライン19を除外
    let components = find_connected_components(board, &[19]);
    
    // 除外ラインのブロックは検出されない
    assert_eq!(components.len(), 0);
}

#[test]
fn test_core_game_state_integration() {
    // CoreGameStateの統合テスト
    let mut state = CoreGameState::new();
    
    // 手動でボードにブロックを配置
    state.board[19][5] = Cell::Occupied(GameColor::Cyan);
    state.board[19][6] = Cell::Occupied(GameColor::Cyan);
    state.board[18][5] = Cell::Occupied(GameColor::Magenta);
    
    // 隣接ブロック処理を実行
    let components = find_connected_components(state.board, &[]);
    state = state.apply_connected_components(components);
    
    // Cyan: 2つのブロックがConnected
    assert!(matches!(state.board[19][5], Cell::Connected { color: GameColor::Cyan, count: 2 }));
    assert!(matches!(state.board[19][6], Cell::Connected { color: GameColor::Cyan, count: 2 }));
    
    // Magenta: 単一ブロック
    assert!(matches!(state.board[18][5], Cell::Connected { color: GameColor::Magenta, count: 1 }));
}