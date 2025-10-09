use super::*;

// 実際のゲームフロー（lock_piece → アニメーション → handle_animation）をテストするファイル
// clear_lines()を直接呼び出す古いテストは削除し、必要最小限のテストのみ保持

#[test]
fn test_game_starts_in_title_mode() {
    let state = GameState::new();
    assert_eq!(state.mode, GameMode::Title);
}

#[test]
fn test_line_clear_triggers_blink_animation() {
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    // Create a full line at the bottom
    for x in 0..BOARD_WIDTH {
        state.board[BOARD_HEIGHT - 1][x] = Cell::Occupied(GameColor::Blue);
    }

    // Create a piece to lock and trigger the line clear
    let piece = Tetromino::from_shape(
        TetrominoShape::I,
        [
            GameColor::Red,
            GameColor::Red,
            GameColor::Red,
            GameColor::Red,
        ],
    );
    state.current_piece = Some(piece);

    state.lock_piece(&time_provider);

    assert!(state
        .animation
        .iter()
        .any(|anim| matches!(anim, Animation::LineBlink { .. })));
}

#[test]
fn test_lock_piece_ignores_solid_lines() {
    let mut time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    // Create a solid line at the bottom
    for x in 0..BOARD_WIDTH {
        state.board[BOARD_HEIGHT - 1][x] = Cell::Solid;
    }
    // Create an occupied line above it
    for x in 0..BOARD_WIDTH {
        state.board[BOARD_HEIGHT - 2][x] = Cell::Occupied(GameColor::Blue);
    }

    // Create a piece to lock and trigger the line clear
    let mut piece = Tetromino::from_shape(
        TetrominoShape::I,
        [
            GameColor::Red,
            GameColor::Red,
            GameColor::Red,
            GameColor::Red,
        ],
    );
    // Position the piece so it will land at the second-to-bottom row
    piece.pos = (1, (BOARD_HEIGHT - 3) as i8);
    state.current_piece = Some(piece);

    state.lock_piece(&time_provider);

    // Manually advance the blink animation to completion
    time_provider.advance(BLINK_ANIMATION_STEP * BLINK_COUNT_MAX as u32);
    handle_animation(&mut state, &time_provider); // Line clear should now have happened

    // Assert that the solid line remains
    for x in 0..BOARD_WIDTH {
        assert_eq!(state.board[BOARD_HEIGHT - 1][x], Cell::Solid);
    }
    // Assert that the occupied line turned Solid and triggered PushDown animation
    for x in 0..BOARD_WIDTH {
        assert_eq!(state.board[BOARD_HEIGHT - 2][x], Cell::Solid);
    }
    let expected_gray_line_y = BOARD_HEIGHT - 2;
    assert!(state.animation.iter().any(|anim| {
        if let Animation::PushDown { gray_line_y: y, .. } = anim {
            *y == expected_gray_line_y
        } else {
            false
        }
    }));
    // Assert custom score (no score yet, as PushDown animation is ongoing)
    assert_eq!(
        state.custom_score_system.scores.total(),
        0,
        "No custom score during animation"
    );
}

#[test]
fn test_solid_cell_is_collision() {
    let mut state = GameState::new();
    let solid_pos = (4, 5);
    state.board[solid_pos.1][solid_pos.0] = Cell::Solid;

    let mut piece = Tetromino::from_shape(TetrominoShape::I, [GameColor::Red; 4]);
    // Position the piece to overlap with the solid cell
    piece.pos = (solid_pos.0 as i8 - 1, solid_pos.1 as i8 - 1);

    assert!(!state.is_valid_position(&piece));
}

#[test]
fn test_max_chain_updated_after_piece_landing() {
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    // Create a test scenario with connected blocks
    // Place some cyan blocks in a connected pattern
    state.board[BOARD_HEIGHT - 2][0] = Cell::Occupied(GameColor::Cyan);
    state.board[BOARD_HEIGHT - 2][1] = Cell::Occupied(GameColor::Cyan);
    state.board[BOARD_HEIGHT - 3][0] = Cell::Occupied(GameColor::Cyan);

    // Place some magenta blocks in a larger connected pattern
    for x in 3..=6 {
        state.board[BOARD_HEIGHT - 2][x] = Cell::Occupied(GameColor::Magenta);
    }
    state.board[BOARD_HEIGHT - 3][3] = Cell::Occupied(GameColor::Magenta);

    // Place some yellow blocks in an even larger pattern
    for x in 7..=9 {
        state.board[BOARD_HEIGHT - 2][x] = Cell::Occupied(GameColor::Yellow);
        state.board[BOARD_HEIGHT - 3][x] = Cell::Occupied(GameColor::Yellow);
    }

    // Initially, the custom score system should have zero max chains
    assert_eq!(state.custom_score_system.max_chains.get(GameColor::Cyan), 0);
    assert_eq!(
        state.custom_score_system.max_chains.get(GameColor::Magenta),
        0
    );
    assert_eq!(
        state.custom_score_system.max_chains.get(GameColor::Yellow),
        0
    );

    // Create a piece and place it to trigger a landing
    let piece = Tetromino::from_shape(
        TetrominoShape::I,
        [
            GameColor::Cyan,
            GameColor::Cyan,
            GameColor::Cyan,
            GameColor::Cyan,
        ],
    );
    state.current_piece = Some(piece);

    // Lock the piece to trigger connected block analysis and max chain update
    state.lock_piece(&time_provider);

    // After landing, the max chains should be updated based on connected block counts
    // The I-piece (4 cyan blocks) should connect with the existing 3 cyan blocks,
    // but only if they are adjacent. Let's check actual results.
    // Cyan: The I-piece likely connects some blocks, creating a larger group
    // Magenta: 5 blocks connected (4 horizontal + 1 extending up)
    // Yellow: 6 blocks connected (3x2 grid)
    assert_eq!(
        state.custom_score_system.max_chains.get(GameColor::Cyan),
        4,
        "Cyan should have 4 connected blocks after I-piece lands"
    );
    assert_eq!(
        state.custom_score_system.max_chains.get(GameColor::Magenta),
        5,
        "Magenta should have 5 connected blocks"
    );
    assert_eq!(
        state.custom_score_system.max_chains.get(GameColor::Yellow),
        6,
        "Yellow should have 6 connected blocks"
    );
    assert_eq!(state.custom_score_system.max_chains.max(), 6);
}

#[test]
fn test_max_chain_only_increases_never_decreases() {
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    // Set initial max chains to some values
    state
        .custom_score_system
        .max_chains
        .update_max(GameColor::Cyan, 8);
    state
        .custom_score_system
        .max_chains
        .update_max(GameColor::Magenta, 10);
    state
        .custom_score_system
        .max_chains
        .update_max(GameColor::Yellow, 5);

    // Create a smaller connected pattern
    state.board[BOARD_HEIGHT - 2][0] = Cell::Occupied(GameColor::Cyan);
    state.board[BOARD_HEIGHT - 2][1] = Cell::Occupied(GameColor::Cyan);

    // Place a piece
    let piece = Tetromino::from_shape(
        TetrominoShape::O,
        [
            GameColor::Cyan,
            GameColor::Cyan,
            GameColor::Cyan,
            GameColor::Cyan,
        ],
    );
    state.current_piece = Some(piece);

    // Lock the piece
    state.lock_piece(&time_provider);

    // Max chains should not decrease even if current connected count is smaller
    assert_eq!(state.custom_score_system.max_chains.get(GameColor::Cyan), 8); // Should remain 8, not decrease to smaller count
    assert_eq!(
        state.custom_score_system.max_chains.get(GameColor::Magenta),
        10
    ); // Should remain unchanged
    assert_eq!(
        state.custom_score_system.max_chains.get(GameColor::Yellow),
        5
    ); // Should remain unchanged
}
