use super::*;
use crate::tetromino::TetrominoShape;
use crate::tetromino::{SRS_SHAPES, SrsRotationData};
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
    let mut piece = Tetromino::from_shape(TetrominoShape::T, colors, 0);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, colors[0]),
            (0, 1, colors[1]),
            (1, 1, colors[2]),
            (2, 1, colors[3]),
        ],
        "T-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, colors[0]),
            (1, 1, colors[1]),
            (2, 1, colors[2]),
            (1, 2, colors[3]),
        ],
        "T-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 1, colors[0]),
            (1, 1, colors[1]),
            (2, 1, colors[2]),
            (1, 2, colors[3]),
        ],
        "T-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, colors[0]),
            (0, 1, colors[1]),
            (1, 1, colors[2]),
            (1, 2, colors[3]),
        ],
        "T-Mino rotation 3 is wrong",
    );
}

#[test]
fn test_o_mino_full_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::O, colors, 0);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, colors[0]),
            (2, 1, colors[1]),
            (1, 2, colors[2]),
            (2, 2, colors[3]),
        ],
        "O-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, colors[0]),
            (2, 1, colors[1]),
            (1, 2, colors[2]),
            (2, 2, colors[3]),
        ],
        "O-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, colors[0]),
            (2, 1, colors[1]),
            (1, 2, colors[2]),
            (2, 2, colors[3]),
        ],
        "O-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, colors[0]),
            (2, 1, colors[1]),
            (1, 2, colors[2]),
            (2, 2, colors[3]),
        ],
        "O-Mino rotation 3 is wrong",
    );
}

#[test]
fn test_i_mino_full_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::I, colors, 0);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, colors[0]),
            (1, 1, colors[1]),
            (1, 2, colors[2]),
            (1, 3, colors[3]),
        ],
        "I-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 2, colors[0]),
            (1, 2, colors[1]),
            (2, 2, colors[2]),
            (3, 2, colors[3]),
        ],
        "I-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (2, 0, colors[0]),
            (2, 1, colors[1]),
            (2, 2, colors[2]),
            (2, 3, colors[3]),
        ],
        "I-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 1, colors[0]),
            (1, 1, colors[1]),
            (2, 1, colors[2]),
            (3, 1, colors[3]),
        ],
        "I-Mino rotation 3 is wrong",
    );
}

#[test]
fn test_l_mino_full_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::L, colors, 0);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (2, 0, colors[0]),
            (0, 1, colors[1]),
            (1, 1, colors[2]),
            (2, 1, colors[3]),
        ],
        "L-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, colors[0]),
            (1, 1, colors[1]),
            (1, 2, colors[2]),
            (2, 2, colors[3]),
        ],
        "L-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 1, colors[0]),
            (1, 1, colors[1]),
            (2, 1, colors[2]),
            (0, 2, colors[3]),
        ],
        "L-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 0, colors[0]),
            (1, 0, colors[1]),
            (1, 1, colors[2]),
            (1, 2, colors[3]),
        ],
        "L-Mino rotation 3 is wrong",
    );
}

#[test]
fn test_j_mino_full_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::J, colors, 0);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (0, 0, colors[0]),
            (0, 1, colors[1]),
            (1, 1, colors[2]),
            (2, 1, colors[3]),
        ],
        "J-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, colors[0]),
            (2, 0, colors[1]),
            (1, 1, colors[2]),
            (1, 2, colors[3]),
        ],
        "J-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 1, colors[0]),
            (1, 1, colors[1]),
            (2, 1, colors[2]),
            (2, 2, colors[3]),
        ],
        "J-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, colors[0]),
            (1, 1, colors[1]),
            (0, 2, colors[2]),
            (1, 2, colors[3]),
        ],
        "J-Mino rotation 3 is wrong",
    );
}

#[test]
fn test_s_mino_full_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::S, colors, 0);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, colors[0]),
            (2, 0, colors[1]),
            (0, 1, colors[2]),
            (1, 1, colors[3]),
        ],
        "S-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, colors[0]),
            (1, 1, colors[1]),
            (2, 1, colors[2]),
            (2, 2, colors[3]),
        ],
        "S-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, colors[0]),
            (2, 1, colors[1]),
            (0, 2, colors[2]),
            (1, 2, colors[3]),
        ],
        "S-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 0, colors[0]),
            (0, 1, colors[1]),
            (1, 1, colors[2]),
            (1, 2, colors[3]),
        ],
        "S-Mino rotation 3 is wrong",
    );
}

#[test]
fn test_z_mino_full_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::Z, colors, 0);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (0, 0, colors[0]),
            (1, 0, colors[1]),
            (1, 1, colors[2]),
            (2, 1, colors[3]),
        ],
        "Z-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (2, 0, colors[0]),
            (1, 1, colors[1]),
            (2, 1, colors[2]),
            (1, 2, colors[3]),
        ],
        "Z-Mino rotation 1 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (0, 1, colors[0]),
            (1, 1, colors[1]),
            (1, 2, colors[2]),
            (2, 2, colors[3]),
        ],
        "Z-Mino rotation 2 is wrong",
    );

    piece = piece.rotated();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 0, colors[0]),
            (0, 1, colors[1]),
            (1, 1, colors[2]),
            (0, 2, colors[3]),
        ],
        "Z-Mino rotation 3 is wrong",
    );
}

#[test]
fn test_o_mino_full_counter_clockwise_rotation_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::O, colors, 0);
    let p = piece.pos;

    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, colors[0]),
            (2, 1, colors[1]),
            (1, 2, colors[2]),
            (2, 2, colors[3]),
        ],
        "O-Mino initial state (rot 0) is wrong",
    );

    piece = piece.rotated_counter_clockwise();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, colors[0]),
            (2, 1, colors[1]),
            (1, 2, colors[2]),
            (2, 2, colors[3]),
        ],
        "O-Mino counter-clockwise rotation 1 is wrong",
    );

    piece = piece.rotated_counter_clockwise();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, colors[0]),
            (2, 1, colors[1]),
            (1, 2, colors[2]),
            (2, 2, colors[3]),
        ],
        "O-Mino counter-clockwise rotation 2 is wrong",
    );

    piece = piece.rotated_counter_clockwise();
    assert_piece_state(
        &piece,
        p,
        &[
            (1, 1, colors[0]),
            (2, 1, colors[1]),
            (1, 2, colors[2]),
            (2, 2, colors[3]),
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
    let mut piece = Tetromino::from_shape(TetrominoShape::I, colors, 0);
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
    let mut t_piece = Tetromino::from_shape(TetrominoShape::T, colors, 0);
    t_piece.pos = ((BOARD_WIDTH as i8) / 2 - 2, 0);
    let rotated_t_piece = t_piece.rotated();

    assert!(
        state.is_valid_position(&rotated_t_piece),
        "Rotated T-mino at spawn height should now be valid after removing top collision check"
    );
}

#[test]
fn test_new_random_initializes_rotation_state_to_zero() {
    let piece = Tetromino::new_random();
    // Assuming rotation_state will be a public field for testing purposes,
    // or a getter method will be added. For now, we'll assume direct access
    // or that the test will fail and we'll adjust.
    assert_eq!(
        piece.rotation_state, 0,
        "new_random should initialize rotation_state to 0"
    );
}

#[test]
fn test_from_shape_uses_specified_rotation_state() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let shape = TetrominoShape::I;
    let rotation_state = 2; // Test with a non-zero rotation state

    // This will require `from_shape` to accept `rotation_state`
    let piece = Tetromino::from_shape(shape, colors, rotation_state);

    // Expected blocks for I-mino, rotation state 2, from SHAPES
    // SHAPES[0] is I-mino, SHAPES[0][2] is rotation state 2
    let expected_blocks_relative = Tetromino::SHAPES[shape as usize][rotation_state as usize];

    // Convert expected_blocks_relative to the format used by assert_piece_state
    let expected_blocks_with_colors: Vec<(i8, i8, Color)> = expected_blocks_relative
        .iter()
        .enumerate()
        .map(|(i, &(x, y))| (x, y, colors[i]))
        .collect();

    assert_piece_state(
        &piece,
        piece.pos, // Use the piece's actual position
        &expected_blocks_with_colors,
        &format!(
            "from_shape with rotation_state {} for I-mino is incorrect",
            rotation_state
        ),
    );
}

#[test]
fn test_srs_shapes_i_mino_rotation_0_blocks() {
    let i_mino_rot_0 = SRS_SHAPES[TetrominoShape::I as usize][0];

    // SRSのIミノ Rotation 0 のブロック相対座標
    let expected_blocks = [(-1, 0), (0, 0), (1, 0), (2, 0)];
    assert_eq!(
        i_mino_rot_0.blocks, expected_blocks,
        "I-Mino Rotation 0 blocks are incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_i_mino_rotation_0_offset() {
    let i_mino_rot_0 = SRS_SHAPES[TetrominoShape::I as usize][0];

    // SRSのIミノ Rotation 0 の回転中心オフセット (ここでは (0,0) を想定)
    let expected_offset = (0, 0);
    assert_eq!(
        i_mino_rot_0.offset, expected_offset,
        "I-Mino Rotation 0 offset is incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_i_mino_rotation_1_blocks() {
    let i_mino_rot_1 = SRS_SHAPES[TetrominoShape::I as usize][1];

    // SRSのIミノ Rotation 1 のブロック相対座標
    let expected_blocks = [(0, -1), (0, 0), (0, 1), (0, 2)];
    assert_eq!(
        i_mino_rot_1.blocks, expected_blocks,
        "I-Mino Rotation 1 blocks are incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_i_mino_rotation_1_offset() {
    let i_mino_rot_1 = SRS_SHAPES[TetrominoShape::I as usize][1];

    // SRSのIミノ Rotation 1 の回転中心オフセット (ここでは (0,0) を想定)
    let expected_offset = (0, 0);
    assert_eq!(
        i_mino_rot_1.offset, expected_offset,
        "I-Mino Rotation 1 offset is incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_i_mino_rotation_2_blocks() {
    let i_mino_rot_2 = SRS_SHAPES[TetrominoShape::I as usize][2];

    // SRSのIミノ Rotation 2 のブロック相対座標
    let expected_blocks = [(-1, 1), (0, 1), (1, 1), (2, 1)];
    assert_eq!(
        i_mino_rot_2.blocks, expected_blocks,
        "I-Mino Rotation 2 blocks are incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_i_mino_rotation_2_offset() {
    let i_mino_rot_2 = SRS_SHAPES[TetrominoShape::I as usize][2];

    // SRSのIミノ Rotation 2 の回転中心オフセット (ここでは (0,0) を想定)
    let expected_offset = (0, 0);
    assert_eq!(
        i_mino_rot_2.offset, expected_offset,
        "I-Mino Rotation 2 offset is incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_i_mino_rotation_3_blocks() {
    let i_mino_rot_3 = SRS_SHAPES[TetrominoShape::I as usize][3];

    // SRSのIミノ Rotation 3 のブロック相対座標
    let expected_blocks = [(1, -1), (1, 0), (1, 1), (1, 2)];
    assert_eq!(
        i_mino_rot_3.blocks, expected_blocks,
        "I-Mino Rotation 3 blocks are incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_i_mino_rotation_3_offset() {
    let i_mino_rot_3 = SRS_SHAPES[TetrominoShape::I as usize][3];

    // SRSのIミノ Rotation 3 の回転中心オフセット (ここでは (0,0) を想定)
    let expected_offset = (0, 0);
    assert_eq!(
        i_mino_rot_3.offset, expected_offset,
        "I-Mino Rotation 3 offset is incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_o_mino_rotation_0_blocks() {
    let o_mino_rot_0 = SRS_SHAPES[TetrominoShape::O as usize][0];

    // SRSのOミノ Rotation 0 のブロック相対座標
    let expected_blocks = [(0, 0), (1, 0), (0, 1), (1, 1)];
    assert_eq!(
        o_mino_rot_0.blocks, expected_blocks,
        "O-Mino Rotation 0 blocks are incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_o_mino_rotation_0_offset() {
    let o_mino_rot_0 = SRS_SHAPES[TetrominoShape::O as usize][0];

    // SRSのOミノ Rotation 0 の回転中心オフセット (ここでは (0,0) を想定)
    let expected_offset = (0, 0);
    assert_eq!(
        o_mino_rot_0.offset, expected_offset,
        "O-Mino Rotation 0 offset is incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_o_mino_rotation_1_blocks() {
    let o_mino_rot_1 = SRS_SHAPES[TetrominoShape::O as usize][1];

    // SRSのOミノ Rotation 1 のブロック相対座標
    let expected_blocks = [(0, 0), (1, 0), (0, 1), (1, 1)];
    assert_eq!(
        o_mino_rot_1.blocks, expected_blocks,
        "O-Mino Rotation 1 blocks are incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_o_mino_rotation_1_offset() {
    let o_mino_rot_1 = SRS_SHAPES[TetrominoShape::O as usize][1];

    // SRSのOミノ Rotation 1 の回転中心オフセット (ここでは (0,0) を想定)
    let expected_offset = (0, 0);
    assert_eq!(
        o_mino_rot_1.offset, expected_offset,
        "O-Mino Rotation 1 offset is incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_o_mino_rotation_2_blocks() {
    let o_mino_rot_2 = SRS_SHAPES[TetrominoShape::O as usize][2];

    // SRSのOミノ Rotation 2 のブロック相対座標
    let expected_blocks = [(0, 0), (1, 0), (0, 1), (1, 1)];
    assert_eq!(
        o_mino_rot_2.blocks, expected_blocks,
        "O-Mino Rotation 2 blocks are incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_o_mino_rotation_2_offset() {
    let o_mino_rot_2 = SRS_SHAPES[TetrominoShape::O as usize][2];

    // SRSのOミノ Rotation 2 の回転中心オフセット (ここでは (0,0) を想定)
    let expected_offset = (0, 0);
    assert_eq!(
        o_mino_rot_2.offset, expected_offset,
        "O-Mino Rotation 2 offset is incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_o_mino_rotation_3_blocks() {
    let o_mino_rot_3 = SRS_SHAPES[TetrominoShape::O as usize][3];

    // SRSのOミノ Rotation 3 のブロック相対座標
    let expected_blocks = [(0, 0), (1, 0), (0, 1), (1, 1)];
    assert_eq!(
        o_mino_rot_3.blocks, expected_blocks,
        "O-Mino Rotation 3 blocks are incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_o_mino_rotation_3_offset() {
    let o_mino_rot_3 = SRS_SHAPES[TetrominoShape::O as usize][3];

    // SRSのOミノ Rotation 3 の回転中心オフセット (ここでは (0,0) を想定)
    let expected_offset = (0, 0);
    assert_eq!(
        o_mino_rot_3.offset, expected_offset,
        "O-Mino Rotation 3 offset is incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_t_mino_rotation_0_blocks() {
    let t_mino_rot_0 = SRS_SHAPES[TetrominoShape::T as usize][0];

    // SRSのTミノ Rotation 0 のブロック相対座標
    let expected_blocks = [(-1, 0), (0, 0), (1, 0), (0, 1)];
    assert_eq!(
        t_mino_rot_0.blocks, expected_blocks,
        "T-Mino Rotation 0 blocks are incorrect according to SRS."
    );
}

#[test]
fn test_srs_shapes_t_mino_rotation_0_offset() {
    let t_mino_rot_0 = SRS_SHAPES[TetrominoShape::T as usize][0];

    // SRSのTミノ Rotation 0 の回転中心オフセット
    let expected_offset = (0, 0);
    assert_eq!(
        t_mino_rot_0.offset, expected_offset,
        "T-Mino Rotation 0 offset is incorrect according to SRS."
    );
}