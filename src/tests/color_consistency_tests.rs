// Phase 5 Red: 色の一貫性テスト
// SRS Wall Kick回転における色位置の保持テスト

use crate::tetromino::{Tetromino, TetrominoShape};
use crossterm::style::Color;
use std::collections::HashMap;

/// 色位置の検証用ヘルパー関数
fn get_color_positions(piece: &Tetromino) -> HashMap<Color, Vec<(i8, i8)>> {
    let mut color_map = HashMap::new();
    for (pos, color) in piece.iter_blocks() {
        color_map.entry(color).or_insert_with(Vec::new).push(pos);
    }
    color_map
}

/// SRS回転後の色の一貫性をテスト
#[test]
fn test_srs_rotation_preserves_color_positions() {
    // 特定の色パターンでテトロミノを作成
    let colors = [Color::Red, Color::Blue, Color::Green, Color::Yellow];
    let t_piece = Tetromino::from_shape(TetrominoShape::T, colors);
    
    // 初期色位置を記録
    let initial_colors = get_color_positions(&t_piece);
    
    // SRS回転を実行
    let rotated_piece = t_piece.rotated();
    let rotated_colors = get_color_positions(&rotated_piece);
    
    // 色の種類が変わらないことを確認
    assert_eq!(initial_colors.keys().collect::<std::collections::HashSet<_>>(),
               rotated_colors.keys().collect::<std::collections::HashSet<_>>(),
               "回転後に色の種類が変わってはいけない");
    
    // 各色のブロック数が同じであることを確認
    for color in initial_colors.keys() {
        assert_eq!(initial_colors[color].len(), rotated_colors[color].len(),
                   "色 {:?} のブロック数が回転前後で変わってはいけない", color);
    }
}

/// Wall Kick回転での色一貫性テスト
#[test]
fn test_wall_kick_preserves_color_positions() {
    // 壁際に近い位置でWall Kickを誘発するテトロミノを作成
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::White];
    let mut j_piece = Tetromino::from_shape(TetrominoShape::J, colors);
    
    // 壁際の位置に配置してWall Kickを誘発
    j_piece.pos = (1, 5); // 境界近くの位置
    
    let initial_colors = get_color_positions(&j_piece);
    
    // Wall Kick回転を実行
    let wall_kicked_piece = j_piece.rotated_with_wall_kick();
    let final_colors = get_color_positions(&wall_kicked_piece);
    
    // 色の一貫性を確認
    assert_eq!(initial_colors.len(), final_colors.len(),
               "Wall Kick後の色数が変わってはいけない");
    
    for color in initial_colors.keys() {
        assert!(final_colors.contains_key(color),
                "Wall Kick後に色 {:?} が失われてはいけない", color);
        assert_eq!(initial_colors[color].len(), final_colors[color].len(),
                   "Wall Kick後に色 {:?} のブロック数が変わってはいけない", color);
    }
}

/// 複雑な回転シーケンスでの色保持テスト
#[test]
fn test_complex_rotation_sequence_color_preservation() {
    let colors = [Color::Red, Color::Blue, Color::Green, Color::Yellow];
    let mut l_piece = Tetromino::from_shape(TetrominoShape::L, colors);
    
    // 初期色状態を記録
    let initial_colors = get_color_positions(&l_piece);
    
    // 複雑な回転シーケンスを実行: 時計回り2回、反時計回り1回、Wall Kick回転1回
    l_piece = l_piece.rotated();
    l_piece = l_piece.rotated(); 
    l_piece = l_piece.rotated_counter_clockwise();
    l_piece = l_piece.rotated_with_wall_kick();
    
    let final_colors = get_color_positions(&l_piece);
    
    // 色の種類と数が保持されていることを確認
    assert_eq!(initial_colors.keys().len(), final_colors.keys().len(),
               "複雑な回転シーケンス後の色種類数が変わってはいけない");
    
    for color in initial_colors.keys() {
        assert!(final_colors.contains_key(color),
                "複雑な回転シーケンス後に色 {:?} が失われてはいけない", color);
        assert_eq!(initial_colors[color].len(), final_colors[color].len(),
                   "複雑な回転シーケンス後に色 {:?} のブロック数が変わってはいけない", color);
    }
}

/// O型テトロミノの色回転一貫性テスト
#[test]
fn test_o_mino_color_rotation_consistency() {
    let colors = [Color::Red, Color::Blue, Color::Green, Color::Yellow];
    let o_piece = Tetromino::from_shape(TetrominoShape::O, colors);
    
    // O型は特別な色回転ロジックを持つ
    let initial_colors = o_piece.get_colors();
    
    // 1回転
    let rotated_once = o_piece.rotated();
    let rotated_colors = rotated_once.get_colors();
    
    // 4回転で元に戻ることを確認
    let mut piece = o_piece.clone();
    for _ in 0..4 {
        piece = piece.rotated();
    }
    let full_rotation_colors = piece.get_colors();
    
    // 4回転後は元の色配置に戻る
    assert_eq!(initial_colors, full_rotation_colors,
               "O型テトロミノは4回転後に元の色配置に戻らなければならない");
    
    // 各回転で色の種類は保持される
    let initial_color_set: std::collections::HashSet<_> = initial_colors.iter().collect();
    let rotated_color_set: std::collections::HashSet<_> = rotated_colors.iter().collect();
    
    assert_eq!(initial_color_set, rotated_color_set,
               "O型テトロミノの回転で色の種類が変わってはいけない");
}

/// I型テトロミノの特殊Wall Kickでの色保持テスト
#[test]
fn test_i_mino_special_wall_kick_color_preservation() {
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::White];
    let mut i_piece = Tetromino::from_shape(TetrominoShape::I, colors);
    
    // I型のWall Kickを誘発する位置
    i_piece.pos = (0, 5); // 境界の位置
    
    let initial_colors = get_color_positions(&i_piece);
    
    // I型特有のSRS Wall Kickオフセットでの回転
    let wall_kicked_piece = i_piece.rotated_with_wall_kick();
    let final_colors = get_color_positions(&wall_kicked_piece);
    
    // I型テトロミノの色一貫性確認
    assert_eq!(initial_colors.len(), final_colors.len(),
               "I型テトロミノのWall Kick後の色数が変わってはいけない");
    
    for color in initial_colors.keys() {
        assert!(final_colors.contains_key(color),
                "I型テトロミノのWall Kick後に色 {:?} が失われてはいけない", color);
        assert_eq!(initial_colors[color].len(), final_colors[color].len(),
                   "I型テトロミノのWall Kick後に色 {:?} のブロック数が変わってはいけない", color);
    }
}

/// T型テトロミノの時計回り回転で物理的位置と色が正しく回転することを確認
#[test]
fn test_t_mino_clockwise_rotation_block_color_preservation() {
    let colors = [Color::Yellow, Color::Cyan, Color::Magenta, Color::Green];
    let mut t_piece = Tetromino::from_shape(TetrominoShape::T, colors);
    t_piece.pos = (2, 2); // 中央に配置
    
    // 初期状態: State 0
    // 相対座標での配置:
    //   Y  (1,0) - 上部
    // C M G (0,1), (1,1), (2,1) - 左・中央・右
    let initial_blocks: Vec<_> = t_piece.iter_blocks().collect();
    let yellow_pos = initial_blocks.iter().find(|(_, color)| *color == Color::Yellow).unwrap().0;
    let cyan_pos = initial_blocks.iter().find(|(_, color)| *color == Color::Cyan).unwrap().0;
    let magenta_pos = initial_blocks.iter().find(|(_, color)| *color == Color::Magenta).unwrap().0;
    let green_pos = initial_blocks.iter().find(|(_, color)| *color == Color::Green).unwrap().0;
    
    println!("\n=== Initial positions ===");
    println!("Yellow: {:?}, Cyan: {:?}, Magenta: {:?}, Green: {:?}", 
             yellow_pos, cyan_pos, magenta_pos, green_pos);
    
    // 1回転後: State 1 (右向きT)
    // 期待される配置:
    //   Y  (1,0) - 上部（Yellowはそのまま）
    //   M  (1,1) - 中央（Magentaはそのまま） 
    //   G  (2,1) - 右側（Greenはそのまま）
    //   C  (1,2) - 下部（Cyanが左から下へ移動）
    t_piece = t_piece.rotated();
    
    let rotated_blocks: Vec<_> = t_piece.iter_blocks().collect();
    let yellow_pos_rotated = rotated_blocks.iter().find(|(_, color)| *color == Color::Yellow).unwrap().0;
    let cyan_pos_rotated = rotated_blocks.iter().find(|(_, color)| *color == Color::Cyan).unwrap().0;
    let magenta_pos_rotated = rotated_blocks.iter().find(|(_, color)| *color == Color::Magenta).unwrap().0;
    let green_pos_rotated = rotated_blocks.iter().find(|(_, color)| *color == Color::Green).unwrap().0;
    
    println!("\n=== After rotation ===");
    println!("Yellow: {:?}, Cyan: {:?}, Magenta: {:?}, Green: {:?}", 
             yellow_pos_rotated, cyan_pos_rotated, magenta_pos_rotated, green_pos_rotated);
    
    // 期待される物理的位置の変化を確認
    assert_eq!(yellow_pos, yellow_pos_rotated, "Yellow should stay at the same position (top)");
    assert_eq!(magenta_pos, magenta_pos_rotated, "Magenta should stay at the same position (center)");
    assert_eq!(green_pos, green_pos_rotated, "Green should stay at the same position (right)");
    
    // Cyanは左側(2,3)から下部(3,4)に移動
    assert_eq!(cyan_pos_rotated, (cyan_pos.0 + 1, cyan_pos.1 + 1), 
               "Cyan should move from left to bottom");
    
    println!("\n=== Test passed: Physical rotation is correct! ===");
}

/// 全テトロミノ形状での色一貫性テスト
#[test]
fn test_all_shapes_color_consistency() {
    let colors = [Color::Red, Color::Blue, Color::Green, Color::Yellow];
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
        let initial_colors = get_color_positions(&piece);
        
        // 各種回転でのテスト
        let rotated = piece.rotated();
        let rotated_colors = get_color_positions(&rotated);
        
        let counter_rotated = piece.rotated_counter_clockwise();
        let counter_colors = get_color_positions(&counter_rotated);
        
        let wall_kicked = piece.rotated_with_wall_kick();
        let wall_kick_colors = get_color_positions(&wall_kicked);
        
        // 全ての回転で色の一貫性を確認
        for test_colors in [&rotated_colors, &counter_colors, &wall_kick_colors] {
            assert_eq!(initial_colors.len(), test_colors.len(),
                       "形状 {:?} で色数が変わってはいけない", shape);
            
            for color in initial_colors.keys() {
                assert!(test_colors.contains_key(color),
                        "形状 {:?} で色 {:?} が失われてはいけない", shape, color);
                assert_eq!(initial_colors[color].len(), test_colors[color].len(),
                           "形状 {:?} で色 {:?} のブロック数が変わってはいけない", shape, color);
            }
        }
    }
}