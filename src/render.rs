use crossterm::{
    cursor::{MoveTo},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
use std::io::{self, Write};

use crate::config::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::cell::{Board, Cell};
use crate::tetromino::Tetromino;
use std::time::Duration;

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

// GameState is defined in main.rs, so we need to import it
// For now, we'll assume GameState will be passed as a parameter or we'll need to adjust
// the structure later. For now, let's define a trait or pass necessary parts.
// Or, more simply, we can make GameState public and import it.
// Let's make GameState public in main.rs and import it here.
// For now, I'll add a placeholder for GameState and will adjust main.rs later.
// Assuming GameState will be imported from main.rs
use crate::GameState; // This will require GameState to be public in main.rs

pub fn draw_title_screen(stdout: &mut io::Stdout) -> io::Result<()> {
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    let title = "TETRIPUSH";
    let start_msg = "Press Enter to Start";
    let quit_msg = "Press 'q' to Quit";

    let title_x = (BOARD_WIDTH * 2 + 3 - title.len()) as u16 / 2;
    let title_y = (BOARD_HEIGHT / 2) as u16 - 2;

    let start_x = (BOARD_WIDTH * 2 + 3 - start_msg.len()) as u16 / 2;
    let start_y = (BOARD_HEIGHT / 2) as u16;

    let quit_x = (BOARD_WIDTH * 2 + 3 - quit_msg.len()) as u16 / 2;
    let quit_y = (BOARD_HEIGHT / 2) as u16 + 1;

    execute!(
        stdout,
        SetForegroundColor(Color::Yellow),
        MoveTo(title_x, title_y),
        Print(title),
        SetForegroundColor(Color::White),
        MoveTo(start_x, start_y),
        Print(start_msg),
        MoveTo(quit_x, quit_y),
        Print(quit_msg),
        ResetColor
    )?;
    stdout.flush()
}

pub fn draw<W: io::Write>(stdout: &mut W, prev_state: &GameState, state: &GameState) -> io::Result<()> {
    if prev_state == state {
        return Ok(());
    }

    match state.mode {
        crate::GameMode::Title => { /* Do nothing, handled by draw_title_screen */ }
        crate::GameMode::Playing => {
            if prev_state.mode != crate::GameMode::Playing {
                // Redraw the whole game screen
                execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
                // Redraw static elements
                execute!(stdout, SetForegroundColor(Color::Grey))?;
                execute!(stdout, MoveTo(0, 0), Print("┌"))?;
                execute!(stdout, MoveTo((BOARD_WIDTH * 2) as u16 + 1, 0), Print("┐"))?;
                execute!(stdout, MoveTo(0, BOARD_HEIGHT as u16 + 1), Print("└"))?;
                execute!(
                    stdout,
                    MoveTo((BOARD_WIDTH * 2) as u16 + 1, BOARD_HEIGHT as u16 + 1),
                    Print("┘")
                )?;
                for y in 1..=BOARD_HEIGHT {
                    execute!(stdout, MoveTo(0, y as u16), Print("│"))?;
                    execute!(
                        stdout,
                        MoveTo((BOARD_WIDTH * 2) as u16 + 1, y as u16),
                        Print("│")
                    )?;
                }
                for x in 0..BOARD_WIDTH {
                    execute!(stdout, MoveTo((x * 2) as u16 + 1, 0), Print("──"))?;
                    execute!(
                        stdout,
                        MoveTo((x * 2) as u16 + 1, BOARD_HEIGHT as u16 + 1),
                        Print("──")
                    )?;
                }
                execute!(stdout, ResetColor)?;
                let ui_x = (BOARD_WIDTH * 2 + 4) as u16;
                execute!(
                    stdout,
                    SetForegroundColor(Color::White),
                    MoveTo(ui_x, 2),
                    Print("Score: 0     ")
                )?;
                execute!(stdout, MoveTo(ui_x, 3), Print("Lines: 0     "))?;
                execute!(stdout, MoveTo(ui_x, 5), Print("Controls:"))?;
                execute!(stdout, MoveTo(ui_x, 6), Print("←/→: Move"))?;
                execute!(stdout, MoveTo(ui_x, 7), Print("↓: Rotate Clockwise"))?;
                execute!(
                    stdout,
                    MoveTo(ui_x, 8),
                    Print("↑: Rotate Counter-Clockwise")
                )?;
                execute!(stdout, MoveTo(ui_x, 9), Print("Space: Soft Drop"))?;
                execute!(stdout, MoveTo(ui_x, 10), Print("Shift + ↓: Hard Drop"))?;
                execute!(stdout, MoveTo(ui_x, 11), Print("q: Quit"))?;
            }

            // --- 消去フェーズ ---
            if let Some(ghost) = &prev_state.ghost_piece()
                && Some(ghost) != prev_state.current_piece.as_ref()
            {
                for ((x, y), _) in ghost.iter_blocks() {
                    if y >= 0 {
                        execute!(
                            stdout,
                            MoveTo((x as u16 * 2) + 1, y as u16 + 1),
                            Print("  ")
                        )?;
                    }
                }
            }
            if let Some(piece) = &prev_state.current_piece
                && prev_state.animation.is_empty()
            {
                for ((x, y), _) in piece.iter_blocks() {
                    if y >= 0 {
                        execute!(
                            stdout,
                            MoveTo((x as u16 * 2) + 1, y as u16 + 1),
                            Print("  ")
                        )?;
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
                            execute!(stdout, MoveTo((x as u16 * 2) + 1, y as u16 + 1))?;
                            if count % 2 == 0 {
                                // "On" state
                                if let Cell::Occupied(color) = state.board[y][x] {
                                    execute!(
                                        stdout,
                                        SetForegroundColor(color),
                                        Print("[]"),
                                        ResetColor
                                    )?;
                                } else {
                                    execute!(stdout, Print("  "))?;
                                }
                            } else {
                                // "Off" state
                                execute!(stdout, Print("  "))?;
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
                        execute!(stdout, MoveTo((x as u16 * 2) + 1, y as u16 + 1))?;
                        match cell {
                            Cell::Empty => execute!(stdout, Print("  "))?,
                            Cell::Occupied(color) => execute!(
                                stdout,
                                SetForegroundColor(color),
                                Print("[]"),
                                ResetColor
                            )?,
                            Cell::Solid => execute!(
                                stdout,
                                SetForegroundColor(Color::Grey),
                                Print("[]"),
                                ResetColor
                            )?,
                            Cell::Connected(color) => execute!(
                                stdout,
                                SetForegroundColor(color),
                                Print("##"),
                                ResetColor
                            )?,
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
                        execute!(
                            stdout,
                            MoveTo((x as u16 * 2) + 1, y as u16 + 1),
                            SetForegroundColor(color), // color を使用
                            Print("::")
                        )?;
                    }
                }
            }

            if let Some(piece) = &state.current_piece {
                for ((x, y), color) in piece.iter_blocks() {
                    if y >= 0 {
                        execute!(
                            stdout,
                            MoveTo((x as u16 * 2) + 1, y as u16 + 1),
                            SetForegroundColor(color),
                            Print("[]"),
                            ResetColor
                        )?;
                    }
                }
            }

            let ui_x = (BOARD_WIDTH * 2 + 4) as u16;
            if prev_state.score != state.score {
                execute!(
                    stdout,
                    SetForegroundColor(Color::White),
                    MoveTo(ui_x, 2),
                    Print(format!("Score: {:<6}", state.score))
                )?;
            }
            if prev_state.lines_cleared != state.lines_cleared {
                execute!(
                    stdout,
                    MoveTo(ui_x, 3),
                    Print(format!("Lines: {:<6}", state.lines_cleared))
                )?;
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
                    execute!(stdout, MoveTo(draw_x, draw_y), Print("  "))?;
                }
            }

            if let Some(next_piece) = &state.next_piece {
                execute!(stdout, SetForegroundColor(Color::White))?;
                execute!(stdout, MoveTo(ui_x, 13), Print("NEXT:"))?; // "NEXT:" ラベル
                let next_piece_offset_x = ui_x;
                let next_piece_offset_y = 15; // "NEXT:" の下あたりに描画

                for ((x, y), color) in next_piece.iter_blocks() {
                    // ミノの座標を調整してUI領域に描画
                    let draw_x = next_piece_offset_x + (x as u16 * 2);
                    let draw_y = next_piece_offset_y + y as u16;
                    execute!(
                        stdout,
                        MoveTo(draw_x, draw_y),
                        SetForegroundColor(color),
                        Print("[]"),
                        ResetColor
                    )?;
                }
            }
        }
        crate::GameMode::GameOver => {
            if prev_state.mode != crate::GameMode::GameOver {
                let msg = "GAME OVER";
                let x = (BOARD_WIDTH * 2 + 3 - msg.len()) as u16 / 2;
                let y = (BOARD_HEIGHT / 2) as u16;
                execute!(
                    stdout,
                    SetForegroundColor(Color::Red),
                    MoveTo(x, y),
                    Print(msg)
                )?;
            }
        }
    }

    stdout.flush()
}