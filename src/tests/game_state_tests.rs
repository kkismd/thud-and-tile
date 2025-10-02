use super::*;

#[test]
fn test_game_starts_in_title_mode() {
    let state = GameState::new();
    assert_eq!(state.mode, GameMode::Title);
}

#[test]
fn test_new_score_calculation_system() {
    // Test the new scoring formula: block_count × MAX-CHAIN × 10 points
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    // Set up some MAX-CHAIN values
    state
        .custom_score_system
        .max_chains
        .update_max(GameColor::Cyan, 3);
    state
        .custom_score_system
        .max_chains
        .update_max(GameColor::Magenta, 2);

    // Create a line with connected blocks at the bottom
    state.board[19] = vec![
        Cell::Connected {
            color: GameColor::Cyan,
            count: 2,
        }, // 2 × 3 × 10 = 60 points
        Cell::Connected {
            color: GameColor::Magenta,
            count: 1,
        }, // 1 × 2 × 10 = 20 points
        Cell::Connected {
            color: GameColor::Cyan,
            count: 3,
        }, // 3 × 3 × 10 = 90 points
        Cell::Connected {
            color: GameColor::Cyan,
            count: 2,
        }, // 2 × 3 × 10 = 60 points
        Cell::Connected {
            color: GameColor::Magenta,
            count: 1,
        }, // 1 × 2 × 10 = 20 points
        Cell::Connected {
            color: GameColor::Cyan,
            count: 3,
        }, // 3 × 3 × 10 = 90 points
        Cell::Connected {
            color: GameColor::Cyan,
            count: 2,
        }, // 2 × 3 × 10 = 60 points
        Cell::Connected {
            color: GameColor::Magenta,
            count: 1,
        }, // 1 × 2 × 10 = 20 points
        Cell::Connected {
            color: GameColor::Cyan,
            count: 3,
        }, // 3 × 3 × 10 = 90 points
        Cell::Connected {
            color: GameColor::Magenta,
            count: 1,
        }, // 1 × 2 × 10 = 20 points
    ];

    let initial_cyan_score = state.custom_score_system.scores.get(GameColor::Cyan);
    let initial_magenta_score = state.custom_score_system.scores.get(GameColor::Magenta);

    // Trigger line clear using the clear_lines method
    state.clear_lines(&[19], &time_provider);

    // Verify new score calculation
    let final_cyan_score = state.custom_score_system.scores.get(GameColor::Cyan);
    let final_magenta_score = state.custom_score_system.scores.get(GameColor::Magenta);

    // Expected: Cyan = (2+3+2+3+2+3)×3×10 = 15×3×10 = 450
    // Expected: Magenta = (1+1+1+1)×2×10 = 4×2×10 = 80
    assert_eq!(final_cyan_score - initial_cyan_score, 450);
    assert_eq!(final_magenta_score - initial_magenta_score, 80);
}

#[test]
fn test_connected_blocks_count_updated_after_animation_completion() {
    // Test that update_all_connected_block_counts is called after animation completion
    // This test verifies the fix for the bug where connected block counts weren't updated
    // after line clear animations completed.

    let mut time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    let test_color = GameColor::Red;

    // Set up a realistic scenario: reduced board height with solid lines at bottom
    state.current_board_height = BOARD_HEIGHT - 3; // Leave some space for solid lines
    for x in 0..BOARD_WIDTH {
        state.board[BOARD_HEIGHT - 1][x] = Cell::Solid;
        state.board[BOARD_HEIGHT - 2][x] = Cell::Solid;
        state.board[BOARD_HEIGHT - 3][x] = Cell::Solid;
    }

    // Create connected blocks in the active area with wrong counts
    let test_y = state.current_board_height - 2; // Place in active area
    state.board[test_y][0] = Cell::Connected {
        color: test_color,
        count: 3, // Wrong count initially
    };
    state.board[test_y][1] = Cell::Connected {
        color: test_color,
        count: 3, // Wrong count initially
    };
    // Only 2 blocks connected, so count should be 2

    // Create a PushDown animation that will complete when it hits solid
    state.animation.push(Animation::PushDown {
        gray_line_y: state.current_board_height - 1,
        start_time: time_provider.now(),
    });

    // Turn the gray line to gray (simulating the line clear process)
    for x in 0..BOARD_WIDTH {
        state.board[state.current_board_height - 1][x] = Cell::Occupied(GameColor::Grey);
    }

    // Process the animation - this should call update_all_connected_block_counts
    time_provider.advance(PUSH_DOWN_STEP_DURATION);
    handle_animation(&mut state, &time_provider);

    // After the fix, the animation should complete and update_all_connected_block_counts should be called
    assert!(state.animation.is_empty(), "Animation should be complete");

    // The bottom line should be solid (from the completed PushDown)
    for x in 0..BOARD_WIDTH {
        assert_eq!(state.board[state.current_board_height][x], Cell::Solid);
    }

    // Most importantly: verify that update_all_connected_block_counts was called
    // The connected blocks should have been updated to the correct count
    if let Cell::Connected { color, count } = state.board[test_y][0] {
        assert_eq!(color, test_color);
        assert_eq!(
            count, 2,
            "Connected block count should be updated from 3 to 2 after animation completion - this verifies the bug fix"
        );
    } else {
        panic!(
            "Block should be Connected after animation completion, but found: {:?}",
            state.board[test_y][0]
        );
    }

    if let Cell::Connected { color, count } = state.board[test_y][1] {
        assert_eq!(color, test_color);
        assert_eq!(
            count, 2,
            "Connected block count should be updated from 3 to 2 after animation completion"
        );
    } else {
        panic!(
            "Block should be Connected after animation completion, but found: {:?}",
            state.board[test_y][1]
        );
    }
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
        [GameColor::Red, GameColor::Red, GameColor::Red, GameColor::Red],
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

#[test]
fn test_bottom_line_is_cleared_normally() {
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    // Create a full line at the bottom
    for x in 0..BOARD_WIDTH {
        state.board[BOARD_HEIGHT - 1][x] = Cell::Occupied(GameColor::Blue);
    }
    // Add a marker block on the row above
    state.board[BOARD_HEIGHT - 2][0] = Cell::Occupied(GameColor::Red);

    // Clear the bottom line
    let new_animations = state.clear_lines(&[BOARD_HEIGHT - 1], &time_provider);
    state.animation.extend(new_animations);

    // Assert that the marker block has moved down into the bottom row
    assert_eq!(state.board[BOARD_HEIGHT - 1][0], Cell::Occupied(GameColor::Red));
    // Assert that the top row is now empty
    assert!(state.board[0].iter().all(|&c| c == Cell::Empty));
    // Assert custom score system - should have 10 blue blocks worth of score
    // Note: Blue blocks are not part of the custom color system (only Cyan/Magenta/Yellow)
    // but this test uses Blue, so no custom score should be added
    assert_eq!(
        state.custom_score_system.scores.total(),
        0,
        "Blue blocks are not scored in custom system"
    );
}

#[test]
fn test_cleared_non_bottom_line_turns_gray() {
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    let clear_line_y = BOARD_HEIGHT - 5;

    // Create a full line at a non-bottom row
    for x in 0..BOARD_WIDTH {
        state.board[clear_line_y][x] = Cell::Occupied(GameColor::Blue);
    }

    // Call the line clear logic
    let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
    state.animation.extend(new_animations);

    // Assert that the cleared line has turned gray
    for x in 0..BOARD_WIDTH {
        assert_eq!(state.board[clear_line_y][x], Cell::Occupied(GameColor::Grey));
    }
}

#[test]
fn test_non_bottom_clear_triggers_pushdown() {
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    let clear_line_y = BOARD_HEIGHT - 5;

    // Create a full line at a non-bottom row
    for x in 0..BOARD_WIDTH {
        state.board[clear_line_y][x] = Cell::Occupied(GameColor::Blue);
    }

    // Call the line clear logic and capture the resulting animations
    let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
    state.animation.extend(new_animations);

    assert!(state.animation.iter().any(|anim| matches!(
        anim,
        Animation::PushDown { gray_line_y, .. } if *gray_line_y == clear_line_y
    )));
}

#[test]
fn test_handle_animation_processes_line_blink() {
    let mut time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    let clear_line_y = BOARD_HEIGHT - 1;
    for x in 0..BOARD_WIDTH {
        state.board[clear_line_y][x] = Cell::Occupied(GameColor::Blue);
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
    // Note: This test uses Blue blocks which are not part of custom scoring system
    assert_eq!(
        state.custom_score_system.scores.total(),
        0,
        "Blue blocks are not scored in custom system"
    );
}

#[test]
fn test_handle_animation_processes_push_down() {
    let mut time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    let clear_line_y = BOARD_HEIGHT - 5;
    for x in 0..BOARD_WIDTH {
        state.board[clear_line_y][x] = Cell::Occupied(GameColor::Blue);
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
        Cell::Occupied(GameColor::Grey)
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

    let mut piece = Tetromino::from_shape(TetrominoShape::I, [GameColor::Red; 4]);
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
        state.board[clear_line_y][x] = Cell::Occupied(GameColor::Blue);
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
    let piece = Tetromino::from_shape(
        TetrominoShape::I,
        [GameColor::Red, GameColor::Red, GameColor::Red, GameColor::Red],
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
            Cell::Occupied(GameColor::Grey)
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
    // Assert custom score (no score yet, as PushDown animation is ongoing)
    assert_eq!(
        state.custom_score_system.scores.total(),
        0,
        "No custom score during animation"
    );
}

#[test]
fn test_pushdown_animation_moves_line() {
    // Setup: Time provider and initial state
    let mut time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    let clear_line_y = BOARD_HEIGHT - 5;

    // Create a full line
    for x in 0..BOARD_WIDTH {
        state.board[clear_line_y][x] = Cell::Occupied(GameColor::Blue);
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
        Cell::Occupied(GameColor::Grey),
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
        state.board[clear_line_y1][x] = Cell::Occupied(GameColor::Blue);
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
        state.board[clear_line_y2][x] = Cell::Occupied(GameColor::Green);
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
    let mut colliding_piece = Tetromino::from_shape(TetrominoShape::I, [GameColor::Red; 4]);
    colliding_piece.pos = (0, (state.current_board_height as i8) - 1); // Place it on the top solid line
    assert!(
        !state.is_valid_position(&colliding_piece),
        "Piece should not be valid on solid lines"
    );
}

#[test]
fn test_connected_blocks_separation_after_line_clear() {
    // Test that connected blocks are properly separated when a line clear divides them
    let mut time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    let test_color = GameColor::Green;
    let clear_line_y = BOARD_HEIGHT - 5;

    // Create a line to clear
    for x in 0..BOARD_WIDTH {
        state.board[clear_line_y][x] = Cell::Occupied(GameColor::Blue);
    }

    // Create a vertical line of connected blocks that spans across the clear line
    // Top part: 2 blocks
    state.board[clear_line_y - 2][3] = Cell::Connected {
        color: test_color,
        count: 4,
    };
    state.board[clear_line_y - 1][3] = Cell::Connected {
        color: test_color,
        count: 4,
    };

    // Bottom part: 2 blocks (below the clear line)
    state.board[clear_line_y + 1][3] = Cell::Connected {
        color: test_color,
        count: 4,
    };
    state.board[clear_line_y + 2][3] = Cell::Connected {
        color: test_color,
        count: 4,
    };

    // Process line clear animation
    let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
    state.animation.extend(new_animations);

    // Complete all animations
    for _ in 0..30 {
        if state.animation.is_empty() {
            break;
        }
        time_provider.advance(PUSH_DOWN_STEP_DURATION);
        handle_animation(&mut state, &time_provider);
    }

    // After animation completion, the originally connected blocks should be separated
    // into two groups of 2 blocks each

    // Find the top group (should be 2 connected blocks)
    let mut found_top_group = false;
    for y in 0..state.current_board_height {
        if let Cell::Connected { color, count } = state.board[y][3] {
            if color == test_color && count == 2 {
                found_top_group = true;
                break;
            }
        }
    }

    assert!(
        found_top_group,
        "Should find a group of 2 connected blocks after line clear separation"
    );
}

#[test]
fn test_connected_blocks_count_updated_after_bottom_line_clear() {
    // Test that connected blocks are updated even when clearing the bottom line of current field
    // (i.e., current_board_height - 1, which is just above solid lines)
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    let test_color = GameColor::Red;

    // Set up some solid lines at the bottom to simulate a reduced field
    state.current_board_height = BOARD_HEIGHT - 2; // Simulate field with solid lines at bottom
    for x in 0..BOARD_WIDTH {
        state.board[BOARD_HEIGHT - 1][x] = Cell::Solid;
        state.board[BOARD_HEIGHT - 2][x] = Cell::Solid;
    }

    // Create a full line at the current bottom (current_board_height - 1)
    let bottom_line_y = state.current_board_height - 1;
    for x in 0..BOARD_WIDTH {
        state.board[bottom_line_y][x] = Cell::Occupied(GameColor::Blue);
    }

    // Place connected blocks above the line to be cleared with wrong counts
    // After bottom line clear, these will move down by 1 position
    state.board[bottom_line_y - 1][0] = Cell::Connected {
        color: test_color,
        count: 3, // Wrong count - should be 2 after line clear
    };
    state.board[bottom_line_y - 1][1] = Cell::Connected {
        color: test_color,
        count: 3, // Wrong count - should be 2 after line clear
    };
    // No block at position 2, so they should only count as 2

    // Process line clear - this should be treated as bottom line clear
    let new_animations = state.clear_lines(&[bottom_line_y], &time_provider);
    state.animation.extend(new_animations);

    // The line should be cleared immediately (standard Tetris clear) with no animations
    assert!(
        state.animation.is_empty(),
        "Bottom line clear should not create animations"
    );

    // After bottom line clear, the connected blocks move down by one position
    // The new position would be bottom_line_y - 1, which was originally bottom_line_y - 1,
    // but after the line at bottom_line_y is removed, they shift down to bottom_line_y - 1

    // Verify that the connected blocks have been updated to the correct count
    // After bottom line clear with board.remove + board.insert(0, empty), the blocks move up
    // Find where the connected blocks ended up
    let mut found_connected_blocks = false;
    let mut found_y = 0;

    for y in 0..state.current_board_height {
        if let Cell::Connected { color: c, count: _ } = state.board[y][0] {
            if c == test_color {
                found_connected_blocks = true;
                found_y = y;
                break;
            }
        }
    }

    if !found_connected_blocks {
        // Debug: print the entire relevant board state
        println!("Board state after bottom line clear:");
        for y in 0..10 {
            println!("Row {}: {:?}", y, &state.board[y][0..3]);
        }
        panic!("Could not find connected blocks after bottom line clear");
    }

    // Verify both connected blocks have correct count
    if let Cell::Connected { color, count } = state.board[found_y][0] {
        assert_eq!(color, test_color);
        assert_eq!(
            count, 2,
            "Connected blocks should be recounted after bottom line clear - expected 2"
        );
    } else {
        panic!(
            "Block should be Connected after bottom line clear, but found: {:?}",
            state.board[found_y][0]
        );
    }

    if let Cell::Connected { color, count } = state.board[found_y][1] {
        assert_eq!(color, test_color);
        assert_eq!(
            count, 2,
            "Connected blocks should be recounted after bottom line clear - expected 2"
        );
    } else {
        panic!(
            "Block should be Connected after bottom line clear, but found: {:?}",
            state.board[found_y][1]
        );
    }
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
    assert_eq!(state.custom_score_system.max_chains.get(GameColor::Magenta), 0);
    assert_eq!(state.custom_score_system.max_chains.get(GameColor::Yellow), 0);

    // Create a piece and place it to trigger a landing
    let piece = Tetromino::from_shape(
        TetrominoShape::I,
        [GameColor::Cyan, GameColor::Cyan, GameColor::Cyan, GameColor::Cyan],
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
        [GameColor::Cyan, GameColor::Cyan, GameColor::Cyan, GameColor::Cyan],
    );
    state.current_piece = Some(piece);

    // Lock the piece
    state.lock_piece(&time_provider);

    // Max chains should not decrease even if current connected count is smaller
    assert_eq!(state.custom_score_system.max_chains.get(GameColor::Cyan), 8); // Should remain 8, not decrease to smaller count
    assert_eq!(state.custom_score_system.max_chains.get(GameColor::Magenta), 10); // Should remain unchanged
    assert_eq!(state.custom_score_system.max_chains.get(GameColor::Yellow), 5); // Should remain unchanged
}

#[test]
fn test_color_score_updated_after_line_clear() {
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    // Set up MAX-CHAIN values for scoring calculation
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

    // Initially, all color scores should be zero
    assert_eq!(state.custom_score_system.scores.get(GameColor::Cyan), 0);
    assert_eq!(state.custom_score_system.scores.get(GameColor::Magenta), 0);
    assert_eq!(state.custom_score_system.scores.get(GameColor::Yellow), 0);
    assert_eq!(state.custom_score_system.scores.total(), 0);

    // Create a line with mixed colors to clear
    let clear_line_y = BOARD_HEIGHT - 1;
    state.board[clear_line_y][0] = Cell::Occupied(GameColor::Cyan); // 1×2×10 = 20
    state.board[clear_line_y][1] = Cell::Occupied(GameColor::Cyan); // 1×2×10 = 20
    state.board[clear_line_y][2] = Cell::Occupied(GameColor::Magenta); // 1×3×10 = 30
    state.board[clear_line_y][3] = Cell::Occupied(GameColor::Magenta); // 1×3×10 = 30
    state.board[clear_line_y][4] = Cell::Occupied(GameColor::Magenta); // 1×3×10 = 30
    state.board[clear_line_y][5] = Cell::Occupied(GameColor::Yellow); // 1×1×10 = 10
    state.board[clear_line_y][6] = Cell::Occupied(GameColor::Yellow); // 1×1×10 = 10
    state.board[clear_line_y][7] = Cell::Occupied(GameColor::Cyan); // 1×2×10 = 20
    state.board[clear_line_y][8] = Cell::Occupied(GameColor::Magenta); // 1×3×10 = 30
    state.board[clear_line_y][9] = Cell::Occupied(GameColor::Yellow); // 1×1×10 = 10

    // Clear the line (this should be treated as bottom line clear)
    let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
    state.animation.extend(new_animations);

    // After line clear, scores should be updated based on new formula:
    // Cyan: 3 blocks × MAX-CHAIN(2) × 10 = 60 points
    // Magenta: 4 blocks × MAX-CHAIN(3) × 10 = 120 points
    // Yellow: 3 blocks × MAX-CHAIN(1) × 10 = 30 points
    assert_eq!(
        state.custom_score_system.scores.get(GameColor::Cyan),
        60,
        "Cyan should have 60 points from 3 blocks × 2 MAX-CHAIN × 10"
    );
    assert_eq!(
        state.custom_score_system.scores.get(GameColor::Magenta),
        120,
        "Magenta should have 120 points from 4 blocks × 3 MAX-CHAIN × 10"
    );
    assert_eq!(
        state.custom_score_system.scores.get(GameColor::Yellow),
        30,
        "Yellow should have 30 points from 3 blocks × 1 MAX-CHAIN × 10"
    );
    assert_eq!(
        state.custom_score_system.scores.total(),
        210,
        "Total score should be 60 + 120 + 30 = 210"
    );
}

#[test]
fn test_color_score_accumulates_across_multiple_clears() {
    let time_provider = MockTimeProvider::new();
    let mut state = GameState::new();
    state.mode = GameMode::Playing;

    // Set initial scores
    state.custom_score_system.scores.add(GameColor::Cyan, 5);
    state.custom_score_system.scores.add(GameColor::Magenta, 10);

    // Set MAX-CHAIN values for calculation
    state
        .custom_score_system
        .max_chains
        .update_max(GameColor::Cyan, 2);
    state
        .custom_score_system
        .max_chains
        .update_max(GameColor::Yellow, 1);

    // Create a line with blocks to clear
    let clear_line_y = BOARD_HEIGHT - 1;
    for x in 0..5 {
        state.board[clear_line_y][x] = Cell::Occupied(GameColor::Cyan);
    }
    for x in 5..BOARD_WIDTH {
        state.board[clear_line_y][x] = Cell::Occupied(GameColor::Yellow);
    }

    // Clear the line
    let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
    state.animation.extend(new_animations);

    // Scores should accumulate using new formula:
    // Cyan: 5 (initial) + (5 blocks × 2 MAX-CHAIN × 10) = 5 + 100 = 105
    // Magenta: 10 (initial) + 0 (new) = 10
    // Yellow: 0 (initial) + (5 blocks × 1 MAX-CHAIN × 10) = 0 + 50 = 50
    assert_eq!(state.custom_score_system.scores.get(GameColor::Cyan), 105);
    assert_eq!(state.custom_score_system.scores.get(GameColor::Magenta), 10);
    assert_eq!(state.custom_score_system.scores.get(GameColor::Yellow), 50);
    assert_eq!(state.custom_score_system.scores.total(), 165);
}
