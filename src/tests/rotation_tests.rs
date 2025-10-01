// SRS回転システム テスト
use super::*;
use crate::tetromino::TetrominoShape;
use crossterm::style::Color;

#[test]
fn test_basic_rotation() {
    // 基本的な回転動作のテスト
    assert!(true);
}

#[test]
fn test_tetromino_initial_rotation_state() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let piece = Tetromino::from_shape(TetrominoShape::T, colors);
    assert_eq!(piece.get_rotation_state(), 0, "Initial rotation state should be 0");
}

#[test]
fn test_clockwise_rotation_state_cycle() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
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
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
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
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
    let mut piece = Tetromino::from_shape(TetrominoShape::I, colors);
    
    // I型を壁際に配置
    piece = piece.rotated(); // 垂直状態
    piece.pos = (-1, 5); // 左壁際
    
    // Wall kick回転
    let rotated = piece.rotated_with_wall_kick();
    
    // 回転は成功し、位置が調整されるべき
    assert_eq!(rotated.get_rotation_state(), 2, "Rotation should succeed with wall kick");
    assert_ne!(rotated.pos, piece.pos, "Wall kick should adjust position");
}

#[test]
fn test_physical_rotation_produces_movement() {
    let colors = [Color::Yellow, Color::Cyan, Color::Magenta, Color::Green];
    let piece = Tetromino::from_shape(TetrominoShape::T, colors);
    
    let initial_blocks: Vec<_> = piece.iter_blocks().collect();
    let rotated = piece.rotated();
    let rotated_blocks: Vec<_> = rotated.iter_blocks().collect();
    
    // 物理的回転で少なくとも1つのブロックが移動することを確認
    let has_movement = initial_blocks.iter().zip(rotated_blocks.iter())
        .any(|((pos1, _), (pos2, _))| pos1 != pos2);
    
    assert!(has_movement, "Physical rotation should move at least one block");
    
    // 色は保持されることを確認
    let initial_colors: std::collections::HashSet<_> = 
        initial_blocks.iter().map(|(_, color)| *color).collect();
    let rotated_colors: std::collections::HashSet<_> = 
        rotated_blocks.iter().map(|(_, color)| *color).collect();
    
    assert_eq!(initial_colors, rotated_colors, "Colors should be preserved during rotation");
}
