use crate::game_color::GameColor;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Occupied(GameColor),
    Solid,
    Connected { color: GameColor, count: u8 },
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cell::Empty => write!(f, "Empty"),
            Cell::Occupied(color) => write!(f, "Occupied({:?})", color),
            Cell::Solid => write!(f, "Solid"),
            Cell::Connected { color, count } => write!(f, "Connected({:?}, {})", color, count),
        }
    }
}

pub type Board = Vec<Vec<Cell>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_color::GameColor;

    #[test]
    fn test_connected_cell_debug_output() {
        let cell = Cell::Connected {
            color: GameColor::Red,
            count: 5,
        };
        // This test will fail initially because the default Debug output is different.
        // It will pass once a custom Debug impl is added.
        assert_eq!(format!("{:?}", cell), "Connected(Red, 5)");
    }
}
