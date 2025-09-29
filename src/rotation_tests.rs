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
            (0, 1, Color::Cyan),
            (-1, 1, Color::Magenta),
            (-2, 1, Color::Yellow),
            (-3, 1, Color::Green),
        ],
        "I-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (-1, 0, Color::Cyan),
            (-1, -1, Color::Magenta),
            (-1, -2, Color::Yellow),
            (-1, -3, Color::Green),
        ],
        "I-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, -1, Color::Cyan),
            (1, -1, Color::Magenta),
            (2, -1, Color::Yellow),
            (3, -1, Color::Green),
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
