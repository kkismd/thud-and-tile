use crate::cell::Cell;
use crate::cli::cli_game_state::CliGameState;
use crate::config::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::game_color::GameColor;

use super::{CliRenderSettings, Frame};

pub struct FrameBuilder;

impl FrameBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn build_full_frame(
        &self,
        cli_state: &CliGameState,
        settings: &CliRenderSettings,
    ) -> Frame {
        let mut lines = Vec::new();

        lines.push(self.build_top_border());
        for y in 0..BOARD_HEIGHT {
            lines.push(self.build_board_line(cli_state, y, settings));
        }
        lines.push(self.build_bottom_border());

        // スコア表示などのUIは旧CLI版表示を完全踏襲する
        lines.push(String::new());
        lines.push(format!("TOTAL SCORE: {}", cli_state.core.score));
        lines.push(format!("CHAIN-BONUS: {}", cli_state.core.chain_bonus));
        lines.push(String::new());
        lines.push("MAX-CHAIN:".to_string());
        lines.push(format!("  CYAN:    {}", cli_state.core.max_chains.cyan));
        lines.push(format!("  MAGENTA: {}", cli_state.core.max_chains.magenta));
        lines.push(format!("  YELLOW:  {}", cli_state.core.max_chains.yellow));
        lines.push(String::new());
        lines.push(format!("Lines: {}", cli_state.core.lines_cleared));
        lines.push(format!("Mode: {:?}", cli_state.core.game_mode));

        if settings.show_fps {
            lines.push(format!("FPS: {:.1}", cli_state.last_fps));
        }

        Frame::new(lines)
    }

    fn build_top_border(&self) -> String {
        let mut top_border = "┌".to_string();
        for _ in 0..BOARD_WIDTH {
            top_border.push_str("──");
        }
        top_border.push('┐');
        top_border
    }

    fn build_bottom_border(&self) -> String {
        let mut bottom_border = "└".to_string();
        for _ in 0..BOARD_WIDTH {
            bottom_border.push_str("──");
        }
        bottom_border.push('┘');
        bottom_border
    }

    fn build_board_line(
        &self,
        cli_state: &CliGameState,
        y: usize,
        settings: &CliRenderSettings,
    ) -> String {
        let mut line = "│".to_string();

        for x in 0..BOARD_WIDTH {
            let cell_display = self.display_for_position(cli_state, x, y, settings);
            line.push_str(&cell_display);
        }

        line.push('│');
        line
    }

    fn display_for_position(
        &self,
        cli_state: &CliGameState,
        x: usize,
        y: usize,
        settings: &CliRenderSettings,
    ) -> String {
        if let Some(ref current_piece) = cli_state.core.current_piece {
            for ((piece_x, piece_y), piece_color) in current_piece.iter_blocks() {
                if piece_x >= 0 && piece_y >= 0 && piece_x as usize == x && piece_y as usize == y {
                    return self.color_to_display(&piece_color, settings.use_colors);
                }
            }
        }

        let board_cell = cli_state.core.board[y][x];
        self.cell_to_display(board_cell, settings.use_colors)
    }

    fn color_to_display(&self, color: &GameColor, use_colors: bool) -> String {
        if !use_colors {
            "██".to_string()
        } else {
            self.color_to_ansi(color).to_string()
        }
    }

    fn cell_to_display(&self, cell: Cell, use_colors: bool) -> String {
        match cell {
            Cell::Empty => "  ".to_string(),
            Cell::Occupied(color) => self.color_to_display(&color, use_colors),
            Cell::Solid => {
                if use_colors {
                    "\x1B[47m  \x1B[0m".to_string()
                } else {
                    "▓▓".to_string()
                }
            }
            Cell::Connected { color, .. } => self.color_to_display(&color, use_colors),
        }
    }

    fn color_to_ansi(&self, color: &GameColor) -> &'static str {
        match color {
            GameColor::Red => "\x1B[41m  \x1B[0m",
            GameColor::Green => "\x1B[42m  \x1B[0m",
            GameColor::Blue => "\x1B[44m  \x1B[0m",
            GameColor::Yellow => "\x1B[43m  \x1B[0m",
            GameColor::Cyan => "\x1B[46m  \x1B[0m",
            GameColor::Magenta => "\x1B[45m  \x1B[0m",
            GameColor::Grey => "\x1B[47m  \x1B[0m",
            _ => "\x1B[40m  \x1B[0m",
        }
    }
}
