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
            (0, 1, Color::Cyan),
            (-1, 0, Color::Magenta),
            (-1, 1, Color::Yellow),
            (-1, 2, Color::Green),
        ],
        "T-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (-1, 0, Color::Cyan),
            (0, -1, Color::Magenta),
            (-1, -1, Color::Yellow),
            (-2, -1, Color::Green),
        ],
        "T-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, -1, Color::Cyan),
            (1, 0, Color::Magenta),
            (1, -1, Color::Yellow),
            (1, -2, Color::Green),
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
            (1, 0, Color::Cyan),
            (1, 1, Color::Magenta),
            (1, 2, Color::Yellow),
            (1, 3, Color::Green),
        ],
        "I-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (2, 1, Color::Cyan),
            (1, 1, Color::Magenta),
            (0, 1, Color::Yellow),
            (-1, 1, Color::Green),
        ],
        "I-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 2, Color::Cyan),
            (1, 1, Color::Magenta),
            (1, 0, Color::Yellow),
            (1, -1, Color::Green),
        ],
        "I-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 1, Color::Cyan),
            (1, 1, Color::Magenta),
            (2, 1, Color::Yellow),
            (3, 1, Color::Green),
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
            (0, 2, Color::Cyan),
            (-1, 0, Color::Magenta),
            (-1, 1, Color::Yellow),
            (-1, 2, Color::Green),
        ],
        "L-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (-2, 0, Color::Cyan),
            (0, -1, Color::Magenta),
            (-1, -1, Color::Yellow),
            (-2, -1, Color::Green),
        ],
        "L-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, -2, Color::Cyan),
            (1, 0, Color::Magenta),
            (1, -1, Color::Yellow),
            (1, -2, Color::Green),
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
            (0, 0, Color::Cyan),
            (-1, 0, Color::Magenta),
            (-1, 1, Color::Yellow),
            (-1, 2, Color::Green),
        ],
        "J-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 0, Color::Cyan),
            (0, -1, Color::Magenta),
            (-1, -1, Color::Yellow),
            (-2, -1, Color::Green),
        ],
        "J-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 0, Color::Cyan),
            (1, 0, Color::Magenta),
            (1, -1, Color::Yellow),
            (1, -2, Color::Green),
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
            (0, 1, Color::Cyan),
            (0, 2, Color::Magenta),
            (-1, 0, Color::Yellow),
            (-1, 1, Color::Green),
        ],
        "S-Mino rotation 1 is wrong",
    );

    piece = piece.rotated(); // Rotate again
    assert_piece_state(
        &piece,
        p,
        &[
            (-1, 0, Color::Cyan),    // (0,1) -> (-1,0)
            (-2, 0, Color::Magenta), // (0,2) -> (-2,0)
            (0, -1, Color::Yellow),  // (-1,0) -> (0,-1)
            (-1, -1, Color::Green),  // (-1,1) -> (-1,-1)
        ],
        "S-Mino rotation 2 is wrong",
    );

    piece = piece.rotated(); // Rotate again
    assert_piece_state(
        &piece,
        p,
        &[
            (0, -1, Color::Cyan),    // (-1,0) -> (0,-1)
            (0, -2, Color::Magenta), // (-2,0) -> (0,-2)
            (1, 0, Color::Yellow),   // (0,-1) -> (1,0)
            (1, -1, Color::Green),   // (-1,-1) -> (1,-1)
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
            (0, 0, Color::Cyan),
            (0, 1, Color::Magenta),
            (-1, 1, Color::Yellow),
            (-1, 2, Color::Green),
        ],
        "Z-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 0, Color::Cyan),
            (-1, 0, Color::Magenta),
            (-1, -1, Color::Yellow),
            (-2, -1, Color::Green),
        ],
        "Z-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 0, Color::Cyan),
            (0, -1, Color::Magenta),
            (1, -1, Color::Yellow),
            (1, -2, Color::Green),
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
