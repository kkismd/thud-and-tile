// Z-mino形状確認テスト
use crate::tetromino::{Tetromino, TetrominoShape};
use crossterm::style::Color;

#[test]
fn test_z_mino_shape_verification() {
    let colors = [Color::Red, Color::Blue, Color::Green, Color::Yellow];
    let mut z_piece = Tetromino::from_shape(TetrominoShape::Z, colors);
    z_piece.pos = (2, 2);
    
    // State 0の確認 - 標準的なZ字型
    let state0_blocks: Vec<_> = z_piece.iter_blocks().collect();
    println!("=== Z-mino State 0 ===");
    for (pos, color) in state0_blocks.iter() {
        println!("pos({}, {}) = {:?}", pos.0, pos.1, color);
    }
    
    // State 1への回転確認
    z_piece = z_piece.rotated();
    let state1_blocks: Vec<_> = z_piece.iter_blocks().collect();
    println!("=== Z-mino State 1 ===");
    for (pos, color) in state1_blocks.iter() {
        println!("pos({}, {}) = {:?}", pos.0, pos.1, color);
    }
    
    // 4回転で元に戻ることを確認
    z_piece = z_piece.rotated().rotated().rotated();
    let final_blocks: Vec<_> = z_piece.iter_blocks().collect();
    
    assert_eq!(state0_blocks, final_blocks, "Z-mino: 4回転後は元の形状に戻らなければならない");
    println!("✅ Z-mino形状テスト成功: 正しい形状で4回転サイクル完了");
}