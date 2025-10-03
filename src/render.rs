use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Print, ResetColor, SetForegroundColor},
    terminal,
};
use std::io::{self, Write};

use crate::cell::Cell;
use crate::config::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::game_color::GameColor;
use std::time::Duration;

use crate::GameState; // Import GameState from main.rs
use crate::GameMode;
use crate::animation::Animation; // 共通Animationを使用

pub trait Renderer {
    fn clear_screen(&mut self) -> io::Result<()>;
    fn move_to(&mut self, x: u16, y: u16) -> io::Result<()>;
    fn set_foreground_color(&mut self, color: GameColor) -> io::Result<()>;
    fn set_background_color(&mut self, color: GameColor) -> io::Result<()>; // Add this line
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

    fn set_foreground_color(&mut self, color: GameColor) -> io::Result<()> {
        execute!(self.stdout, SetForegroundColor(color.into()))
    }

    fn set_background_color(&mut self, color: GameColor) -> io::Result<()> {
        // Add this method
        execute!(self.stdout, crossterm::style::SetBackgroundColor(color.into()))
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
    use crate::game_color::GameColor;
    use std::cell::RefCell;
    use std::io;
    use std::rc::Rc;

    #[derive(Clone, Debug, PartialEq)]
    pub enum RenderCommand {
        ClearScreen,
        MoveTo(u16, u16),
        SetForegroundColor(GameColor),
        SetBackgroundColor(GameColor), // Add this line
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

        fn set_foreground_color(&mut self, color: GameColor) -> io::Result<()> {
            self.commands
                .borrow_mut()
                .push(RenderCommand::SetForegroundColor(color));
            Ok(())
        }

        fn set_background_color(&mut self, color: GameColor) -> io::Result<()> {
            // Add this method
            self.commands
                .borrow_mut()
                .push(RenderCommand::SetBackgroundColor(color));
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
    use crate::game_color::GameColor;
    use std::time::Duration;

    #[test]
    fn test_connected_blocks_blink_during_line_clear_animation() {
        let mut mock_renderer = mock_renderer::MockRenderer::new();
        let mut state = GameState::new();
        state.mode = GameMode::Playing;

        // Setup a board with a line to clear and connected blocks
        let clear_line_y = BOARD_HEIGHT - 2;
        let connected_block_x = 2;
        let connected_block_y = clear_line_y;

        // Create a full line to be cleared
        for x in 0..BOARD_WIDTH {
            state.board[clear_line_y][x] = Cell::Occupied(GameColor::Blue);
        }
        // Place a connected block on the line to be cleared
        state.board[connected_block_y][connected_block_x] = Cell::Connected {
            color: GameColor::Green,
            count: 1,
        };

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
                && matches!(command, RenderCommand::SetBackgroundColor(GameColor::Green))
            {
                // Check for SetBackgroundColor(GameColor::Green) followed by SetForegroundColor(GameColor::Black) and Print(count)
                let mut iter = commands_on.iter().skip_while(|&c| c != command).skip(1);
                if let Some(RenderCommand::SetForegroundColor(fg_color)) = iter.next() {
                    if *fg_color == GameColor::Black {
                        if let Some(RenderCommand::Print(s)) = iter.next() {
                            // The connected block in the test has count 1
                            if s == " 1" {
                                found_print_connected = true;
                                break;
                            }
                        }
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
            "Connected block was not drawn with count during blink on state"
        );
    }

    #[test]
    fn test_render_connected_block_with_count() {
        let mut mock_renderer = mock_renderer::MockRenderer::new();
        let mut state = GameState::new();
        state.mode = GameMode::Playing;

        let test_color = GameColor::Red;
        let test_count = 5;
        let block_x = 3;
        let block_y = 5;

        // Set a connected block with a count
        state.board[block_y][block_x] = Cell::Connected {
            color: test_color,
            count: test_count,
        };

        // Create a prev_state where the block was empty to force a redraw
        let mut prev_state = state.clone();
        prev_state.board[block_y][block_x] = Cell::Empty;

        // Draw the state
        draw(&mut mock_renderer, &prev_state, &state).unwrap();

        let expected_x = (block_x as u16 * 2) + 1;
        let expected_y = block_y as u16 + 1;

        let commands = mock_renderer.commands.borrow();

        // Find the sequence of commands for the specific block
        let expected_commands_sequence = vec![
            RenderCommand::MoveTo(expected_x, expected_y),
            RenderCommand::SetBackgroundColor(test_color),
            RenderCommand::SetForegroundColor(GameColor::Black),
            RenderCommand::Print(format!("{:>2}", test_count)),
            RenderCommand::ResetColor,
        ];

        let mut found_sequence = false;
        for window in commands.windows(expected_commands_sequence.len()) {
            if window.to_vec() == expected_commands_sequence {
                found_sequence = true;
                break;
            }
        }

        assert!(
            found_sequence,
            "Did not find the expected rendering sequence for connected block with count. Commands: {:?}",
            commands
        );
    }
}

pub fn draw_title_screen<R: Renderer>(renderer: &mut R) -> io::Result<()> {
    renderer.clear_screen()?;
    let title = "THUD & TILE";
    let start_msg = "Press Enter to Start";
    let quit_msg = "Press 'q' to Quit";

    let title_x = (BOARD_WIDTH * 2 + 3 - title.len()) as u16 / 2;
    let title_y = (BOARD_HEIGHT / 2) as u16 - 2;

    let start_x = (BOARD_WIDTH * 2 + 3 - start_msg.len()) as u16 / 2;
    let start_y = (BOARD_HEIGHT / 2) as u16;

    let quit_x = (BOARD_WIDTH * 2 + 3 - quit_msg.len()) as u16 / 2;
    let quit_y = (BOARD_HEIGHT / 2) as u16 + 1;

    renderer.set_foreground_color(GameColor::Yellow)?;
    renderer.move_to(title_x, title_y)?;
    renderer.print(title)?;
    renderer.set_foreground_color(GameColor::White)?;
    renderer.move_to(start_x, start_y)?;
    renderer.print(start_msg)?;
    renderer.move_to(quit_x, quit_y)?;
    renderer.print(quit_msg)?;
    renderer.reset_color()?;
    renderer.flush()
}

fn draw_connected_cell<R: Renderer>(
    renderer: &mut R,
    color: GameColor,
    count: u8,
    x: u16,
    y: u16,
) -> io::Result<()> {
    renderer.move_to(x, y)?;
    renderer.set_background_color(color)?;
    renderer.set_foreground_color(GameColor::Black)?;
    renderer.print(&format!("{:>2}", count))?;
    renderer.reset_color()?;
    Ok(())
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
        GameMode::Title => { /* Do nothing, handled by draw_title_screen */ }
        GameMode::Playing => {
            if prev_state.mode != GameMode::Playing {
                // Redraw the whole game screen
                renderer.clear_screen()?;
                // Redraw static elements
                renderer.set_foreground_color(GameColor::Grey)?;
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
                renderer.set_foreground_color(GameColor::White)?;
                renderer.move_to(ui_x, 2)?;
                renderer.print("SCORE:     0     ")?;
                renderer.move_to(ui_x, 3)?;
                renderer.print("  CYAN:    0     ")?;
                renderer.move_to(ui_x, 4)?;
                renderer.print("  MAGENTA: 0     ")?;
                renderer.move_to(ui_x, 5)?;
                renderer.print("  YELLOW:  0     ")?;
                renderer.move_to(ui_x, 7)?;
                renderer.print("MAX-CHAIN: 0     ")?;
                renderer.move_to(ui_x, 8)?;
                renderer.print("  CYAN:    0     ")?;
                renderer.move_to(ui_x, 9)?;
                renderer.print("  MAGENTA: 0     ")?;
                renderer.move_to(ui_x, 10)?;
                renderer.print("  YELLOW:  0     ")?;
            }

            // --- 消去フェーズ ---
            if let Some(ghost) = &prev_state.ghost_piece() {
                if Some(ghost) != prev_state.current_piece.as_ref() {
                    for ((x, y), _) in ghost.iter_blocks() {
                        if y >= 0 {
                            renderer.move_to((x as u16 * 2) + 1, y as u16 + 1)?;
                            renderer.print("  ")?;
                        }
                    }
                }
            }
            if let Some(piece) = &prev_state.current_piece {
                if prev_state.animation.is_empty() {
                    for ((x, y), _) in piece.iter_blocks() {
                        if y >= 0 {
                            renderer.move_to((x as u16 * 2) + 1, y as u16 + 1)?;
                            renderer.print("  ")?;
                        }
                    }
                }
            }

            // --- 描画フェーズ ---
            let blink_state = if let Some(Animation::LineBlink { lines, count, .. }) = state
                .animation
                .iter()
                .find(|a| matches!(a, Animation::LineBlink { .. }))
            {
                Some((lines, count))
            } else {
                None
            };

            for (y, row) in state.board.iter().enumerate() {
                // Handle blinking lines
                if let Some((blinking_lines, count)) = blink_state {
                    if blinking_lines.contains(&y) {
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
                    if prev_anim_count.is_none() || (prev_anim_count.unwrap_or(&0) % 2 != *count % 2)
                    {
                        for x in 0..BOARD_WIDTH {
                            renderer.move_to((x as u16 * 2) + 1, y as u16 + 1)?;
                            if count % 2 == 0 {
                                // "On" state
                                if let Cell::Occupied(color) = state.board[y][x] {
                                    renderer.set_foreground_color(color)?;
                                    renderer.print("[]")?;
                                    renderer.reset_color()?;
                                } else if let Cell::Connected { color, count } = state.board[y][x] {
                                    draw_connected_cell(
                                        renderer,
                                        color,
                                        count,
                                        (x as u16 * 2) + 1,
                                        y as u16 + 1,
                                    )?;
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
                                renderer.set_foreground_color(GameColor::Grey)?;
                                renderer.print("[]")?;
                                renderer.reset_color()?;
                            }
                            Cell::Connected { color, count } => {
                                draw_connected_cell(
                                    renderer,
                                    color,
                                    count,
                                    (x as u16 * 2) + 1,
                                    y as u16 + 1,
                                )?;
                            }
                        }
                    }
                }
            }

            if let Some(ghost) = &state.ghost_piece() {
                if Some(ghost) != state.current_piece.as_ref() {
                    for ((x, y), color) in ghost.iter_blocks() {
                        // color を取得
                        if y >= 0 && state.board[y as usize][x as usize] == Cell::Empty {
                            renderer.move_to((x as u16 * 2) + 1, y as u16 + 1)?;
                            renderer.set_foreground_color(color)?;
                            renderer.print("::")?;
                        }
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

            // Display custom score system instead of simple score/lines
            if prev_state.custom_score_system != state.custom_score_system {
                renderer.set_foreground_color(GameColor::White)?;

                // Display total score
                renderer.move_to(ui_x, 2)?;
                renderer.print(
                    format!("SCORE:     {:<6}", state.custom_score_system.scores.total()).as_str(),
                )?;

                // Display color breakdown
                renderer.move_to(ui_x, 3)?;
                renderer.print(
                    format!("  CYAN:    {:<6}", state.custom_score_system.scores.cyan).as_str(),
                )?;
                renderer.move_to(ui_x, 4)?;
                renderer.print(
                    format!("  MAGENTA: {:<6}", state.custom_score_system.scores.magenta).as_str(),
                )?;
                renderer.move_to(ui_x, 5)?;
                renderer.print(
                    format!("  YELLOW:  {:<6}", state.custom_score_system.scores.yellow).as_str(),
                )?;

                // Display max chain
                renderer.move_to(ui_x, 7)?;
                renderer.print(
                    format!(
                        "MAX-CHAIN: {:<6}",
                        state.custom_score_system.max_chains.max()
                    )
                    .as_str(),
                )?;
                renderer.move_to(ui_x, 8)?;
                renderer.print(
                    format!(
                        "  CYAN:    {:<6}",
                        state.custom_score_system.max_chains.cyan
                    )
                    .as_str(),
                )?;
                renderer.move_to(ui_x, 9)?;
                renderer.print(
                    format!(
                        "  MAGENTA: {:<6}",
                        state.custom_score_system.max_chains.magenta
                    )
                    .as_str(),
                )?;
                renderer.move_to(ui_x, 10)?;
                renderer.print(
                    format!(
                        "  YELLOW:  {:<6}",
                        state.custom_score_system.max_chains.yellow
                    )
                    .as_str(),
                )?;

                renderer.reset_color()?;
            }

            // NEXTミノの描画
            let next_piece_offset_x = ui_x;
            let next_piece_offset_y = 14; // NEXT:ラベルの下

            // 以前のNEXTミノをクリア
            if let Some(prev_next_piece) = &prev_state.next_piece {
                if prev_state.next_piece != state.next_piece {
                    // NEXTミノが変更された場合、以前の位置をクリア
                    for ((x, y), _) in prev_next_piece.iter_blocks() {
                        let draw_x = next_piece_offset_x + (x as u16 * 2);
                        let draw_y = next_piece_offset_y + y as u16;
                        renderer.move_to(draw_x, draw_y)?;
                        renderer.print("  ")?;
                    }
                }
            }

            // 現在のNEXTミノを描画
            if let Some(next_piece) = &state.next_piece {
                renderer.set_foreground_color(GameColor::White)?;
                renderer.move_to(ui_x, 12)?;
                renderer.print("NEXT:")?; // "NEXT:" ラベル

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
        GameMode::GameOver => {
            if prev_state.mode != GameMode::GameOver {
                let msg = "GAME OVER";
                let x = (BOARD_WIDTH * 2 + 3 - msg.len()) as u16 / 2;
                let y = (BOARD_HEIGHT / 2) as u16;
                renderer.set_foreground_color(GameColor::Red)?;
                renderer.move_to(x, y)?;
                renderer.print(msg)?;
            }
        }
    }

    renderer.flush()
}
