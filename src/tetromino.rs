use crossterm::style::Color;
use lazy_static::lazy_static;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::{self, seq::SliceRandom};
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
    rng: StdRng,
}

impl TetrominoBag {
    pub fn new() -> Self {
        let mut bag = TetrominoShape::all_shapes();
        let mut rng = StdRng::from_entropy(); // Use from_entropy for production, fixed seed for testing if needed
        bag.shuffle(&mut rng);
        TetrominoBag { bag, rng }
    }

    pub fn next(&mut self) -> TetrominoShape {
        if self.bag.is_empty() {
            self.bag = TetrominoShape::all_shapes();
            self.bag.shuffle(&mut self.rng);
        }
        self.bag.pop().unwrap()
    }
}

lazy_static! {
    static ref TETROMINO_BAG: Mutex<TetrominoBag> = Mutex::new(TetrominoBag::new());
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tetromino {
    _shape: TetrominoShape,
    pub pos: (i8, i8),
    blocks: Vec<((i8, i8), Color)>,
    rotation_state: u8, // SRS rotation state: 0, 1, 2, 3
}

impl Tetromino {
    pub fn new_random() -> Self {
        let shape = TETROMINO_BAG.lock().unwrap().next();

        // Loop until a valid coloring is found
        loop {
            let mut rng = rand::thread_rng();
            let colors = [
                *COLOR_PALETTE.choose(&mut rng).unwrap(),
                *COLOR_PALETTE.choose(&mut rng).unwrap(),
                *COLOR_PALETTE.choose(&mut rng).unwrap(),
                *COLOR_PALETTE.choose(&mut rng).unwrap(),
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

    pub fn from_shape(shape: TetrominoShape, colors: [Color; 4]) -> Self {
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
            _shape: shape,
            pos: ((BOARD_WIDTH as i8) / 2 - 2, 0),
            blocks,
            rotation_state: 0, // Initial rotation state
        }
    }

    pub fn iter_blocks(&self) -> impl Iterator<Item = ((i8, i8), Color)> + '_ {
        self.blocks.iter().map(move |&((block_x, block_y), color)| {
            let pos = (self.pos.0 + block_x, self.pos.1 + block_y);
            (pos, color)
        })
    }

    /// Gets the current rotation state (0, 1, 2, 3).
    /// This method is primarily used for testing SRS compliance.
    pub fn get_rotation_state(&self) -> u8 {
        self.rotation_state
    }

    pub fn moved(&self, dx: i8, dy: i8) -> Self {
        let mut new_piece = self.clone();
        new_piece.pos = (self.pos.0 + dx, self.pos.1 + dy);
        new_piece
    }

    pub fn rotated(&self) -> Self {
        let mut new_piece = self.clone();
        if self._shape == TetrominoShape::O {
            let old_colors: Vec<Color> = self.blocks.iter().map(|&(_, color)| color).collect();
            let mut new_colors: Vec<Color> = vec![Color::Black; 4]; // Initialize with dummy colors

            // Apply the specific clockwise color rotation for O-mino
            new_colors[0] = old_colors[2]; // Top-Left gets Bottom-Left's color
            new_colors[1] = old_colors[0]; // Top-Right gets Top-Left's color
            new_colors[2] = old_colors[3]; // Bottom-Left gets Bottom-Right's color
            new_colors[3] = old_colors[1]; // Bottom-Right gets Top-Right's color

            new_piece.blocks = self
                .blocks
                .iter()
                .enumerate()
                .map(|(i, &((x, y), _))| ((x, y), new_colors[i]))
                .collect();
        } else if self._shape == TetrominoShape::I {
            // I-mino specific rotation around the second block
            let pivot_block_index = 1; // Second block is at index 1
            let (pivot_x, pivot_y) = self.blocks[pivot_block_index].0;

            new_piece.blocks = self
                .blocks
                .iter()
                .map(|&((block_x, block_y), color)| {
                    // Translate block so pivot is at (0,0)
                    let translated_x = block_x - pivot_x;
                    let translated_y = block_y - pivot_y;

                    // Rotate around (0,0)
                    let rotated_x = -translated_y;
                    let rotated_y = translated_x;

                    // Translate back
                    ((rotated_x + pivot_x, rotated_y + pivot_y), color)
                })
                .collect();
        } else {
            new_piece.blocks = self
                .blocks
                .iter()
                .map(|&((x, y), color)| ((-y, x), color))
                .collect();
        }

        // Update rotation state: clockwise (0 -> 1 -> 2 -> 3 -> 0)
        new_piece.rotation_state = (self.rotation_state + 1) % 4;

        new_piece
    }

    pub fn rotated_counter_clockwise(&self) -> Self {
        let mut new_piece = self.clone();
        if self._shape == TetrominoShape::O {
            let old_colors: Vec<Color> = self.blocks.iter().map(|&(_, color)| color).collect();
            let mut new_colors: Vec<Color> = vec![Color::Black; 4]; // Initialize with dummy colors

            // Apply the specific counter-clockwise color rotation for O-mino
            new_colors[0] = old_colors[1]; // Top-Left gets Top-Right's color
            new_colors[1] = old_colors[3]; // Top-Right gets Bottom-Right's color
            new_colors[2] = old_colors[0]; // Bottom-Left gets Top-Left's color
            new_colors[3] = old_colors[2]; // Bottom-Right gets Bottom-Left's color

            new_piece.blocks = self
                .blocks
                .iter()
                .enumerate()
                .map(|(i, &((x, y), _))| ((x, y), new_colors[i]))
                .collect();
        } else if self._shape == TetrominoShape::I {
            // I-mino specific counter-clockwise rotation around the second block
            let pivot_block_index = 1; // Second block is at index 1
            let (pivot_x, pivot_y) = self.blocks[pivot_block_index].0;

            new_piece.blocks = self
                .blocks
                .iter()
                .map(|&((block_x, block_y), color)| {
                    // Translate block so pivot is at (0,0)
                    let translated_x = block_x - pivot_x;
                    let translated_y = block_y - pivot_y;

                    // Rotate around (0,0) (counter-clockwise)
                    let rotated_x = translated_y;
                    let rotated_y = -translated_x;

                    // Translate back
                    ((rotated_x + pivot_x, rotated_y + pivot_y), color)
                })
                .collect();
        } else {
            new_piece.blocks = self
                .blocks
                .iter()
                .map(|&((x, y), color)| ((y, -x), color))
                .collect();
        }

        // Update rotation state: counter-clockwise (0 -> 3 -> 2 -> 1 -> 0)
        new_piece.rotation_state = (self.rotation_state + 3) % 4; // +3 is equivalent to -1 in modulo 4

        new_piece
    }

    const SHAPES: [[[(i8, i8); 4]; 4]; 7] = [
        // I
        [
            [(1, 0), (1, 1), (1, 2), (1, 3)],
            [(0, 2), (1, 2), (2, 2), (3, 2)],
            [(2, 0), (2, 1), (2, 2), (2, 3)],
            [(0, 1), (1, 1), (2, 1), (3, 1)],
        ],
        // O
        [
            [(1, 1), (2, 1), (1, 2), (2, 2)],
            [(1, 1), (2, 1), (1, 2), (2, 2)],
            [(1, 1), (2, 1), (1, 2), (2, 2)],
            [(1, 1), (2, 1), (1, 2), (2, 2)],
        ],
        // T
        [
            [(1, 0), (0, 1), (1, 1), (2, 1)],
            [(1, 0), (1, 1), (2, 1), (1, 2)],
            [(0, 1), (1, 1), (2, 1), (1, 2)],
            [(1, 0), (0, 1), (1, 1), (1, 2)],
        ],
        // L
        [
            [(2, 0), (0, 1), (1, 1), (2, 1)],
            [(1, 0), (1, 1), (1, 2), (2, 2)],
            [(0, 1), (1, 1), (2, 1), (0, 2)],
            [(0, 0), (1, 0), (1, 1), (1, 2)],
        ],
        // J
        [
            [(0, 0), (0, 1), (1, 1), (2, 1)],
            [(1, 0), (2, 0), (1, 1), (1, 2)],
            [(0, 1), (1, 1), (2, 1), (2, 2)],
            [(1, 0), (1, 1), (0, 2), (1, 2)],
        ],
        // S
        [
            [(1, 0), (2, 0), (0, 1), (1, 1)],
            [(1, 0), (1, 1), (2, 1), (2, 2)],
            [(1, 1), (2, 1), (0, 2), (1, 2)],
            [(0, 0), (0, 1), (1, 1), (1, 2)],
        ],
        // Z
        [
            [(0, 0), (1, 0), (1, 1), (2, 1)],
            [(2, 0), (1, 1), (2, 1), (1, 2)],
            [(0, 1), (1, 1), (1, 2), (2, 2)],
            [(1, 0), (0, 1), (1, 1), (0, 2)],
        ],
    ];
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_new_random_uses_7_bag_system() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let rng = StdRng::seed_from_u64(123);
        let mut test_bag = TetrominoBag {
            bag: TetrominoShape::all_shapes(),
            rng: rng,
        };
        test_bag.bag.shuffle(&mut test_bag.rng);

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
        let allowed_colors = [Color::Cyan, Color::Magenta, Color::Yellow];

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
