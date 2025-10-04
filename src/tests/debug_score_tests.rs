use super::*;

#[test]
fn debug_score_calculation() {
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    // MAX-CHAINを手動で設定
    state
        .custom_score_system
        .max_chains
        .update_max(GameColor::Cyan, 2);
    state
        .custom_score_system
        .max_chains
        .update_max(GameColor::Magenta, 3);
    state
        .custom_score_system
        .max_chains
        .update_max(GameColor::Yellow, 1);

    // 底辺ラインにシンプルなブロックを配置
    let line_y = BOARD_HEIGHT - 1;
    state.board[line_y] = vec![
        Cell::Occupied(GameColor::Cyan),    // 1 × 2 × 10 = 20
        Cell::Occupied(GameColor::Magenta), // 1 × 3 × 10 = 30
        Cell::Occupied(GameColor::Yellow),  // 1 × 1 × 10 = 10
        Cell::Occupied(GameColor::Cyan),    // 1 × 2 × 10 = 20
        Cell::Occupied(GameColor::Magenta), // 1 × 3 × 10 = 30
        Cell::Occupied(GameColor::Yellow),  // 1 × 1 × 10 = 10
        Cell::Occupied(GameColor::Cyan),    // 1 × 2 × 10 = 20
        Cell::Occupied(GameColor::Magenta), // 1 × 3 × 10 = 30
        Cell::Occupied(GameColor::Yellow),  // 1 × 1 × 10 = 10
        Cell::Occupied(GameColor::Cyan),    // 1 × 2 × 10 = 20
    ];

    // ライン消去を実行
    state.clear_lines(&[line_y], &time_provider);

    // 期待値：
    // Cyan: 4 blocks × 2 MAX-CHAIN × 10 = 80
    // Magenta: 3 blocks × 3 MAX-CHAIN × 10 = 90
    // Yellow: 3 blocks × 1 MAX-CHAIN × 10 = 30
    // Total: 200

    assert_eq!(state.custom_score_system.scores.cyan, 80);
    assert_eq!(state.custom_score_system.scores.magenta, 90);
    assert_eq!(state.custom_score_system.scores.yellow, 30);
    assert_eq!(state.custom_score_system.scores.total(), 200);
}

#[test]
fn debug_connected_blocks_score_calculation() {
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    println!("=== Testing Connected blocks ===");

    // MAX-CHAINを手動で設定
    state
        .custom_score_system
        .max_chains
        .update_max(GameColor::Cyan, 2);
    state
        .custom_score_system
        .max_chains
        .update_max(GameColor::Magenta, 3);
    state
        .custom_score_system
        .max_chains
        .update_max(GameColor::Yellow, 1);

    // 底辺ラインにConnectedブロックを配置
    let line_y = BOARD_HEIGHT - 1;
    state.board[line_y] = vec![
        Cell::Connected {
            color: GameColor::Cyan,
            count: 3,
        }, // 3 × 2 × 10 = 60
        Cell::Connected {
            color: GameColor::Magenta,
            count: 2,
        }, // 2 × 3 × 10 = 60
        Cell::Connected {
            color: GameColor::Yellow,
            count: 5,
        }, // 5 × 1 × 10 = 50
        Cell::Connected {
            color: GameColor::Cyan,
            count: 1,
        }, // 1 × 2 × 10 = 20
        Cell::Connected {
            color: GameColor::Magenta,
            count: 4,
        }, // 4 × 3 × 10 = 120
        Cell::Connected {
            color: GameColor::Yellow,
            count: 7,
        }, // 7 × 1 × 10 = 70
        Cell::Connected {
            color: GameColor::Cyan,
            count: 2,
        }, // 2 × 2 × 10 = 40
        Cell::Connected {
            color: GameColor::Magenta,
            count: 1,
        }, // 1 × 3 × 10 = 30
        Cell::Connected {
            color: GameColor::Yellow,
            count: 3,
        }, // 3 × 1 × 10 = 30
        Cell::Connected {
            color: GameColor::Cyan,
            count: 6,
        }, // 6 × 2 × 10 = 120
    ];

    println!("\n=== Before line clear ===");
    println!("Cyan Score: {}", state.custom_score_system.scores.cyan);
    println!(
        "Magenta Score: {}",
        state.custom_score_system.scores.magenta
    );
    println!("Yellow Score: {}", state.custom_score_system.scores.yellow);
    println!("Total Score: {}", state.custom_score_system.scores.total());

    // ライン消去を実行
    state.clear_lines(&[line_y], &time_provider);

    println!("\n=== After line clear ===");
    println!("Cyan Score: {}", state.custom_score_system.scores.cyan);
    println!(
        "Magenta Score: {}",
        state.custom_score_system.scores.magenta
    );
    println!("Yellow Score: {}", state.custom_score_system.scores.yellow);
    println!("Total Score: {}", state.custom_score_system.scores.total());

    // 期待値：
    // Cyan: (3+1+2+6) × 2 × 10 = 12 × 2 × 10 = 240
    // Magenta: (2+4+1) × 3 × 10 = 7 × 3 × 10 = 210
    // Yellow: (5+7+3) × 1 × 10 = 15 × 1 × 10 = 150
    // Total: 600

    let expected_cyan = 240;
    let expected_magenta = 210;
    let expected_yellow = 150;
    let expected_total = 600;

    println!("\n=== Expected vs Actual ===");
    println!(
        "Expected Cyan: {}, Actual: {}",
        expected_cyan, state.custom_score_system.scores.cyan
    );
    println!(
        "Expected Magenta: {}, Actual: {}",
        expected_magenta, state.custom_score_system.scores.magenta
    );
    println!(
        "Expected Yellow: {}, Actual: {}",
        expected_yellow, state.custom_score_system.scores.yellow
    );
    println!(
        "Expected Total: {}, Actual: {}",
        expected_total,
        state.custom_score_system.scores.total()
    );

    assert_eq!(state.custom_score_system.scores.cyan, expected_cyan);
    assert_eq!(state.custom_score_system.scores.magenta, expected_magenta);
    assert_eq!(state.custom_score_system.scores.yellow, expected_yellow);
    assert_eq!(state.custom_score_system.scores.total(), expected_total);
}

// Phase 3-1: calculate_line_clear_total_score関数のTDDテスト
#[test]
fn test_calculate_line_clear_total_score_basic() {
    // RED: まだ実装されていない関数のテスト
    use crate::scoring::{calculate_line_clear_total_score, ColorMaxChains};

    let board = create_test_board_with_line();
    let max_chains = ColorMaxChains {
        cyan: 2,
        magenta: 3,
        yellow: 1,
        chain_bonus: 0,
    };

    let total_score = calculate_line_clear_total_score(&board, 19, &max_chains);
    assert_eq!(total_score, 200); // Cyan:4×2×10 + Magenta:3×3×10 + Yellow:3×1×10 = 200
}

fn create_test_board_with_line() -> Vec<Vec<Cell>> {
    let mut board = vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
    let line_y = 19;
    board[line_y] = vec![
        Cell::Occupied(GameColor::Cyan),    // 1
        Cell::Occupied(GameColor::Magenta), // 2
        Cell::Occupied(GameColor::Yellow),  // 3
        Cell::Occupied(GameColor::Cyan),    // 4
        Cell::Occupied(GameColor::Magenta), // 5
        Cell::Occupied(GameColor::Yellow),  // 6
        Cell::Occupied(GameColor::Cyan),    // 7
        Cell::Occupied(GameColor::Magenta), // 8
        Cell::Occupied(GameColor::Yellow),  // 9
        Cell::Occupied(GameColor::Cyan),    // 10
    ];
    board
}

// Phase 3-2: Cell種類別スコア計算のテスト
#[test]
fn test_calculate_line_clear_total_score_connected() {
    // RED: Connected型セル対応テスト
    use crate::scoring::{calculate_line_clear_total_score, ColorMaxChains};

    let mut board = vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
    board[19][0] = Cell::Connected { color: GameColor::Cyan, count: 3 };
    board[19][1] = Cell::Connected { color: GameColor::Yellow, count: 5 };
    
    let max_chains = ColorMaxChains {
        cyan: 2,
        magenta: 1,
        yellow: 4,
        chain_bonus: 0,
    };
    
    let total_score = calculate_line_clear_total_score(&board, 19, &max_chains);
    assert_eq!(total_score, 260); // (3*2*10) + (5*4*10) = 260
}
