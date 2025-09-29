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
}

impl Tetromino {
    pub fn new_random() -> Self {
        let shape = TETROMINO_BAG.lock().unwrap().next();

        let mut colors = COLOR_PALETTE;
        let mut rng = rand::thread_rng(); // Use ThreadRng for color shuffling
        colors.shuffle(&mut rng);

        Self::from_shape(shape, colors)
    }

    pub fn from_shape(shape: TetrominoShape, colors: [Color; 4]) -> Self {
        let matrix = match shape {
            TetrominoShape::I => &SHAPES[0],
            TetrominoShape::O => &SHAPES[1],
            TetrominoShape::T => &SHAPES[2],
            TetrominoShape::L => &SHAPES[3],
            TetrominoShape::J => &SHAPES[4],
            TetrominoShape::S => &SHAPES[5],
            _ => &SHAPES[6],
        };
        let mut blocks = Vec::new();
        for (i, &(block_x, block_y)) in matrix[0].iter().enumerate() {
            blocks.push(((block_x, block_y), colors[i]));
        }

        Tetromino {
            _shape: shape,
            pos: ((BOARD_WIDTH as i8) / 2 - 2, 0),
            blocks,
        }
    }

    pub fn iter_blocks(&self) -> impl Iterator<Item = ((i8, i8), Color)> + '_ {
        self.blocks.iter().map(move |&((block_x, block_y), color)| {
            let pos = (self.pos.0 + block_x, self.pos.1 + block_y);
            (pos, color)
        })
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
            return new_piece;
        }
        new_piece.blocks = self
            .blocks
            .iter()
            .map(|&((x, y), color)| ((-y, x), color))
            .collect();
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
            return new_piece;
        }
        new_piece.blocks = self
            .blocks
            .iter()
            .map(|&((x, y), color)| ((y, -x), color))
            .collect();
        new_piece
    }
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
}
