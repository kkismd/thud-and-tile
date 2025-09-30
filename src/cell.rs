use crossterm::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Occupied(Color),
    Solid,
    Connected { color: Color, count: u8 },
}

pub type Board = Vec<Vec<Cell>>;
