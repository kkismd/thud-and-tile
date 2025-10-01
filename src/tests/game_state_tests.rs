use super::*;

#[test]
fn test_max_chain_initial_state() {
    let state = GameState::new();
    assert_eq!(*state.custom_score.max_chains.get(&Color::Cyan).unwrap(), 0);
    assert_eq!(*state.custom_score.max_chains.get(&Color::Magenta).unwrap(), 0);
    assert_eq!(*state.custom_score.max_chains.get(&Color::Yellow).unwrap(), 0);
}

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
        state.board[BOARD_HEIGHT - 1][x] = Cell::Occupied(Color::Blue);
    }

    // Create a piece to lock and trigger the line clear
    let piece = Tetromino::from_shape(
        TetrominoShape::I,
        [Color::Red, Color::Red, Color::Red, Color::Red],
    );
    state.current_piece = Some(piece);

    state.lock_piece(&time_provider);

    assert!(
        state
            .animation
            .iter()
            .any(|anim| matches!(anim, Animation::LineBlink { .. }))
    );
}

/*
#[test]
fn test_bottom_line_is_cleared_normally() {
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    // Create a full line at the bottom
    for x in 0..BOARD_WIDTH {
        state.board[BOARD_HEIGHT - 1][x] = Cell::Occupied(Color::Blue);
    }
    // Add a marker block on the row above
    state.board[BOARD_HEIGHT - 2][0] = Cell::Occupied(Color::Red);

    // Clear the bottom line
    let new_animations = state.clear_lines(&[BOARD_HEIGHT - 1], &time_provider);
    state.animation.extend(new_animations);

    // Assert that the marker block has moved down into the bottom row
    assert_eq!(state.board[BOARD_HEIGHT - 1][0], Cell::Occupied(Color::Red));
    // Assert that the top row is now empty
    assert!(state.board[0].iter().all(|&c| c == Cell::Empty));
    // Assert score and line count
    assert_eq!(state.lines_cleared, 1);
    assert_eq!(state.score, 100);
}
*/

#[test]
fn test_cleared_non_bottom_line_turns_gray() {
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    let clear_line_y = BOARD_HEIGHT - 5;

    // Create a full line at a non-bottom row
    for x in 0..BOARD_WIDTH {
        state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
    }

    // Call the line clear logic
    let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
    state.animation.extend(new_animations);

    // Assert that the cleared line has turned gray
    for x in 0..BOARD_WIDTH {
        assert_eq!(state.board[clear_line_y][x], Cell::Occupied(Color::Grey));
    }
}

#[test]
fn test_non_bottom_clear_triggers_pushdown() {
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    let clear_line_y = BOARD_HEIGHT - 5;

    // Create a full line at a non-bottom row
    for x in 0..BOARD_WIDTH {
        state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
    }

    // Call the line clear logic and capture the resulting animations
    let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
    state.animation.extend(new_animations);

    assert!(state.animation.iter().any(|anim| matches!(
        anim,
        Animation::PushDown { gray_line_y, .. } if *gray_line_y == clear_line_y
    )));
}

/*
#[test]
fn test_scoring_after_pushdown() {
    let mut state = GameState::new();
    let clear_line_y = BOARD_HEIGHT - 5;

    // Setup a 2x2 group of green blocks below the clear line
    let green_group = [
        (2, clear_line_y + 2),
        (3, clear_line_y + 2),
        (2, clear_line_y + 3),
        (3, clear_line_y + 3),
    ];
    for &(x, y) in &green_group {
        state.board[y][x] = Cell::Occupied(Color::Green);
    }

    // The `blocks_to_score` is populated by `clear_lines`
    state.blocks_to_score = board_logic::count_connected_blocks(&state.board, clear_line_y);
    assert_eq!(state.blocks_to_score.len(), 4); // Sanity check

    // Manually call the scoring logic
    board_logic::handle_scoring(&mut state);

    // Each of the 4 blocks is in a component of size 4, so 4 * (4 * 10) = 160
    assert_eq!(state.score, 160);
    // The scoring list should be cleared after processing
    assert!(state.blocks_to_score.is_empty());
}
*/

/*
#[test]
fn test_handle_animation_processes_line_blink() {
    let mut time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    let clear_line_y = BOARD_HEIGHT - 1;
    for x in 0..BOARD_WIDTH {
        state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
    }

    state.animation.push(Animation::LineBlink {
        lines: vec![clear_line_y],
        count: 0,
        start_time: time_provider.now(),
    });

    // Advance time past the blink animation step
    time_provider.advance(BLINK_ANIMATION_STEP * BLINK_COUNT_MAX as u32);

    // Call handle_animation
    handle_animation(&mut state, &time_provider);

    // After blinking, clear_lines should be called, which will either spawn a new piece
    // or add PushDown animations. In this case, it's a bottom line, so it's should spawn a new piece.
    // We can assert that the animation queue is empty and a new piece is spawned.
    assert!(state.animation.is_empty());
    assert!(state.current_piece.is_some());
    assert_eq!(state.lines_cleared, 1);
    assert_eq!(state.score, 100);
}
*/

#[test]
fn test_handle_animation_processes_push_down() {
    let mut time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    let clear_line_y = BOARD_HEIGHT - 5;
    for x in 0..BOARD_WIDTH {
        state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
    }

    // Trigger a PushDown animation
    let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
    state.animation.extend(new_animations);

    // Ensure there's a PushDown animation
    assert!(
        state
            .animation
            .iter()
            .any(|anim| matches!(anim, Animation::PushDown { .. }))
    );

    // Advance time to trigger the push down step
    time_provider.advance(PUSH_DOWN_STEP_DURATION);
    handle_animation(&mut state, &time_provider);

    // Assert that the gray line has moved down one step
    assert_eq!(
        state.board[clear_line_y + 1][0],
        Cell::Occupied(Color::Grey)
    );
    assert_eq!(state.board[clear_line_y][0], Cell::Empty);

    // Assert that the animation is still ongoing (unless it reached the bottom)
    assert!(!state.animation.is_empty());
}

#[test]
fn test_solid_cell_is_collision() {
    let mut state = GameState::new();
    let solid_pos = (4, 5);
    state.board[solid_pos.1][solid_pos.0] = Cell::Solid;

    let mut piece = Tetromino::from_shape(TetrominoShape::I, [Color::Red; 4]);
    // Position the piece to overlap with the solid cell
    piece.pos = (solid_pos.0 as i8 - 1, solid_pos.1 as i8 - 1);

    assert!(!state.is_valid_position(&piece));
}

#[test]
fn test_pushdown_finishes_with_solid_line() {
    let mut time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    let clear_line_y = BOARD_HEIGHT - 2; // Clear line near bottom

    // Create a full line at a non-bottom row
    for x in 0..BOARD_WIDTH {
        state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
    }

    // Trigger the line clear and subsequent pushdown animation
    let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
    state.animation.extend(new_animations);

    // Loop until the animation is complete
    while !state.animation.is_empty() {
        time_provider.advance(PUSH_DOWN_STEP_DURATION);
        handle_animation(&mut state, &time_provider);
    }

    // Assert that the bottom row is now solid
    for x in 0..BOARD_WIDTH {
        assert_eq!(state.board[BOARD_HEIGHT - 1][x], Cell::Solid);
    }
}

/*
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
        state.board[BOARD_HEIGHT - 2][x] = Cell::Occupied(Color::Blue);
    }

    // Create a piece to lock and trigger the line clear
    let piece = Tetromino::from_shape(
        TetrominoShape::I,
        [Color::Red, Color::Red, Color::Red, Color::Red],
    );
    state.current_piece = Some(piece);

    state.lock_piece(&time_provider);

    // Manually advance the blink animation to completion
    time_provider.advance(BLINK_ANIMATION_STEP * BLINK_COUNT_MAX as u32);
    handle_animation(&mut state, &time_provider); // Line clear should now have happened

    // Assert that the solid line remains
    for x in 0..BOARD_WIDTH {
        assert_eq!(state.board[BOARD_HEIGHT - 1][x], Cell::Solid);
    }
    // Assert that the occupied line turned gray and triggered PushDown animation
    for x in 0..BOARD_WIDTH {
        assert_eq!(
            state.board[BOARD_HEIGHT - 2][x],
            Cell::Occupied(Color::Grey)
        );
    }
    let expected_gray_line_y = BOARD_HEIGHT - 2;
    assert!(state.animation.iter().any(|anim| {
        if let Animation::PushDown { gray_line_y: y, .. } = anim {
            *y == expected_gray_line_y
        } else {
            false
        }
    }));
    // Assert score and line count (no score yet, as PushDown animation is ongoing)
    assert_eq!(state.lines_cleared, 0);
    assert_eq!(state.score, 0);
}
*/

#[test]
fn test_pushdown_animation_moves_line() {
    // Setup: Time provider and initial state
    let mut time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    let clear_line_y = BOARD_HEIGHT - 5;

    // Create a full line
    for x in 0..BOARD_WIDTH {
        state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
    }

    // Trigger the animation
    let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
    state.animation.extend(new_animations);

    // Advance time and handle animation for one step
    time_provider.advance(PUSH_DOWN_STEP_DURATION);
    handle_animation(&mut state, &time_provider);

    // Assert: The gray line has moved down one step
    assert_eq!(
        state.board[clear_line_y + 1][0],
        Cell::Occupied(Color::Grey),
        "Gray line should have moved down"
    );
    assert_eq!(
        state.board[clear_line_y][0],
        Cell::Empty,
        "Original gray line row should be empty"
    );
}

#[test]
fn test_multiple_gray_lines_stack_and_reduce_board_height() {
    let mut time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    // 1. Clear a line at BOARD_HEIGHT - 5
    let clear_line_y1 = BOARD_HEIGHT - 5;
    for x in 0..BOARD_WIDTH {
        state.board[clear_line_y1][x] = Cell::Occupied(Color::Blue);
    }
    let new_animations = state.clear_lines(&[clear_line_y1], &time_provider);
    state.animation.extend(new_animations);

    // Loop until the first animation is complete
    while !state.animation.is_empty() {
        time_provider.advance(PUSH_DOWN_STEP_DURATION);
        handle_animation(&mut state, &time_provider);
    }

    // Assert first gray line is solid and board height reduced
    for x in 0..BOARD_WIDTH {
        assert_eq!(
            state.board[BOARD_HEIGHT - 1][x],
            Cell::Solid,
            "First gray line should be solid"
        );
    }
    assert_eq!(
        state.current_board_height,
        BOARD_HEIGHT - 1,
        "Board height should be reduced by 1 after first clear"
    );

    // 2. Clear a second line at a higher position
    let clear_line_y2 = BOARD_HEIGHT - 10;
    for x in 0..BOARD_WIDTH {
        state.board[clear_line_y2][x] = Cell::Occupied(Color::Green);
    }
    let new_animations = state.clear_lines(&[clear_line_y2], &time_provider);
    state.animation.extend(new_animations);

    // Loop until the second animation is complete
    while !state.animation.is_empty() {
        time_provider.advance(PUSH_DOWN_STEP_DURATION);
        handle_animation(&mut state, &time_provider);
    }

    // Assert second gray line is solid and board height reduced further
    // It should settle on top of the first solid line, at BOARD_HEIGHT - 2
    for x in 0..BOARD_WIDTH {
        assert_eq!(
            state.board[BOARD_HEIGHT - 2][x],
            Cell::Solid,
            "Second gray line should be solid on top of the first"
        );
    }
    assert_eq!(
        state.current_board_height,
        BOARD_HEIGHT - 2,
        "Board height should be reduced by 2 after second clear"
    );

    // Verify that new pieces would spawn above the solid lines
    state.spawn_piece(); // A new piece should have spawned automatically
    assert!(
        state.current_piece.is_some(),
        "A new piece should spawn after animations"
    );

    // Position a test piece to overlap with the solid lines
    let mut colliding_piece = Tetromino::from_shape(TetrominoShape::I, [Color::Red; 4]);
    colliding_piece.pos = (0, (state.current_board_height as i8) - 1); // Place it on the top solid line
    assert!(
        !state.is_valid_position(&colliding_piece),
        "Piece should not be valid on solid lines"
    );
}
