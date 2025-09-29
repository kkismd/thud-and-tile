use crossterm::style::Color;
use rand::{self, Rng, seq::SliceRandom};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tetromino {
    _shape: TetrominoShape,
    pub pos: (i8, i8),
    blocks: Vec<((i8, i8), Color)>,
}

impl Tetromino {
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        let shape = match rng.gen_range(0..7) {
            0 => TetrominoShape::I,
            1 => TetrominoShape::O,
            2 => TetrominoShape::T,
            3 => TetrominoShape::L,
            4 => TetrominoShape::J,
            5 => TetrominoShape::S,
            _ => TetrominoShape::Z,
        };

        let mut colors = COLOR_PALETTE;
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
