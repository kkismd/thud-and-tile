use super::*;
use std::collections::HashMap;

fn assert_piece_state(
    piece: &Tetromino,
    base_pos: (i8, i8),
    colors: &[Color; 4],
    expected_blocks: &[(i8, i8, usize)],
    message: &str,
) {
    let p = base_pos;
    let expected: HashMap<(i8, i8), Color> = expected_blocks
        .iter()
        .map(|(dx, dy, color_idx)| ((p.0 + dx, p.1 + dy), colors[*color_idx]))
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
        &colors,
        &[(1, 0, 0), (0, 1, 1), (1, 1, 2), (2, 1, 3)],
        "T-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(1, 0, 1), (1, 1, 2), (2, 1, 0), (1, 2, 3)],
        "T-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(0, 1, 3), (1, 1, 2), (2, 1, 1), (1, 2, 0)],
        "T-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(1, 0, 0), (0, 1, 3), (1, 1, 2), (1, 2, 1)],
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
        &colors,
        &[(1, 1, 0), (2, 1, 1), (1, 2, 2), (2, 2, 3)],
        "O-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(1, 1, 2), (2, 1, 0), (1, 2, 3), (2, 2, 1)],
        "O-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(1, 1, 3), (2, 1, 1), (1, 2, 0), (2, 2, 2)],
        "O-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(1, 1, 1), (2, 1, 0), (1, 2, 2), (2, 2, 3)],
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
        &colors,
        &[(1, 0, 0), (1, 1, 1), (1, 2, 2), (1, 3, 3)],
        "I-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(0, 2, 3), (1, 2, 2), (2, 2, 1), (3, 2, 0)],
        "I-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(2, 0, 3), (2, 1, 2), (2, 2, 1), (2, 3, 0)],
        "I-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(0, 1, 3), (1, 1, 2), (2, 1, 1), (3, 1, 0)],
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
        &colors,
        &[(2, 0, 0), (0, 1, 1), (1, 1, 2), (2, 1, 3)],
        "L-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(1, 0, 1), (1, 1, 2), (1, 2, 3), (2, 2, 0)],
        "L-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(0, 1, 3), (1, 1, 2), (2, 1, 1), (0, 2, 0)],
        "L-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(0, 0, 0), (1, 0, 3), (1, 1, 2), (1, 2, 1)],
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
        &colors,
        &[(0, 0, 0), (0, 1, 1), (1, 1, 2), (2, 1, 3)],
        "J-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(1, 0, 1), (2, 0, 0), (1, 1, 2), (1, 2, 3)],
        "J-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(0, 1, 3), (1, 1, 2), (2, 1, 1), (2, 2, 0)],
        "J-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(1, 0, 3), (1, 1, 2), (0, 2, 0), (1, 2, 1)],
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
        &colors,
        &[(1, 0, 0), (2, 0, 1), (0, 1, 2), (1, 1, 3)],
        "S-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(1, 0, 2), (1, 1, 3), (2, 1, 0), (2, 2, 1)],
        "S-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(1, 1, 1), (2, 1, 0), (0, 2, 3), (1, 2, 2)],
        "S-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(0, 0, 0), (0, 1, 3), (1, 1, 2), (1, 2, 1)],
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
        &colors,
        &[(0, 0, 0), (1, 0, 1), (1, 1, 2), (2, 1, 3)],
        "Z-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(2, 0, 0), (1, 1, 2), (2, 1, 1), (1, 2, 3)],
        "Z-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(0, 1, 3), (1, 1, 2), (1, 2, 1), (2, 2, 0)],
        "Z-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &colors,
        &[(1, 0, 3), (0, 1, 2), (1, 1, 1), (0, 2, 0)],
        "Z-Mino rotation 3 is wrong",
    );
}
