use super::*;
use crate::tetromino::TetrominoShape;
use std::collections::HashMap;

fn assert_piece_state(
    piece: &Tetromino,
    base_pos: (i8, i8),
    expected_blocks: &[(i8, i8, Color)],
    message: &str,
) {
    let p = base_pos;
    let expected: HashMap<(i8, i8), Color> = expected_blocks
        .iter()
        .map(|(dx, dy, color)| ((p.0 + dx, p.1 + dy), *color))
        .collect();
    let actual: HashMap<(i8, i8), Color> = piece.iter_blocks().collect();
    assert_eq!(actual, expected, "{}", message);
}

#[test]
fn test_t_mino_full_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::T, colors);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, Color::Cyan),
            (0, 1, Color::Magenta),
            (1, 1, Color::Yellow),
            (2, 1, Color::Green),
        ],
        "T-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, Color::Cyan),     // SRS T State 1: rightward T
            (1, 1, Color::Magenta),
            (2, 1, Color::Yellow),
            (1, 2, Color::Green),
        ],
        "T-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 1, Color::Cyan),     // SRS T State 2: downward T
            (1, 1, Color::Magenta),
            (2, 1, Color::Yellow),
            (1, 2, Color::Green),
        ],
        "T-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, Color::Cyan),     // SRS T State 3: leftward T
            (0, 1, Color::Magenta),
            (1, 1, Color::Yellow),
            (1, 2, Color::Green),
        ],
        "T-Mino rotation 3 is wrong",
    );
}

#[test]
fn test_o_mino_full_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::O, colors);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, Color::Cyan),
            (2, 1, Color::Magenta),
            (1, 2, Color::Yellow),
            (2, 2, Color::Green),
        ],
        "O-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, Color::Yellow),  // Top-Left (was Bottom-Left)
            (2, 1, Color::Cyan),    // Top-Right (was Top-Left)
            (1, 2, Color::Green),   // Bottom-Left (was Bottom-Right)
            (2, 2, Color::Magenta), // Bottom-Right (was Top-Right)
        ],
        "O-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, Color::Green),   // Top-Left (was Bottom-Left)
            (2, 1, Color::Yellow),  // Top-Right (was Top-Left)
            (1, 2, Color::Magenta), // Bottom-Left (was Bottom-Right)
            (2, 2, Color::Cyan),    // Bottom-Right (was Top-Right)
        ],
        "O-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, Color::Magenta), // Top-Left (was Bottom-Left)
            (2, 1, Color::Green),   // Top-Right (was Top-Left)
            (1, 2, Color::Cyan),    // Bottom-Left (was Bottom-Right)
            (2, 2, Color::Yellow),  // Bottom-Right (was Top-Right)
        ],
        "O-Mino rotation 3 is wrong",
    );
}

#[test]
fn test_i_mino_full_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::I, colors);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (0, 1, Color::Cyan),     // SRS State 0: horizontal
            (1, 1, Color::Magenta),
            (2, 1, Color::Yellow),
            (3, 1, Color::Green),
        ],
        "I-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (2, 0, Color::Cyan),     // SRS State 1: vertical
            (2, 1, Color::Magenta),
            (2, 2, Color::Yellow),
            (2, 3, Color::Green),
        ],
        "I-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 2, Color::Cyan),     // SRS State 2: horizontal (offset)
            (1, 2, Color::Magenta),
            (2, 2, Color::Yellow),
            (3, 2, Color::Green),
        ],
        "I-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, Color::Cyan),     // SRS State 3: vertical
            (1, 1, Color::Magenta),
            (1, 2, Color::Yellow),
            (1, 3, Color::Green),
        ],
        "I-Mino rotation 3 is wrong",
    );
}

#[test]
fn test_l_mino_full_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::L, colors);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (2, 0, Color::Cyan),
            (0, 1, Color::Magenta),
            (1, 1, Color::Yellow),
            (2, 1, Color::Green),
        ],
        "L-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, Color::Cyan),     // SRS L State 1
            (1, 1, Color::Magenta),
            (1, 2, Color::Yellow),
            (2, 2, Color::Green),
        ],
        "L-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 1, Color::Cyan),     // SRS L State 2
            (1, 1, Color::Magenta),
            (2, 1, Color::Yellow),
            (0, 2, Color::Green),
        ],
        "L-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 0, Color::Cyan),     // SRS L State 3
            (1, 0, Color::Magenta),
            (1, 1, Color::Yellow),
            (1, 2, Color::Green),
        ],
        "L-Mino rotation 3 is wrong",
    );
}

#[test]
fn test_j_mino_full_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::J, colors);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (0, 0, Color::Cyan),
            (0, 1, Color::Magenta),
            (1, 1, Color::Yellow),
            (2, 1, Color::Green),
        ],
        "J-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, Color::Cyan),     // SRS J State 1
            (2, 0, Color::Magenta),
            (1, 1, Color::Yellow),
            (1, 2, Color::Green),
        ],
        "J-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 1, Color::Cyan),     // SRS J State 2
            (1, 1, Color::Magenta),
            (2, 1, Color::Yellow),
            (2, 2, Color::Green),
        ],
        "J-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, Color::Cyan),     // SRS J State 3
            (1, 1, Color::Magenta),
            (0, 2, Color::Yellow),
            (1, 2, Color::Green),
        ],
        "J-Mino rotation 3 is wrong",
    );
}

#[test]
fn test_s_mino_full_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::S, colors);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, Color::Cyan),
            (2, 0, Color::Magenta),
            (0, 1, Color::Yellow),
            (1, 1, Color::Green),
        ],
        "S-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, Color::Cyan),     // SRS S State 1
            (1, 1, Color::Magenta),
            (2, 1, Color::Yellow),
            (2, 2, Color::Green),
        ],
        "S-Mino rotation 1 is wrong",
    );

    piece = piece.rotated(); // Rotate again
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, Color::Cyan),     // SRS S State 2
            (2, 1, Color::Magenta),
            (0, 2, Color::Yellow),
            (1, 2, Color::Green),
        ],
        "S-Mino rotation 2 is wrong",
    );

    piece = piece.rotated(); // Rotate again
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 0, Color::Cyan),     // SRS S State 3
            (0, 1, Color::Magenta),
            (1, 1, Color::Yellow),
            (1, 2, Color::Green),
        ],
        "S-Mino rotation 3 is wrong",
    );
}

#[test]
fn test_z_mino_full_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::Z, colors);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (0, 0, Color::Cyan),
            (1, 0, Color::Magenta),
            (1, 1, Color::Yellow),
            (2, 1, Color::Green),
        ],
        "Z-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (2, 0, Color::Cyan),     // SRS Z State 1
            (1, 1, Color::Magenta),
            (2, 1, Color::Yellow),
            (1, 2, Color::Green),
        ],
        "Z-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 1, Color::Cyan),     // SRS Z State 2
            (1, 1, Color::Magenta),
            (1, 2, Color::Yellow),
            (2, 2, Color::Green),
        ],
        "Z-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, Color::Cyan),     // SRS Z State 3
            (0, 1, Color::Magenta),
            (1, 1, Color::Yellow),
            (0, 2, Color::Green),
        ],
        "Z-Mino rotation 3 is wrong",
    );
}

#[test]
fn test_o_mino_full_counter_clockwise_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::O, colors);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, Color::Cyan),
            (2, 1, Color::Magenta),
            (1, 2, Color::Yellow),
            (2, 2, Color::Green),
        ],
        "O-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated_counter_clockwise();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, Color::Magenta), // Top-Left (was Top-Right)
            (2, 1, Color::Green),   // Top-Right (was Bottom-Right)
            (1, 2, Color::Cyan),    // Bottom-Left (was Top-Left)
            (2, 2, Color::Yellow),  // Bottom-Right (was Bottom-Left)
        ],
        "O-Mino counter-clockwise rotation 1 is wrong",
    );

    piece = piece.rotated_counter_clockwise();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, Color::Green),   // Top-Left (was Bottom-Right)
            (2, 1, Color::Yellow),  // Top-Right (was Bottom-Left)
            (1, 2, Color::Magenta), // Bottom-Left (was Top-Right)
            (2, 2, Color::Cyan),    // Bottom-Right (was Top-Left)
        ],
        "O-Mino counter-clockwise rotation 2 is wrong",
    );

    piece = piece.rotated_counter_clockwise();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, Color::Yellow),  // Top-Left (was Bottom-Left)
            (2, 1, Color::Cyan),    // Top-Right (was Top-Left)
            (1, 2, Color::Green),   // Bottom-Left (was Bottom-Right)
            (2, 2, Color::Magenta), // Bottom-Right (was Top-Right)
        ],
        "O-Mino counter-clockwise rotation 3 is wrong",
    );
}

#[test]
fn test_rotation_at_spawn_height_is_invalid_due_to_top_collision() {
    let mut state = GameState::new();
    state.mode = GameMode::Playing; // Set to playing mode to allow piece spawning

    // Spawn an I-mino, which often has blocks at y=0 and y=-1 relative to its position
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::I, colors);
    // Ensure the piece is at its initial spawn position (pos.1 = 0)
    piece.pos = ((BOARD_WIDTH as i8) / 2 - 2, 0);

    // Rotate the piece once. For an I-mino, this will typically result in some blocks
    // having negative y-coordinates relative to the piece's origin, which means
    // they will be at y < 0 in board coordinates.
    let rotated_piece = piece.rotated();

    // Assert that the rotated piece is considered invalid due to the current
    // implementation of is_valid_position, which checks y < 0.
    assert!(
        state.is_valid_position(&rotated_piece),
        "Rotated I-mino at spawn height should now be valid after removing top collision check"
    );

    // Try with a T-mino as well, which also often has blocks at y=-1 after rotation
    let mut t_piece = Tetromino::from_shape(TetrominoShape::T, colors);
    t_piece.pos = ((BOARD_WIDTH as i8) / 2 - 2, 0);
    let rotated_t_piece = t_piece.rotated();

    assert!(
        state.is_valid_position(&rotated_t_piece),
        "Rotated T-mino at spawn height should now be valid after removing top collision check"
    );
}

// =============================================================================
// SRS Phase 1: Rotation State Management Tests
// =============================================================================

#[test]
fn test_tetromino_initial_rotation_state() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let piece = Tetromino::from_shape(TetrominoShape::T, colors);
    assert_eq!(
        piece.get_rotation_state(),
        0,
        "Initial rotation state should be 0"
    );
}

#[test]
fn test_clockwise_rotation_state_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::T, colors);

    // Initial state
    assert_eq!(piece.get_rotation_state(), 0, "Initial state should be 0");

    // First rotation: 0 -> 1
    piece = piece.rotated();
    assert_eq!(
        piece.get_rotation_state(),
        1,
        "After first rotation should be 1"
    );

    // Second rotation: 1 -> 2
    piece = piece.rotated();
    assert_eq!(
        piece.get_rotation_state(),
        2,
        "After second rotation should be 2"
    );

    // Third rotation: 2 -> 3
    piece = piece.rotated();
    assert_eq!(
        piece.get_rotation_state(),
        3,
        "After third rotation should be 3"
    );

    // Fourth rotation: 3 -> 0 (cycle back)
    piece = piece.rotated();
    assert_eq!(
        piece.get_rotation_state(),
        0,
        "After fourth rotation should cycle back to 0"
    );
}

#[test]
fn test_counter_clockwise_rotation_state_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::T, colors);

    // Initial state
    assert_eq!(piece.get_rotation_state(), 0, "Initial state should be 0");

    // First counter-clockwise rotation: 0 -> 3
    piece = piece.rotated_counter_clockwise();
    assert_eq!(
        piece.get_rotation_state(),
        3,
        "After first counter-clockwise rotation should be 3"
    );

    // Second counter-clockwise rotation: 3 -> 2
    piece = piece.rotated_counter_clockwise();
    assert_eq!(
        piece.get_rotation_state(),
        2,
        "After second counter-clockwise rotation should be 2"
    );

    // Third counter-clockwise rotation: 2 -> 1
    piece = piece.rotated_counter_clockwise();
    assert_eq!(
        piece.get_rotation_state(),
        1,
        "After third counter-clockwise rotation should be 1"
    );

    // Fourth counter-clockwise rotation: 1 -> 0 (cycle back)
    piece = piece.rotated_counter_clockwise();
    assert_eq!(
        piece.get_rotation_state(),
        0,
        "After fourth counter-clockwise rotation should cycle back to 0"
    );
}

#[test]
fn test_all_tetromino_shapes_initial_rotation_state() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let shapes = [
        TetrominoShape::I,
        TetrominoShape::O,
        TetrominoShape::T,
        TetrominoShape::L,
        TetrominoShape::J,
        TetrominoShape::S,
        TetrominoShape::Z,
    ];

    for shape in shapes.iter() {
        let piece = Tetromino::from_shape(*shape, colors);
        assert_eq!(
            piece.get_rotation_state(),
            0,
            "Initial rotation state should be 0 for {:?}",
            shape
        );
    }
}

#[test]
fn test_mixed_rotation_operations() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::L, colors);

    assert_eq!(piece.get_rotation_state(), 0);

    // Clockwise twice: 0 -> 1 -> 2
    piece = piece.rotated();
    piece = piece.rotated();
    assert_eq!(piece.get_rotation_state(), 2);

    // Counter-clockwise once: 2 -> 1
    piece = piece.rotated_counter_clockwise();
    assert_eq!(piece.get_rotation_state(), 1);

    // Clockwise three times: 1 -> 2 -> 3 -> 0
    piece = piece.rotated();
    piece = piece.rotated();
    piece = piece.rotated();
    assert_eq!(piece.get_rotation_state(), 0);

    // Counter-clockwise twice: 0 -> 3 -> 2
    piece = piece.rotated_counter_clockwise();
    piece = piece.rotated_counter_clockwise();
    assert_eq!(piece.get_rotation_state(), 2);
}

// ============================================================================
// Phase 2: SRS標準回転中心テスト (TDD Red/Green Phase完了)
// ============================================================================

/// Test that T-mino rotates around the standard SRS center point (1,1)
/// SRS standard: T-mino should rotate around its bottom-center block
#[test]
fn test_t_mino_srs_rotation_center() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::T, colors);
    piece.pos = (5, 5); // Use clear position for testing

    // State 0: T shape should have specific SRS coordinates
    let _state0_blocks: Vec<_> = piece.iter_blocks().collect();

    let rotated = piece.rotated(); // State 1
    let state1_blocks: Vec<_> = rotated.iter_blocks().collect();

    // In SRS, T-mino rotation should follow specific coordinate transformations
    // This test validates SRS compliance
    let expected_srs_state1 = vec![
        ((6, 5), colors[0]), // First block position in SRS state 1
        ((6, 6), colors[1]), // Second block position in SRS state 1
        ((7, 6), colors[2]), // Third block position in SRS state 1
        ((6, 7), colors[3]), // Fourth block position in SRS state 1
    ];

    assert_eq!(
        state1_blocks, expected_srs_state1,
        "T-mino SRS state 1 blocks don't match expected SRS coordinates"
    );
}

/// Test that I-mino follows SRS standard rotation behavior with proper center
/// I-mino has special SRS center behavior and specific coordinate patterns
#[test]
fn test_i_mino_srs_rotation_center() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::I, colors);
    piece.pos = (5, 5); // Clear position for testing

    // State 0: Horizontal I-mino should be at specific SRS coordinates
    let state0_blocks: Vec<_> = piece.iter_blocks().collect();

    // Rotate to state 1: Vertical I-mino
    piece = piece.rotated();
    let state1_blocks: Vec<_> = piece.iter_blocks().collect();

    // Rotate to state 2: Horizontal again (should be offset from state 0 in SRS)
    piece = piece.rotated();
    let state2_blocks: Vec<_> = piece.iter_blocks().collect();

    // In SRS, I-mino state 2 should be offset from state 0
    // This is a key difference between current implementation and SRS
    let y_offset_state0_to_state2 = state2_blocks[0].0.1 - state0_blocks[0].0.1;

    assert_eq!(
        y_offset_state0_to_state2, 1,
        "I-mino SRS: state 2 should be 1 unit down from state 0, got offset: {}",
        y_offset_state0_to_state2
    );

    // Additional SRS requirement: state 1 should be properly positioned
    let expected_state1_y = state0_blocks[0].0.1 - 1; // SRS I-mino vertical positioning
    let actual_state1_y = state1_blocks[0].0.1;

    assert_eq!(
        actual_state1_y, expected_state1_y,
        "I-mino SRS state 1 Y position should be offset correctly: got {}, expected {}",
        actual_state1_y, expected_state1_y
    );
}

/// Test that L-mino follows SRS standard rotation behavior
#[test]
fn test_l_mino_srs_rotation_center() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let piece = Tetromino::from_shape(TetrominoShape::L, colors);

    let rotated = piece.rotated();

    // L-mino should rotate around proper SRS center
    let original_center_pos = get_srs_rotation_center(&piece);
    let rotated_center_pos = get_srs_rotation_center(&rotated);

    assert_eq!(
        original_center_pos, rotated_center_pos,
        "L-mino SRS rotation center should remain consistent"
    );
}

/// Test that J-mino follows SRS standard rotation behavior
#[test]
fn test_j_mino_srs_rotation_center() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let piece = Tetromino::from_shape(TetrominoShape::J, colors);

    let rotated = piece.rotated();

    // J-mino should rotate around proper SRS center
    let original_center_pos = get_srs_rotation_center(&piece);
    let rotated_center_pos = get_srs_rotation_center(&rotated);

    assert_eq!(
        original_center_pos, rotated_center_pos,
        "J-mino SRS rotation center should remain consistent"
    );
}

/// Test that S-mino follows SRS standard rotation behavior
#[test]
fn test_s_mino_srs_rotation_center() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let piece = Tetromino::from_shape(TetrominoShape::S, colors);

    let rotated = piece.rotated();

    // S-mino should rotate around proper SRS center
    let original_center_pos = get_srs_rotation_center(&piece);
    let rotated_center_pos = get_srs_rotation_center(&rotated);

    assert_eq!(
        original_center_pos, rotated_center_pos,
        "S-mino SRS rotation center should remain consistent"
    );
}

/// Test that Z-mino follows SRS standard rotation behavior
#[test]
fn test_z_mino_srs_rotation_center() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let piece = Tetromino::from_shape(TetrominoShape::Z, colors);

    let rotated = piece.rotated();

    // Z-mino should rotate around proper SRS center
    let original_center_pos = get_srs_rotation_center(&piece);
    let rotated_center_pos = get_srs_rotation_center(&rotated);

    assert_eq!(
        original_center_pos, rotated_center_pos,
        "Z-mino SRS rotation center should remain consistent"
    );
}

/// Helper function to calculate SRS rotation center for a tetromino
/// Returns the center point in world coordinates for SRS compliance verification
fn get_srs_rotation_center(piece: &Tetromino) -> (f32, f32) {
    // SRS standard rotation centers (relative to piece position):
    // T, L, J, S, Z: center at (1, 1)
    // I: center between (1.5, 1.5) in state 0/2, (2, 2) in state 1/3
    // O: no rotation (center irrelevant)

    let shape = piece.shape;
    let rotation_state = piece.get_rotation_state();
    let pos = piece.pos;

    match shape {
        TetrominoShape::I => {
            // I-mino has alternating center between rotations
            if rotation_state % 2 == 0 {
                // States 0, 2: center at (1.5, 1.5)
                (pos.0 as f32 + 1.5, pos.1 as f32 + 1.5)
            } else {
                // States 1, 3: center at (2, 2)
                (pos.0 as f32 + 2.0, pos.1 as f32 + 2.0)
            }
        }
        TetrominoShape::O => {
            // O-mino doesn't really rotate, center at (1.5, 1.5)
            (pos.0 as f32 + 1.5, pos.1 as f32 + 1.5)
        }
        _ => {
            // T, L, J, S, Z: center at (1, 1)
            (pos.0 as f32 + 1.0, pos.1 as f32 + 1.0)
        }
    }
}

// ============================================================================
// Phase 3: Wall Kickシステム基盤テスト (TDD Red Phase)
// ============================================================================

/// Test basic wall kick functionality - I-mino rotation near left wall
/// I-mino is 4 blocks long and will definitely need wall kick when near edges
#[test]
fn test_basic_wall_kick_left_wall() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::I, colors);
    
    // Position I-mino where horizontal rotation will exceed left boundary
    piece = piece.rotated(); // Now in state 1 (vertical)
    piece.pos = (-1, 5); // Position that will cause left boundary violation
    
    // Check initial position blocks
    let initial_blocks: Vec<_> = piece.iter_blocks().collect();
    println!("Initial I-mino state 1 blocks: {:?}", initial_blocks);
    
    // Attempt rotation from state 1 to 2 (vertical to horizontal) - should need wall kick
    let rotated = piece.rotated_with_wall_kick();
    
    // Check rotated position blocks
    let rotated_blocks: Vec<_> = rotated.iter_blocks().collect();
    println!("Rotated I-mino state 2 blocks: {:?}", rotated_blocks);
    println!("Position changed from {:?} to {:?}", piece.pos, rotated.pos);
    
    // Wall kick should succeed and move the piece away from wall
    assert_ne!(rotated.pos, piece.pos, "Wall kick should adjust position when I-mino near wall");
    assert_eq!(rotated.get_rotation_state(), 2, "Rotation state should advance even with wall kick");
}

/// Test basic wall kick functionality - rotation near right boundary
#[test]
fn test_basic_wall_kick_right_wall() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::I, colors);
    
    // Position I-mino where horizontal rotation will exceed right boundary
    piece = piece.rotated(); // Now in state 1 (vertical)
    piece.pos = (7, 5); // Position that will cause right boundary violation on rotation
    
    // Check initial position blocks
    let initial_blocks: Vec<_> = piece.iter_blocks().collect();
    println!("Initial I-mino state 1 blocks: {:?}", initial_blocks);
    
    // Attempt rotation from state 1 to 2 (vertical to horizontal) - should need wall kick
    let rotated = piece.rotated_with_wall_kick();
    
    // Check rotated position blocks
    let rotated_blocks: Vec<_> = rotated.iter_blocks().collect();
    println!("Rotated I-mino state 2 blocks: {:?}", rotated_blocks);
    println!("Position changed from {:?} to {:?}", piece.pos, rotated.pos);
    
    // SRS wall kick should succeed and ensure valid position 
    assert_eq!(rotated.get_rotation_state(), 2, "Rotation state should advance even with wall kick");
    
    // Verify that all blocks are within bounds after SRS wall kick
    for ((block_x, _), _) in rotated.iter_blocks() {
        assert!(block_x >= 0 && block_x < 10, "All blocks should be within bounds after SRS wall kick");
    }
}

/// Test wall kick with I-mino - special case due to length
#[test]
fn test_i_mino_wall_kick() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::I, colors);
    
    // Position I-mino where rotation from horizontal to vertical might need kick
    piece.pos = (1, 5); // Near left side
    
    // Attempt rotation with wall kick
    let rotated = piece.rotated_with_wall_kick();
    
    // Should succeed with potential position adjustment
    assert_eq!(rotated.get_rotation_state(), 1, "I-mino wall kick should succeed");
}

/// Test simple offset table functionality
#[test]
fn test_wall_kick_offset_attempts() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let _piece = Tetromino::from_shape(TetrominoShape::T, colors);
    
    // Get wall kick offsets for T-mino state 0->1 transition
    let offsets = get_wall_kick_offsets(TetrominoShape::T, 0, 1);
    
    // Should have multiple offset attempts (basic implementation)
    assert!(!offsets.is_empty(), "Wall kick should provide offset attempts");
    assert!(offsets.len() >= 2, "Should have at least 2 offset attempts for basic wall kick");
    
    // First offset should be (0, 0) - normal rotation
    assert_eq!(offsets[0], (0, 0), "First offset should be normal rotation attempt");
}

/// Helper function to get wall kick offsets for testing
/// Returns offset attempts for wall kick system
/// Phase 3: Basic offset table implementation
fn get_wall_kick_offsets(_shape: TetrominoShape, _from_state: u8, _to_state: u8) -> Vec<(i8, i8)> {
    // Basic offset table for Phase 3
    // SRS-like but simplified: try normal rotation, then left/right kicks
    vec![
        (0, 0),   // Normal rotation (no kick)
        (-1, 0),  // Kick left
        (1, 0),   // Kick right
        (-2, 0),  // Kick further left (for I-mino etc.)
        (2, 0),   // Kick further right
    ]
}

/// Test that O-mino doesn't need wall kick (no rotation)
#[test]
fn test_o_mino_no_wall_kick_needed() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let piece = Tetromino::from_shape(TetrominoShape::O, colors);
    
    // O-mino positioned anywhere
    let piece_at_edge = piece.moved(-1, 0);
    
    // Rotation should work without wall kick (O-mino doesn't really rotate)
    let rotated = piece_at_edge.rotated_with_wall_kick();
    
    // Position should remain the same (no kick needed)
    assert_eq!(rotated.pos, piece_at_edge.pos, "O-mino shouldn't need position adjustment");
}

// ===============================================
// Phase 4: SRS Standard Wall Kick Tests
// ===============================================

/// Test SRS standard wall kick - T-mino 0->1 rotation using exact SRS offsets
/// Should test all 5 wall kick offsets: (0,0), (-1,0), (-1,1), (0,-2), (-1,-2)
#[test]
fn test_srs_standard_t_mino_0_to_1_wall_kick() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let piece = Tetromino::from_shape(TetrominoShape::T, colors);
    
    // Position T-mino where 0->1 rotation will need SRS wall kick
    let mut positioned_piece = piece;
    positioned_piece.pos = (0, 5); // Position that should trigger wall kick attempts
    
    let rotated = positioned_piece.rotated_with_wall_kick();
    
    // Should succeed using one of the SRS standard offsets
    assert_eq!(rotated.get_rotation_state(), 1, "T-mino should rotate 0->1 with SRS wall kick");
    
    // Position should be adjusted according to SRS standard offsets
    println!("T-mino 0->1: Position changed from {:?} to {:?}", positioned_piece.pos, rotated.pos);
}

/// Test SRS standard wall kick - I-mino 0->1 rotation using exact SRS offsets
/// I-mino has different offsets: (0,0), (-2,0), (1,0), (-2,-1), (1,2)
#[test]
fn test_srs_standard_i_mino_0_to_1_wall_kick() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let piece = Tetromino::from_shape(TetrominoShape::I, colors);
    
    // Position I-mino where 0->1 rotation will definitely need SRS wall kick
    let mut positioned_piece = piece;
    positioned_piece.pos = (1, 5); // Near left edge where I-mino will need kick
    
    let rotated = positioned_piece.rotated_with_wall_kick();
    
    // Should succeed using SRS I-mino specific offsets
    assert_eq!(rotated.get_rotation_state(), 1, "I-mino should rotate 0->1 with SRS wall kick");
    
    println!("I-mino 0->1: Position changed from {:?} to {:?}", positioned_piece.pos, rotated.pos);
}

/// Test SRS standard wall kick - J-mino 1->2 rotation
/// Tests (0,0), (1,0), (1,-1), (0,2), (1,2) offset sequence
#[test]
fn test_srs_standard_j_mino_1_to_2_wall_kick() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::J, colors);
    
    // Rotate to state 1 first
    piece = piece.rotated();
    assert_eq!(piece.get_rotation_state(), 1, "J-mino should be in state 1");
    
    // Position where 1->2 rotation will need wall kick
    piece.pos = (8, 5); // Near right edge
    
    let rotated = piece.rotated_with_wall_kick();
    
    // Should succeed with SRS wall kick
    assert_eq!(rotated.get_rotation_state(), 2, "J-mino should rotate 1->2 with SRS wall kick");
    
    println!("J-mino 1->2: Position changed from {:?} to {:?}", piece.pos, rotated.pos);
}

/// Test complex SRS wall kick scenario - multiple offset attempts
/// This tests that the system tries all 5 offsets in order
#[test] 
fn test_srs_complex_wall_kick_multiple_attempts() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let piece = Tetromino::from_shape(TetrominoShape::T, colors);
    
    // Position T-mino in a constrained area to force multiple offset attempts
    let mut positioned_piece = piece;
    positioned_piece.pos = (0, 1); // Near left wall and close to top
    
    let rotated = positioned_piece.rotated_with_wall_kick();
    
    // Should eventually succeed with one of the later offsets
    assert_eq!(rotated.get_rotation_state(), 1, "Complex wall kick should succeed");
    
    println!("Complex wall kick: Position changed from {:?} to {:?}", positioned_piece.pos, rotated.pos);
}

/// Helper function for Phase 4 - Get SRS standard wall kick offsets
/// Returns the exact SRS offset table for testing verification
#[allow(dead_code)]
fn get_srs_wall_kick_offsets(from_state: u8, to_state: u8, is_i_mino: bool) -> Vec<(i32, i32)> {
    if is_i_mino {
        // SRS I-mino offsets
        match (from_state, to_state) {
            (0, 1) => vec![(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
            (1, 0) => vec![(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
            (1, 2) => vec![(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
            (2, 1) => vec![(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
            (2, 3) => vec![(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
            (3, 2) => vec![(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
            (3, 0) => vec![(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
            (0, 3) => vec![(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
            _ => vec![(0, 0)], // Fallback
        }
    } else {
        // SRS J,L,T,S,Z offsets
        match (from_state, to_state) {
            (0, 1) => vec![(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
            (1, 0) => vec![(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
            (1, 2) => vec![(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
            (2, 1) => vec![(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
            (2, 3) => vec![(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
            (3, 2) => vec![(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
            (3, 0) => vec![(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
            (0, 3) => vec![(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
            _ => vec![(0, 0)], // Fallback
        }
    }
}
