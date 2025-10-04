use crate::game_color::GameColor;
use crate::random::{RandomProvider, create_default_random_provider};
use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::config::{BOARD_WIDTH, COLOR_PALETTE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TetrominoShape {
    I,
    O,
    T,
    L,
    J,
    S,
    Z,
}

// SRS Standard Wall Kick Offset Tables
// Phase 4 Refactor: Static tables for performance optimization

/// SRS offset table for J, L, T, S, Z tetrominoes
/// Index corresponds to transition: [0->1, 1->0, 1->2, 2->1, 2->3, 3->2, 3->0, 0->3]
pub const SRS_JLTSZ_OFFSETS: [[[i8; 2]; 5]; 8] = [
    // 0->1 transition
    [[0, 0], [-1, 0], [-1, 1], [0, -2], [-1, -2]],
    // 1->0 transition
    [[0, 0], [1, 0], [1, -1], [0, 2], [1, 2]],
    // 1->2 transition
    [[0, 0], [1, 0], [1, -1], [0, 2], [1, 2]],
    // 2->1 transition
    [[0, 0], [-1, 0], [-1, 1], [0, -2], [-1, -2]],
    // 2->3 transition
    [[0, 0], [1, 0], [1, 1], [0, -2], [1, -2]],
    // 3->2 transition
    [[0, 0], [-1, 0], [-1, -1], [0, 2], [-1, 2]],
    // 3->0 transition
    [[0, 0], [-1, 0], [-1, -1], [0, 2], [-1, 2]],
    // 0->3 transition
    [[0, 0], [1, 0], [1, 1], [0, -2], [1, -2]],
];

/// SRS offset table for I tetromino
/// Index corresponds to transition: [0->1, 1->0, 1->2, 2->1, 2->3, 3->2, 3->0, 0->3]
pub const SRS_I_OFFSETS: [[[i8; 2]; 5]; 8] = [
    // 0->1 transition
    [[0, 0], [-2, 0], [1, 0], [-2, -1], [1, 2]],
    // 1->0 transition
    [[0, 0], [2, 0], [-1, 0], [2, 1], [-1, -2]],
    // 1->2 transition
    [[0, 0], [-1, 0], [2, 0], [-1, 2], [2, -1]],
    // 2->1 transition
    [[0, 0], [1, 0], [-2, 0], [1, -2], [-2, 1]],
    // 2->3 transition
    [[0, 0], [2, 0], [-1, 0], [2, 1], [-1, -2]],
    // 3->2 transition
    [[0, 0], [-2, 0], [1, 0], [-2, -1], [1, 2]],
    // 3->0 transition
    [[0, 0], [1, 0], [-2, 0], [1, -2], [-2, 1]],
    // 0->3 transition
    [[0, 0], [-1, 0], [2, 0], [-1, 2], [2, -1]],
];

/// Convert rotation state transition to offset table index
/// Phase 4 Refactor: Optimized lookup function
pub const fn get_transition_index(from_state: u8, to_state: u8) -> usize {
    match (from_state, to_state) {
        (0, 1) => 0,
        (1, 0) => 1,
        (1, 2) => 2,
        (2, 1) => 3,
        (2, 3) => 4,
        (3, 2) => 5,
        (3, 0) => 6,
        (0, 3) => 7,
        _ => 0, // Default fallback
    }
}

/// Web版で使用するための独立したSRS wall kick offsets関数
/// 形状を数値で指定してSRSオフセットを取得
pub fn get_srs_wall_kick_offsets_by_shape(shape: u8, from_state: u8, to_state: u8) -> &'static [[i8; 2]; 5] {
    let index = get_transition_index(from_state, to_state);

    match shape {
        0 => &SRS_I_OFFSETS[index], // I piece
        1 => {
            // O piece doesn't need wall kicks (rotates in place)
            static O_OFFSETS: [[i8; 2]; 5] = [[0, 0], [0, 0], [0, 0], [0, 0], [0, 0]];
            &O_OFFSETS
        }
        _ => &SRS_JLTSZ_OFFSETS[index], // J, L, T, S, Z pieces
    }
}

impl TetrominoShape {
    pub fn all_shapes() -> Vec<TetrominoShape> {
        vec![
            TetrominoShape::I,
            TetrominoShape::O,
            TetrominoShape::T,
            TetrominoShape::L,
            TetrominoShape::J,
            TetrominoShape::S,
            TetrominoShape::Z,
        ]
    }
}

pub struct TetrominoBag {
    bag: Vec<TetrominoShape>,
    random_provider: crate::random::RandomProviderImpl,
}

impl TetrominoBag {
    pub fn new() -> Self {
        let mut bag = TetrominoShape::all_shapes();
        let mut random_provider = create_default_random_provider();
        
        random_provider.shuffle(&mut bag);
        TetrominoBag { bag, random_provider }
    }

    pub fn next(&mut self) -> TetrominoShape {
        if self.bag.is_empty() {
            self.bag = TetrominoShape::all_shapes();
            self.random_provider.shuffle(&mut self.bag);
        }
        self.bag.pop().unwrap()
    }
}

lazy_static! {
    static ref TETROMINO_BAG: Mutex<TetrominoBag> = Mutex::new(TetrominoBag::new());
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tetromino {
    pub shape: TetrominoShape, // Made public for SRS testing
    pub pos: (i8, i8),
    blocks: Vec<((i8, i8), GameColor)>,
    rotation_state: u8, // SRS rotation state: 0, 1, 2, 3
}

impl Tetromino {
    pub fn new_random() -> Self {
        let shape = TETROMINO_BAG.lock().unwrap().next();

        // Loop until a valid coloring is found
        loop {
            let mut provider = create_default_random_provider();
            let colors = [
                *provider.choose(&COLOR_PALETTE).unwrap(),
                *provider.choose(&COLOR_PALETTE).unwrap(),
                *provider.choose(&COLOR_PALETTE).unwrap(),
                *provider.choose(&COLOR_PALETTE).unwrap(),
            ];

            let tetromino = Self::from_shape(shape, colors);

            // Check for adjacency validity
            let blocks = &tetromino.blocks;
            let mut is_valid = true;
            'outer: for i in 0..blocks.len() {
                for j in (i + 1)..blocks.len() {
                    let (pos1, color1) = blocks[i];
                    let (pos2, color2) = blocks[j];

                    let is_adjacent = (pos1.0 - pos2.0).abs() + (pos1.1 - pos2.1).abs() == 1;

                    if is_adjacent && color1 == color2 {
                        is_valid = false;
                        break 'outer;
                    }
                }
            }

            if is_valid {
                return tetromino;
            }
        }
    }

    pub fn from_shape(shape: TetrominoShape, colors: [GameColor; 4]) -> Self {
        let matrix = match shape {
            TetrominoShape::I => &Self::SHAPES[0],
            TetrominoShape::O => &Self::SHAPES[1],
            TetrominoShape::T => &Self::SHAPES[2],
            TetrominoShape::L => &Self::SHAPES[3],
            TetrominoShape::J => &Self::SHAPES[4],
            TetrominoShape::S => &Self::SHAPES[5],
            _ => &Self::SHAPES[6],
        };
        let mut blocks = Vec::new();
        for (i, &(block_x, block_y)) in matrix[0].iter().enumerate() {
            blocks.push(((block_x, block_y), colors[i]));
        }

        Tetromino {
            shape,
            pos: ((BOARD_WIDTH as i8) / 2 - 2, 0),
            blocks,
            rotation_state: 0, // Initial rotation state
        }
    }

    pub fn iter_blocks(&self) -> impl Iterator<Item = ((i8, i8), GameColor)> + '_ {
        self.blocks.iter().map(move |&((block_x, block_y), color)| {
            let pos = (self.pos.0 + block_x, self.pos.1 + block_y);
            (pos, color)
        })
    }

    /// Gets the colors of the blocks in order
    /// Used for testing color consistency during rotations
    #[allow(dead_code)]
    pub fn get_colors(&self) -> Vec<GameColor> {
        self.blocks.iter().map(|(_, color)| *color).collect()
    }

    /// Gets the current rotation state (0, 1, 2, 3).
    /// This method is primarily used for testing SRS compliance.
    #[allow(dead_code)]
    pub fn get_rotation_state(&self) -> u8 {
        self.rotation_state
    }

    /// Rotates the tetromino with wall kick functionality
    /// Attempts normal rotation first, then tries SRS standard wall kick offsets
    /// Phase 4: Complete SRS standard wall kick implementation
    #[allow(dead_code)]
    pub fn rotated_with_wall_kick(&self) -> Self {
        let original_state = self.get_rotation_state();
        let target_state = (original_state + 1) % 4;

        // Get SRS wall kick offset table for this transition
        let offsets = self.get_srs_wall_kick_offsets(original_state, target_state);

        // Try each offset in order until one works or all fail
        for &[offset_x, offset_y] in offsets {
            let mut candidate_piece = self.rotated();
            candidate_piece.pos = (
                candidate_piece.pos.0 + offset_x,
                candidate_piece.pos.1 + offset_y,
            );

            // For now, check basic bounds (in full game would check board collision)
            if self.is_position_valid(&candidate_piece) {
                return candidate_piece;
            }
        }
        // If all offsets fail, return normal rotation (in real game might fail)
        self.rotated()
    }

    /// Get SRS standard wall kick offsets for a rotation transition
    /// Phase 4 Refactor: Optimized with static table lookup
    pub fn get_srs_wall_kick_offsets(&self, from_state: u8, to_state: u8) -> &'static [[i8; 2]; 5] {
        let index = get_transition_index(from_state, to_state);

        match self.shape {
            TetrominoShape::I => &SRS_I_OFFSETS[index],
            TetrominoShape::O => {
                // O-mino doesn't need wall kicks (rotates in place)
                static O_OFFSETS: [[i8; 2]; 5] = [[0, 0], [0, 0], [0, 0], [0, 0], [0, 0]];
                &O_OFFSETS
            }
            _ => &SRS_JLTSZ_OFFSETS[index],
        }
    }

    /// Check if a piece position is valid (basic bounds checking)
    /// In full game would also check board collision
    #[allow(dead_code)]
    fn is_position_valid(&self, piece: &Self) -> bool {
        const BOARD_WIDTH: i8 = 10;
        const BOARD_HEIGHT: i8 = 20;

        for ((block_x, block_y), _) in piece.iter_blocks() {
            if block_x < 0 || block_x >= BOARD_WIDTH || block_y < 0 || block_y >= BOARD_HEIGHT {
                return false;
            }
        }
        true
    }

    pub fn moved(&self, dx: i8, dy: i8) -> Self {
        let mut new_piece = self.clone();
        new_piece.pos = (self.pos.0 + dx, self.pos.1 + dy);
        new_piece
    }

    pub fn rotated(&self) -> Self {
        let mut new_piece = self.clone();

        // Use SRS standard rotation data
        let shape_index = match self.shape {
            TetrominoShape::I => 0,
            TetrominoShape::O => 1,
            TetrominoShape::T => 2,
            TetrominoShape::L => 3,
            TetrominoShape::J => 4,
            TetrominoShape::S => 5,
            TetrominoShape::Z => 6,
        };

        let next_rotation_state = (self.rotation_state + 1) % 4;
        let next_state_blocks = &Self::SHAPES[shape_index][next_rotation_state as usize];

        // Apply rotation using SRS standard coordinates
        new_piece.blocks = next_state_blocks
            .iter()
            .enumerate()
            .map(|(i, &(x, y))| {
                // All pieces use the same rotation mapping: colors follow blocks
                let color = self.get_rotated_color_mapping(i, self.rotation_state, next_rotation_state);
                ((x, y), color)
            })
            .collect();

        new_piece.rotation_state = next_rotation_state;
        new_piece
    }

    /// Get the color mapping for rotation transitions
    /// With physical rotation order in SHAPES, all tetrominoes use direct 1:1 mapping
    fn get_rotated_color_mapping(&self, new_index: usize, _from_state: u8, _to_state: u8) -> GameColor {
        // All tetrominoes use direct mapping due to physical rotation order in SHAPES
        // Colors follow blocks naturally through the rotation sequence
        self.blocks[new_index].1
    }

    pub fn rotated_counter_clockwise(&self) -> Self {
        let mut new_piece = self.clone();

        // Use SRS standard rotation data
        let shape_index = match self.shape {
            TetrominoShape::I => 0,
            TetrominoShape::O => 1,
            TetrominoShape::T => 2,
            TetrominoShape::L => 3,
            TetrominoShape::J => 4,
            TetrominoShape::S => 5,
            TetrominoShape::Z => 6,
        };

        let next_rotation_state = (self.rotation_state + 3) % 4; // +3 is equivalent to -1 in modulo 4
        let next_state_blocks = &Self::SHAPES[shape_index][next_rotation_state as usize];

        // Apply rotation using SRS standard coordinates
        new_piece.blocks = next_state_blocks
            .iter()
            .enumerate()
            .map(|(i, &(x, y))| {
                // All pieces use the same rotation mapping: colors follow blocks
                let color = self.get_rotated_color_mapping(i, self.rotation_state, next_rotation_state);
                ((x, y), color)
            })
            .collect();

        new_piece.rotation_state = next_rotation_state;
        new_piece
    }

    const SHAPES: [[[(i8, i8); 4]; 4]; 7] = [
        // I - SRS standard coordinates with physical rotation order
        [
            [(0, 1), (1, 1), (2, 1), (3, 1)], // State 0: horizontal - A,B,C,D
            [(2, 3), (2, 2), (2, 1), (2, 0)], // State 1: vertical - D,C,B,A (physical rotation)
            [(3, 2), (2, 2), (1, 2), (0, 2)], // State 2: horizontal - A,B,C,D (continue rotation)
            [(1, 0), (1, 1), (1, 2), (1, 3)], // State 3: vertical - D,C,B,A (back to original)
        ],
        // O - SRS standard coordinates with True Rotation (wobble effect)
        [
            [(1, 1), (2, 1), (1, 2), (2, 2)], // State 0: TL,TR,BL,BR - A,B,C,D
            [(2, 1), (2, 2), (1, 1), (1, 2)], // State 1: clockwise rotation - B,D,A,C
            [(2, 2), (1, 2), (2, 1), (1, 1)], // State 2: 180 degree rotation - D,C,B,A
            [(1, 2), (1, 1), (2, 2), (2, 1)], // State 3: counter-clockwise - C,A,D,B
        ],
        // T - SRS standard coordinates with physical rotation order
        [
            [(1, 0), (0, 1), (1, 1), (2, 1)], // State 0: upward T - A,B,C,D
            [(2, 1), (1, 0), (1, 1), (1, 2)], // State 1: rightward T - D,A,C,B (physical rotation)
            [(1, 2), (2, 1), (1, 1), (0, 1)], // State 2: downward T - B,D,C,A (continue rotation)
            [(0, 1), (1, 2), (1, 1), (1, 0)], // State 3: leftward T - A,B,C,D (back to original)
        ],
        // L - SRS standard coordinates with physical rotation order
        [
            [(2, 0), (0, 1), (1, 1), (2, 1)], // State 0: A,B,C,D
            [(2, 2), (1, 0), (1, 1), (1, 2)], // State 1: A,C,D,B (physical rotation)
            [(0, 2), (2, 1), (1, 1), (0, 1)], // State 2: A,D,C,B (continue rotation)
            [(0, 0), (1, 2), (1, 1), (1, 0)], // State 3: A,B,C,D (back to original)
        ],
        // J - SRS standard coordinates with physical rotation order
        [
            [(0, 0), (0, 1), (1, 1), (2, 1)], // State 0: A,B,C,D
            [(2, 0), (1, 0), (1, 1), (1, 2)], // State 1: A,B,C,D (physical rotation)
            [(2, 2), (2, 1), (1, 1), (0, 1)], // State 2: A,D,C,B (continue rotation)
            [(0, 2), (1, 2), (1, 1), (1, 0)], // State 3: A,D,C,B (back to original)
        ],
        // S - SRS standard coordinates with physical rotation order
        [
            [(1, 0), (2, 0), (0, 1), (1, 1)], // State 0: A,B,C,D
            [(2, 1), (2, 2), (1, 0), (1, 1)], // State 1: A,B,C,D (physical rotation)
            [(1, 2), (0, 2), (2, 1), (1, 1)], // State 2: A,B,C,D (continue rotation)
            [(0, 1), (0, 0), (1, 2), (1, 1)], // State 3: A,B,C,D (back to original)
        ],
        // Z - SRS standard coordinates with physical rotation order
        [
            [(0, 0), (1, 0), (1, 1), (2, 1)], // State 0: A,B,C,D - standard Z shape
            [(2, 0), (2, 1), (1, 1), (1, 2)], // State 1: A,B,C,D after 90° rotation
            [(0, 1), (1, 1), (1, 2), (2, 2)], // State 2: A,B,C,D after 180° rotation  
            [(1, 0), (1, 1), (0, 1), (0, 2)], // State 3: A,B,C,D after 270° rotation
        ],
    ];

    /// 共通のSHAPES配列にアクセスするためのパブリック関数
    /// CLI・Web版両方で同じ座標データを使用するために提供
    pub fn get_shape_coordinates(shape: usize, rotation: usize) -> Option<[(i8, i8); 4]> {
        if shape < Self::SHAPES.len() && rotation < Self::SHAPES[shape].len() {
            Some(Self::SHAPES[shape][rotation])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_new_random_uses_7_bag_system() {
        use crate::random::{RandomProviderImpl, DeterministicRandomProvider};

        let deterministic_provider = DeterministicRandomProvider::new(vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7]);
        let mut test_bag = TetrominoBag {
            bag: TetrominoShape::all_shapes(),
            random_provider: RandomProviderImpl::Deterministic(deterministic_provider),
        };
        test_bag.random_provider.shuffle(&mut test_bag.bag);

        let mut generated_shapes = Vec::new();
        for _ in 0..14 {
            // Generate enough pieces for two full bags
            generated_shapes.push(test_bag.next());
        }

        // Check the first bag
        let first_bag_shapes: HashSet<_> = generated_shapes[0..7].iter().collect();
        assert_eq!(
            first_bag_shapes.len(),
            7,
            "First bag should contain 7 unique shapes"
        );

        // Check the second bag
        let second_bag_shapes: HashSet<_> = generated_shapes[7..14].iter().collect();
        assert_eq!(
            second_bag_shapes.len(),
            7,
            "Second bag should contain 7 unique shapes"
        );
    }

    #[test]
    fn test_new_tetromino_uses_only_three_colors() {
        let tetromino = Tetromino::new_random();
        let allowed_colors = [GameColor::Cyan, GameColor::Magenta, GameColor::Yellow];

        for (_, color) in tetromino.iter_blocks() {
            assert!(
                allowed_colors.contains(&color),
                "Tetromino contains a disallowed color: {:?}",
                color
            );
        }
    }

    #[test]
    fn test_adjacent_blocks_have_different_colors() {
        // ループを複数回実行して、ランダム性の問題を検出する確率を上げる
        for _ in 0..100 {
            let tetromino = Tetromino::new_random();
            let blocks = &tetromino.blocks;

            // すべてのブロックのペアをチェック
            for i in 0..blocks.len() {
                for j in (i + 1)..blocks.len() {
                    let (pos1, color1) = blocks[i];
                    let (pos2, color2) = blocks[j];

                    // 隣接しているかどうかを判断
                    let is_adjacent = (pos1.0 - pos2.0).abs() + (pos1.1 - pos2.1).abs() == 1;

                    if is_adjacent {
                        assert_ne!(
                            color1, color2,
                            "Adjacent blocks have the same color in tetromino: {:?}",
                            tetromino
                        );
                    }
                }
            }
        }
    }
}
