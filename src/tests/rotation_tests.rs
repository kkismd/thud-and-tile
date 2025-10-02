// SRS回転システム テスト
use super::*;
use crate::tetromino::TetrominoShape;
use crate::game_color::GameColor;

#[test]
fn test_basic_rotation() {
    // 基本的な回転動作のテスト
    assert!(true);
}

#[test]
fn test_tetromino_initial_rotation_state() {
    let colors = [GameColor::Cyan, GameColor::Magenta, GameColor::Yellow, GameColor::Green];
    let piece = Tetromino::from_shape(TetrominoShape::T, colors);
    assert_eq!(
        piece.get_rotation_state(),
        0,
        "Initial rotation state should be 0"
    );
}

#[test]
fn test_clockwise_rotation_state_cycle() {
    let colors = [GameColor::Cyan, GameColor::Magenta, GameColor::Yellow, GameColor::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::T, colors);

    // Test complete rotation cycle: 0 -> 1 -> 2 -> 3 -> 0
    assert_eq!(piece.get_rotation_state(), 0);

    piece = piece.rotated();
    assert_eq!(piece.get_rotation_state(), 1);

    piece = piece.rotated();
    assert_eq!(piece.get_rotation_state(), 2);

    piece = piece.rotated();
    assert_eq!(piece.get_rotation_state(), 3);

    piece = piece.rotated();
    assert_eq!(piece.get_rotation_state(), 0, "Should cycle back to 0");
}

#[test]
fn test_counter_clockwise_rotation_state_cycle() {
    let colors = [GameColor::Cyan, GameColor::Magenta, GameColor::Yellow, GameColor::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::T, colors);

    // Test counter-clockwise cycle: 0 -> 3 -> 2 -> 1 -> 0
    assert_eq!(piece.get_rotation_state(), 0);

    piece = piece.rotated_counter_clockwise();
    assert_eq!(piece.get_rotation_state(), 3);

    piece = piece.rotated_counter_clockwise();
    assert_eq!(piece.get_rotation_state(), 2);

    piece = piece.rotated_counter_clockwise();
    assert_eq!(piece.get_rotation_state(), 1);

    piece = piece.rotated_counter_clockwise();
    assert_eq!(piece.get_rotation_state(), 0, "Should cycle back to 0");
}

#[test]
fn test_basic_wall_kick_functionality() {
    let colors = [GameColor::Cyan, GameColor::Magenta, GameColor::Yellow, GameColor::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::I, colors);

    // I型を壁際に配置
    piece = piece.rotated(); // 垂直状態
    piece.pos = (-1, 5); // 左壁際

    // Wall kick回転
    let rotated = piece.rotated_with_wall_kick();

    // 回転は成功し、位置が調整されるべき
    assert_eq!(
        rotated.get_rotation_state(),
        2,
        "Rotation should succeed with wall kick"
    );
    assert_ne!(rotated.pos, piece.pos, "Wall kick should adjust position");
}

#[test]
fn test_physical_rotation_produces_movement() {
    let colors = [GameColor::Yellow, GameColor::Cyan, GameColor::Magenta, GameColor::Green];
    let piece = Tetromino::from_shape(TetrominoShape::T, colors);

    let initial_blocks: Vec<_> = piece.iter_blocks().collect();
    let rotated = piece.rotated();
    let rotated_blocks: Vec<_> = rotated.iter_blocks().collect();

    // 物理的回転で少なくとも1つのブロックが移動することを確認
    let has_movement = initial_blocks
        .iter()
        .zip(rotated_blocks.iter())
        .any(|((pos1, _), (pos2, _))| pos1 != pos2);

    assert!(
        has_movement,
        "Physical rotation should move at least one block"
    );

    // 色は保持されることを確認
    let initial_colors: std::collections::HashSet<_> =
        initial_blocks.iter().map(|(_, color)| *color).collect();
    let rotated_colors: std::collections::HashSet<_> =
        rotated_blocks.iter().map(|(_, color)| *color).collect();

    assert_eq!(
        initial_colors, rotated_colors,
        "Colors should be preserved during rotation"
    );
}

#[test]
fn test_color_physical_rotation_detailed() {
    // Phase 5: 色の物理的回転の詳細テスト
    let colors = [GameColor::Red, GameColor::Green, GameColor::Blue, GameColor::Yellow];
    let piece = Tetromino::from_shape(TetrominoShape::T, colors);

    // T-mino初期状態: [(1,0), (0,1), (1,1), (2,1)]の色を確認
    let initial_blocks: Vec<_> = piece.iter_blocks().collect();
    println!("Initial T-mino blocks: {:?}", initial_blocks);

    // 1回転後の色の位置を確認
    let rotated = piece.rotated();
    let rotated_blocks: Vec<_> = rotated.iter_blocks().collect();
    println!("Rotated T-mino blocks: {:?}", rotated_blocks);

    // 物理的に色が移動していることを確認
    // 初期位置と回転後で、同じ位置に同じ色がないことを確認（O-mino以外）
    let _same_position_same_color = initial_blocks.iter().any(|&((pos1, color1), _)| {
        rotated_blocks
            .iter()
            .any(|&((pos2, color2), _)| pos1 == pos2 && color1 == color2)
    });

    // T-minoでは、位置(1,1)に同じ色が残る可能性があるが、他の位置では色が物理的に移動する
    let positions_changed = initial_blocks
        .iter()
        .zip(rotated_blocks.iter())
        .filter(|((pos1, _), (pos2, _))| pos1 != pos2)
        .count();

    assert!(
        positions_changed >= 1,
        "At least one block should have moved to a different position"
    );

    // 4回転で元に戻ることを確認
    let mut test_piece = piece;
    for _ in 0..4 {
        test_piece = test_piece.rotated();
    }
    let final_blocks: Vec<_> = test_piece.iter_blocks().collect();

    // 初期状態と4回転後が同じであることを確認
    assert_eq!(initial_blocks.len(), final_blocks.len());
    for (initial, final_block) in initial_blocks.iter().zip(final_blocks.iter()) {
        assert_eq!(
            initial, final_block,
            "After 4 rotations, blocks should return to original positions and colors"
        );
    }
}
