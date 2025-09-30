use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
use std::io::{self, Write};

use crate::cell::Cell;
use crate::config::{BOARD_HEIGHT, BOARD_WIDTH};
use std::time::Duration;

use crate::GameState; // Import GameState from crate root

#[derive(Clone, Debug, PartialEq)]
pub enum Animation {
    LineBlink {
        lines: Vec<usize>,
        count: usize,
        start_time: Duration,
    },
    PushDown {
        gray_line_y: usize,
        start_time: Duration,
    },
}

pub trait Renderer {
    fn clear_screen(&mut self) -> io::Result<()>;
    fn move_to(&mut self, x: u16, y: u16) -> io::Result<()>;
    fn set_foreground_color(&mut self, color: Color) -> io::Result<()>;
    fn print(&mut self, s: &str) -> io::Result<()>;
    fn reset_color(&mut self) -> io::Result<()>;
    fn flush(&mut self) -> io::Result<()>;
}

pub struct CrosstermRenderer {
    pub stdout: io::Stdout,
}

impl CrosstermRenderer {
    pub fn new() -> Self {
        Self {
            stdout: io::stdout(),
        }
    }
}

impl Renderer for CrosstermRenderer {
    fn clear_screen(&mut self) -> io::Result<()> {
        execute!(self.stdout, terminal::Clear(terminal::ClearType::All))
    }

    fn move_to(&mut self, x: u16, y: u16) -> io::Result<()> {
        execute!(self.stdout, MoveTo(x, y))
    }

    fn set_foreground_color(&mut self, color: Color) -> io::Result<()> {
        execute!(self.stdout, SetForegroundColor(color))
    }

    fn print(&mut self, s: &str) -> io::Result<()> {
        execute!(self.stdout, Print(s))
    }

    fn reset_color(&mut self) -> io::Result<()> {
        execute!(self.stdout, ResetColor)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }
}

#[cfg(test)]
pub mod mock_renderer {
    use super::Renderer;
    use crossterm::style::Color;
    use std::cell::RefCell;
    use std::io;
    use std::rc::Rc;

    #[derive(Debug, PartialEq)]
    pub enum RenderCommand {
        ClearScreen,
        MoveTo(u16, u16),
        SetForegroundColor(Color),
        Print(String),
        ResetColor,
        Flush,
    }

    pub struct MockRenderer {
        pub commands: Rc<RefCell<Vec<RenderCommand>>>,
    }

    impl MockRenderer {
        pub fn new() -> Self {
            Self {
                commands: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }

    impl Renderer for MockRenderer {
        fn clear_screen(&mut self) -> io::Result<()> {
            self.commands.borrow_mut().push(RenderCommand::ClearScreen);
            Ok(())
        }

        fn move_to(&mut self, x: u16, y: u16) -> io::Result<()> {
            self.commands.borrow_mut().push(RenderCommand::MoveTo(x, y));
            Ok(())
        }

        fn set_foreground_color(&mut self, color: Color) -> io::Result<()> {
            self.commands
                .borrow_mut()
                .push(RenderCommand::SetForegroundColor(color));
            Ok(())
        }

        fn print(&mut self, s: &str) -> io::Result<()> {
            self.commands
                .borrow_mut()
                .push(RenderCommand::Print(s.to_string()));
            Ok(())
        }

        fn reset_color(&mut self) -> io::Result<()> {
            self.commands.borrow_mut().push(RenderCommand::ResetColor);
            Ok(())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.commands.borrow_mut().push(RenderCommand::Flush);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock_renderer::RenderCommand;
    use super::*;
    use crate::GameState;
    use crate::cell::Cell;
    use crate::config::{BOARD_HEIGHT, BOARD_WIDTH};
    use crossterm::style::Color;
    use std::time::Duration;

    #[test]
    fn test_connected_blocks_blink_during_line_clear_animation() {
        let mut mock_renderer = mock_renderer::MockRenderer::new();
        let mut state = GameState::new();
        state.mode = crate::GameMode::Playing;

        // Setup a board with a line to clear and connected blocks
        let clear_line_y = BOARD_HEIGHT - 2;
        let connected_block_x = 2;
        let connected_block_y = clear_line_y;

        // Create a full line to be cleared
        for x in 0..BOARD_WIDTH {
            state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
        }
        // Place a connected block on the line to be cleared
        state.board[connected_block_y][connected_block_x] = Cell::Connected(Color::Green);

        // Trigger a line blink animation
        state.animation.push(Animation::LineBlink {
            lines: vec![clear_line_y],
            count: 1, // "Off" state for blinking
            start_time: Duration::new(0, 0),
        });

        // Create a prev_state where the animation was just starting (count 0)
        let mut prev_state = state.clone();
        if let Some(Animation::LineBlink { count, .. }) = prev_state.animation.get_mut(0) {
            *count = 0;
        }

        // Draw the state
        draw(&mut mock_renderer, &prev_state, &state).unwrap();

        // Assert that the connected block was drawn as "  " (off state)
        let expected_x = (connected_block_x as u16 * 2) + 1;
        let expected_y = connected_block_y as u16 + 1;

        let commands = mock_renderer.commands.borrow();
        let mut found_move_to = false;
        let mut found_print_empty = false;

        for command in commands.iter() {
            if let RenderCommand::MoveTo(x, y) = command {
                if *x == expected_x && *y == expected_y {
                    found_move_to = true;
                }
            }
            if found_move_to && matches!(command, RenderCommand::Print(s) if s == "  ") {
                found_print_empty = true;
                break;
            }
        }

        assert!(
            found_move_to,
            "MoveTo command for connected block not found"
        );
        assert!(
            found_print_empty,
            "Connected block was not drawn as empty during blink off state"
        );

        // Now test the "On" state (count 0)
        let mut state_on = state.clone();
        if let Some(Animation::LineBlink { count, .. }) = state_on.animation.get_mut(0) {
            *count = 0; // "On" state
        }
        let prev_state_on = state.clone(); // Previous state was "Off" (count 1)

        let mut mock_renderer_on = mock_renderer::MockRenderer::new();
        draw(&mut mock_renderer_on, &prev_state_on, &state_on).unwrap();

        let commands_on = mock_renderer_on.commands.borrow();
        let mut found_move_to_on = false;
        let mut found_print_connected = false;

        for command in commands_on.iter() {
            if let RenderCommand::MoveTo(x, y) = command {
                if *x == expected_x && *y == expected_y {
                    found_move_to_on = true;
                }
            }
            if found_move_to_on
                && matches!(command, RenderCommand::SetForegroundColor(Color::Green))
            {
                // Check for SetForegroundColor(Color::Green) followed by Print("##")
                let mut iter = commands_on.iter().skip_while(|&c| c != command).skip(1);
                if let Some(RenderCommand::Print(s)) = iter.next() {
                    if s == "##" {
                        found_print_connected = true;
                        break;
                    }
                }
            }
        }
        assert!(
            found_move_to_on,
            "MoveTo command for connected block (on state) not found"
        );
        assert!(
            found_print_connected,
            "Connected block was not drawn as '##' during blink on state"
        );
    }
}

pub fn draw_title_screen<R: Renderer>(renderer: &mut R) -> io::Result<()> {
    renderer.clear_screen()?;
    let title = "TETRIPUSH";
    let start_msg = "Press Enter to Start";
    let quit_msg = "Press 'q' to Quit";

    let title_x = (BOARD_WIDTH * 2 + 3 - title.len()) as u16 / 2;
    let title_y = (BOARD_HEIGHT / 2) as u16 - 2;

    let start_x = (BOARD_WIDTH * 2 + 3 - start_msg.len()) as u16 / 2;
    let start_y = (BOARD_HEIGHT / 2) as u16;

    let quit_x = (BOARD_WIDTH * 2 + 3 - quit_msg.len()) as u16 / 2;
    let quit_y = (BOARD_HEIGHT / 2) as u16 + 1;

    renderer.set_foreground_color(Color::Yellow)?;
    renderer.move_to(title_x, title_y)?;
    renderer.print(title)?;
    renderer.set_foreground_color(Color::White)?;
    renderer.move_to(start_x, start_y)?;
    renderer.print(start_msg)?;
    renderer.move_to(quit_x, quit_y)?;
    renderer.print(quit_msg)?;
    renderer.reset_color()?;
    renderer.flush()
}

pub fn draw<R: Renderer>(
    renderer: &mut R,
    prev_state: &GameState,
    state: &GameState,
) -> io::Result<()> {
    if prev_state == state {
        return Ok(());
    }

    match state.mode {
        crate::GameMode::Title => { /* Do nothing, handled by draw_title_screen */ }
        crate::GameMode::Playing => {
            if prev_state.mode != crate::GameMode::Playing {
                // Redraw the whole game screen
                renderer.clear_screen()?;
                // Redraw static elements
                renderer.set_foreground_color(Color::Grey)?;
                renderer.move_to(0, 0)?;
                renderer.print("┌")?;
                renderer.move_to((BOARD_WIDTH * 2) as u16 + 1, 0)?;
                renderer.print("┐")?;
                renderer.move_to(0, BOARD_HEIGHT as u16 + 1)?;
                renderer.print("└")?;
                renderer.move_to((BOARD_WIDTH * 2) as u16 + 1, BOARD_HEIGHT as u16 + 1)?;
                renderer.print("┘")?;
                for y in 1..=BOARD_HEIGHT {
                    renderer.move_to(0, y as u16)?;
                    renderer.print("│")?;
                    renderer.move_to((BOARD_WIDTH * 2) as u16 + 1, y as u16)?;
                    renderer.print("│")?;
                }
                for x in 0..BOARD_WIDTH {
                    renderer.move_to((x * 2) as u16 + 1, 0)?;
                    renderer.print("──")?;
                    renderer.move_to((x * 2) as u16 + 1, BOARD_HEIGHT as u16 + 1)?;
                    renderer.print("──")?;
                }
                renderer.reset_color()?;
                let ui_x = (BOARD_WIDTH * 2 + 4) as u16;
                renderer.set_foreground_color(Color::White)?;
                renderer.move_to(ui_x, 2)?;
                renderer.print("Score: 0     ")?;
                renderer.move_to(ui_x, 3)?;
                renderer.print("Lines: 0     ")?;
                renderer.move_to(ui_x, 5)?;
                renderer.print("Controls:")?;
                renderer.move_to(ui_x, 6)?;
                renderer.print("←/→: Move")?;
                renderer.move_to(ui_x, 7)?;
                renderer.print("↓: Rotate Clockwise")?;
                renderer.move_to(ui_x, 8)?;
                renderer.print("↑: Rotate Counter-Clockwise")?;
                renderer.move_to(ui_x, 9)?;
                renderer.print("Space: Soft Drop")?;
                renderer.move_to(ui_x, 10)?;
                renderer.print("Shift + ↓: Hard Drop")?;
                renderer.move_to(ui_x, 11)?;
                renderer.print("q: Quit")?;
            }

            // --- 消去フェーズ ---
            if let Some(ghost) = &prev_state.ghost_piece()
                && Some(ghost) != prev_state.current_piece.as_ref()
            {
                for ((x, y), _) in ghost.iter_blocks() {
                    if y >= 0 {
                        renderer.move_to((x as u16 * 2) + 1, y as u16 + 1)?;
                        renderer.print("  ")?;
                    }
                }
            }
            if let Some(piece) = &prev_state.current_piece
                && prev_state.animation.is_empty()
            {
                for ((x, y), _) in piece.iter_blocks() {
                    if y >= 0 {
                        renderer.move_to((x as u16 * 2) + 1, y as u16 + 1)?;
                        renderer.print("  ")?;
                    }
                }
            }

            // --- 描画フェーズ ---
            let blink_state = if let Some(Animation::LineBlink { lines, count, .. }) = state
                .animation
                .iter()
                .find(|a| matches!(a, Animation::LineBlink { .. }))
            {
                Some((lines, *count))
            } else {
                None
            };

            for (y, row) in state.board.iter().enumerate() {
                // Handle blinking lines
                if let Some((blinking_lines, count)) = blink_state
                    && blinking_lines.contains(&y)
                {
                    let prev_anim_count = if let Some(Animation::LineBlink { count, .. }) =
                        prev_state
                            .animation
                            .iter()
                            .find(|a| matches!(a, Animation::LineBlink { .. }))
                    {
                        Some(count)
                    } else {
                        None
                    };

                    // Redraw if the blink on/off state has changed, or if animation just started.
                    if prev_anim_count.is_none() || (prev_anim_count.unwrap_or(&0) % 2 != count % 2)
                    {
                        for x in 0..BOARD_WIDTH {
                            renderer.move_to((x as u16 * 2) + 1, y as u16 + 1)?;
                            if count % 2 == 0 {
                                // "On" state
                                if let Cell::Occupied(color) = state.board[y][x] {
                                    renderer.set_foreground_color(color)?;
                                    renderer.print("[]")?;
                                    renderer.reset_color()?;
                                } else if let Cell::Connected { color, count: _ } = state.board[y][x] {
                                    renderer.set_foreground_color(color)?;
                                    renderer.print("##")?;
                                    renderer.reset_color()?;
                                } else {
                                    renderer.print("  ")?;
                                }
                            } else {
                                // "Off" state
                                renderer.print("  ")?;
                            }
                        }
                    }
                    continue; // Done with this row
                }

                // Default drawing for non-blinking lines
                for (x, &cell) in row.iter().enumerate() {
                    let pos = (x as i8, y as i8);
                    let was_ghost = prev_state
                        .ghost_piece()
                        .as_ref()
                        .is_some_and(|g| g.iter_blocks().any(|(p, _)| p == pos));
                    let was_piece = prev_state
                        .current_piece
                        .as_ref()
                        .is_some_and(|p| p.iter_blocks().any(|(p, _)| p == pos));

                    if cell != prev_state.board[y][x]
                        || ((was_ghost || was_piece) && cell != Cell::Empty)
                    {
                        renderer.move_to((x as u16 * 2) + 1, y as u16 + 1)?;
                        match cell {
                            Cell::Empty => renderer.print("  ")?,
                            Cell::Occupied(color) => {
                                renderer.set_foreground_color(color)?;
                                renderer.print("[]")?;
                                renderer.reset_color()?;
                            }
                            Cell::Solid => {
                                renderer.set_foreground_color(Color::Grey)?;
                                renderer.print("[]")?;
                                renderer.reset_color()?;
                            }
                            Cell::Connected { color, count: _ } => {
                                renderer.set_foreground_color(color)?;
                                renderer.print("##")?;
                                renderer.reset_color()?;
                            }
                        }
                    }
                }
            }

            if let Some(ghost) = &state.ghost_piece()
                && Some(ghost) != state.current_piece.as_ref()
            {
                for ((x, y), color) in ghost.iter_blocks() {
                    // color を取得
                    if y >= 0 && state.board[y as usize][x as usize] == Cell::Empty {
                        renderer.move_to((x as u16 * 2) + 1, y as u16 + 1)?;
                        renderer.set_foreground_color(color)?;
                        renderer.print("::")?;
                    }
                }
            }

            if let Some(piece) = &state.current_piece {
                for ((x, y), color) in piece.iter_blocks() {
                    if y >= 0 {
                        renderer.move_to((x as u16 * 2) + 1, y as u16 + 1)?;
                        renderer.set_foreground_color(color)?;
                        renderer.print("[]")?;
                        renderer.reset_color()?;
                    }
                }
            }

            let ui_x = (BOARD_WIDTH * 2 + 4) as u16;
            if prev_state.score != state.score {
                renderer.set_foreground_color(Color::White)?;
                renderer.move_to(ui_x, 2)?;
                renderer.print(format!("Score: {:<6}", state.score).as_str())?;
            }
            if prev_state.lines_cleared != state.lines_cleared {
                renderer.move_to(ui_x, 3)?;
                renderer.print(format!("Lines: {:<6}", state.lines_cleared).as_str())?;
            }

            // NEXTミノの描画
            // 以前のNEXTミノをクリア
            if let Some(prev_next_piece) = &prev_state.next_piece
                && prev_state.next_piece != state.next_piece
            {
                // NEXTミノが変更された場合
                let next_piece_offset_x = ui_x;
                let next_piece_offset_y = 15;
                for ((x, y), _) in prev_next_piece.iter_blocks() {
                    let draw_x = next_piece_offset_x + (x as u16 * 2);
                    let draw_y = next_piece_offset_y + y as u16;
                    renderer.move_to(draw_x, draw_y)?;
                    renderer.print("  ")?;
                }
            }

            if let Some(next_piece) = &state.next_piece {
                renderer.set_foreground_color(Color::White)?;
                renderer.move_to(ui_x, 13)?;
                renderer.print("NEXT:")?; // "NEXT:" ラベル
                let next_piece_offset_x = ui_x;
                let next_piece_offset_y = 15; // "NEXT:" の下あたりに描画

                for ((x, y), color) in next_piece.iter_blocks() {
                    // ミノの座標を調整してUI領域に描画
                    let draw_x = next_piece_offset_x + (x as u16 * 2);
                    let draw_y = next_piece_offset_y + y as u16;
                    renderer.move_to(draw_x, draw_y)?;
                    renderer.set_foreground_color(color)?;
                    renderer.print("[]")?;
                    renderer.reset_color()?;
                }
            }
        }
        crate::GameMode::GameOver => {
            if prev_state.mode != crate::GameMode::GameOver {
                let msg = "GAME OVER";
                let x = (BOARD_WIDTH * 2 + 3 - msg.len()) as u16 / 2;
                let y = (BOARD_HEIGHT / 2) as u16;
                renderer.set_foreground_color(Color::Red)?;
                renderer.move_to(x, y)?;
                renderer.print(msg)?;
            }
        }
    }

    renderer.flush()
}
