use super::*;

#[test]
fn test_isolated_blocks_are_removed_on_non_bottom_clear() {
    // This test verifies the board_logic::remove_isolated_blocks function directly
    let mut board = vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];

    let cleared_line_y = BOARD_HEIGHT - 5;

    // Create a full line that will be "cleared"
    for x in 0..BOARD_WIDTH {
        board[cleared_line_y][x] = Cell::Occupied(GameColor::Blue);
    }

    // Place an isolated single block below the cleared line
    board[cleared_line_y + 2][5] = Cell::Occupied(GameColor::Red);

    // Place a connected pair of blocks below the cleared line
    board[cleared_line_y + 3][2] = Cell::Occupied(GameColor::Green);
    board[cleared_line_y + 3][3] = Cell::Occupied(GameColor::Green);

    // Call the remove_isolated_blocks function
    board_logic::remove_isolated_blocks(&mut board, cleared_line_y);

    // The isolated red block should be removed
    assert_eq!(
        board[cleared_line_y + 2][5],
        Cell::Empty,
        "Isolated red block should be removed"
    );

    // The connected green blocks should remain
    assert_ne!(
        board[cleared_line_y + 3][2],
        Cell::Empty,
        "Connected green block should remain"
    );
    assert_ne!(
        board[cleared_line_y + 3][3],
        Cell::Empty,
        "Connected green block should remain"
    );
}

#[test]
fn test_counts_connected_blocks() {
    let mut board = vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
    let cleared_line_y = 15;

    // Setup a 2x2 group of green blocks
    let green_group = [
        (2, cleared_line_y + 2),
        (3, cleared_line_y + 2),
        (2, cleared_line_y + 3),
        (3, cleared_line_y + 3),
    ];
    for &(x, y) in &green_group {
        board[y][x] = Cell::Occupied(GameColor::Green);
    }

    // Setup a single isolated red block
    let red_block = (7, cleared_line_y + 1);
    board[red_block.1][red_block.0] = Cell::Occupied(GameColor::Red);

    let mut results = board_logic::count_connected_blocks(&board, cleared_line_y);
    results.sort_by_key(|k| (k.0 .1, k.0 .0)); // Sort for consistent order

    let mut expected = vec![
        (red_block, 1),
        (green_group[0], 4),
        (green_group[1], 4),
        (green_group[2], 4),
        (green_group[3], 4),
    ];
    expected.sort_by_key(|k| (k.0 .1, k.0 .0));

    assert_eq!(results, expected);
}

#[test]
fn test_newly_landed_block_connects_to_existing_connected_block() {
    let mut board = vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
    let test_color = GameColor::Red;

    // 既存のConnectedブロックを配置
    board[BOARD_HEIGHT - 1][0] = Cell::Connected {
        color: test_color,
        count: 1,
    };

    // その隣に同じ色の新しく着地したOccupiedブロックを配置
    board[BOARD_HEIGHT - 1][1] = Cell::Occupied(test_color);

    // 接続を試みる関数を呼び出す
    board_logic::find_and_connect_adjacent_blocks(&mut board, &[]);

    // 新しく着地したブロックがConnectedになっていることをアサート
    assert_eq!(
        board[BOARD_HEIGHT - 1][1],
        Cell::Connected {
            color: test_color,
            count: 1
        },
        "既存のConnectedブロックの隣に同じ色のブロックが着地した場合、新しく着地したブロックもConnectedになるべきです"
    );
}

#[test]
fn test_connected_blocks_count_after_lock_piece() {
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    let test_color = GameColor::Red;

    // Scenario 1: Two adjacent blocks
    // Place two adjacent blocks at the bottom
    state.board[BOARD_HEIGHT - 1][0] = Cell::Occupied(test_color);
    state.board[BOARD_HEIGHT - 1][1] = Cell::Occupied(test_color);

    // Lock a dummy piece to trigger find_and_connect_adjacent_blocks
    // (This will be replaced by actual piece locking in Green phase)
    let dummy_piece = Tetromino::from_shape(TetrominoShape::I, [test_color; 4]);
    state.current_piece = Some(dummy_piece);
    state.lock_piece(&time_provider);

    // Assert that the two blocks are now Connected and have count 2
    if let Cell::Connected { color, count } = state.board[BOARD_HEIGHT - 1][0] {
        assert_eq!(color, test_color);
        assert_eq!(count, 2, "Expected count 2 for adjacent blocks");
    } else {
        panic!("Block at [{}, {}] is not Connected", BOARD_HEIGHT - 1, 0);
    }
    if let Cell::Connected { color, count } = state.board[BOARD_HEIGHT - 1][1] {
        assert_eq!(color, test_color);
        assert_eq!(count, 2, "Expected count 2 for adjacent blocks");
    } else {
        panic!("Block at [{}, {}] is not Connected", BOARD_HEIGHT - 1, 1);
    }

    // Reset board for next scenario
    state.board = vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];

    // Scenario 2: Multiple connected blocks (e.g., 2x2 square)
    let square_color = GameColor::Green;
    state.board[BOARD_HEIGHT - 2][0] = Cell::Occupied(square_color);
    state.board[BOARD_HEIGHT - 2][1] = Cell::Occupied(square_color);
    state.board[BOARD_HEIGHT - 1][0] = Cell::Occupied(square_color);
    state.board[BOARD_HEIGHT - 1][1] = Cell::Occupied(square_color);

    let dummy_piece_2 = Tetromino::from_shape(TetrominoShape::O, [square_color; 4]);
    state.current_piece = Some(dummy_piece_2);
    state.lock_piece(&time_provider);

    // Assert that all four blocks are Connected and have count 4
    let positions = [
        (BOARD_HEIGHT - 2, 0),
        (BOARD_HEIGHT - 2, 1),
        (BOARD_HEIGHT - 1, 0),
        (BOARD_HEIGHT - 1, 1),
    ];
    for &(y, x) in &positions {
        if let Cell::Connected { color, count } = state.board[y][x] {
            assert_eq!(color, square_color);
            assert_eq!(
                count, 4,
                "Expected count 4 for 2x2 square at [{}, {}]",
                y, x
            );
        } else {
            panic!("Block at [{}, {}] is not Connected", y, x);
        }
    }

    // Reset board for next scenario
    state.board = vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];

    // Scenario 3: Single isolated block
    let isolated_color = GameColor::Blue;
    state.board[BOARD_HEIGHT - 1][5] = Cell::Occupied(isolated_color);

    let dummy_piece_3 = Tetromino::from_shape(TetrominoShape::I, [isolated_color; 4]);
    state.current_piece = Some(dummy_piece_3);
    state.lock_piece(&time_provider);

    // Assert that the isolated block is still Occupied or Connected with count 1 (if it was converted)
    // Currently, find_and_connect_adjacent_blocks only converts if component.len() > 1
    // So it should remain Occupied.
    if let Cell::Occupied(color) = state.board[BOARD_HEIGHT - 1][5] {
        assert_eq!(color, isolated_color);
    } else {
        panic!(
            "Isolated block at [{}, {}] should be Occupied",
            BOARD_HEIGHT - 1,
            5
        );
    }
}
