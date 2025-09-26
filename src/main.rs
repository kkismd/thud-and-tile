use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        self, Event, KeyCode, KeyEventKind, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::seq::SliceRandom;
use rand::{self, Rng};
use std::collections::VecDeque;
use std::io::{self, Write, stdout};
use std::thread;
use std::time::{Duration, Instant};

// --- 定数 ---
const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;
const FALL_SPEED_START: Duration = Duration::from_millis(800);

const COLOR_PALETTE: [Color; 4] = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];
const BLINK_ANIMATION_STEP: Duration = Duration::from_millis(120);
const BLINK_COUNT_MAX: usize = 6; // 3 blinks: on-off-on-off-on-off
const PUSH_DOWN_STEP_DURATION: Duration = Duration::from_millis(100);

// --- データ構造 ---

type Point = (usize, usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cell {
    Empty,
    Occupied(Color),
    Solid,
}

type Board = Vec<Vec<Cell>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TetrominoShape {
    I,
    O,
    T,
    L,
    J,
    S,
    Z,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameMode {
    Title,
    Playing,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Tetromino {
    _shape: TetrominoShape,
    matrix: &'static [[(i8, i8); 4]; 4],
    pos: (i8, i8),
    colors: [Color; 4],
    rotation: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Animation {
    LineBlink {
        lines: Vec<usize>,
        count: usize,
        start: Instant,
    },
    PushDown {
        gray_line_y: usize,
        start: Instant,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct GameState {
    mode: GameMode,
    board: Board,
    current_piece: Option<Tetromino>,
    animation: Option<Animation>,
    score: u32,
    lines_cleared: u32,
    fall_speed: Duration,
    blocks_to_score: Vec<(Point, u32)>,
}

impl Tetromino {
    fn new_random() -> Self {
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

    fn from_shape(shape: TetrominoShape, colors: [Color; 4]) -> Self {
        let matrix = match shape {
            TetrominoShape::I => &SHAPES[0],
            TetrominoShape::O => &SHAPES[1],
            TetrominoShape::T => &SHAPES[2],
            TetrominoShape::L => &SHAPES[3],
            TetrominoShape::J => &SHAPES[4],
            TetrominoShape::S => &SHAPES[5],
            TetrominoShape::Z => &SHAPES[6],
        };
        Tetromino {
            _shape: shape,
            matrix,
            pos: ((BOARD_WIDTH as i8) / 2 - 2, 0),
            colors,
            rotation: 0,
        }
    }

    fn iter_blocks(&self) -> impl Iterator<Item = ((i8, i8), Color)> + '_ {
        self.matrix[self.rotation as usize]
            .iter()
            .zip(self.colors.iter())
            .map(move |(&pos, &color)| ((self.pos.0 + pos.0, self.pos.1 + pos.1), color))
    }

    fn moved(&self, dx: i8, dy: i8) -> Self {
        let mut new_piece = self.clone();
        new_piece.pos = (self.pos.0 + dx, self.pos.1 + dy);
        new_piece
    }

    fn rotated(&self) -> Self {
        let mut new_piece = self.clone();
        new_piece.rotation = (self.rotation + 1) % 4;
        new_piece
    }
}

impl GameState {
    fn new() -> Self {
        Self {
            mode: GameMode::Title,
            board: vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: None,
            animation: None,
            score: 0,
            lines_cleared: 0,
            fall_speed: FALL_SPEED_START,
            blocks_to_score: Vec::new(),
        }
    }

    fn ghost_piece(&self) -> Option<Tetromino> {
        self.current_piece.as_ref().map(|piece| {
            let mut ghost = piece.clone();
            while self.is_valid_position(&ghost.moved(0, 1)) {
                ghost = ghost.moved(0, 1);
            }
            ghost
        })
    }

    fn spawn_piece(&mut self) {
        let new_piece = Tetromino::new_random();
        if self.is_valid_position(&new_piece) {
            self.current_piece = Some(new_piece);
        } else {
            self.mode = GameMode::GameOver;
        }
    }

    fn is_valid_position(&self, piece: &Tetromino) -> bool {
        for ((x, y), _) in piece.iter_blocks() {
            if x < 0 || x >= BOARD_WIDTH as i8 || y < 0 || y >= BOARD_HEIGHT as i8 {
                return false;
            }
            if y >= 0 && self.board[y as usize][x as usize] != Cell::Empty {
                return false;
            }
        }
        true
    }

    fn lock_piece(&mut self) {
        if let Some(piece) = self.current_piece.take() {
            for ((x, y), color) in piece.iter_blocks() {
                if y >= 0 {
                    self.board[y as usize][x as usize] = Cell::Occupied(color);
                }
            }
        }

        let lines_to_clear: Vec<usize> = self
            .board
            .iter()
            .enumerate()
            .filter(|(_, row)| row.iter().all(|&cell| matches!(cell, Cell::Occupied(_))))
            .map(|(y, _)| y)
            .collect();

        if !lines_to_clear.is_empty() {
            self.animation = Some(Animation::LineBlink {
                lines: lines_to_clear,
                count: 0,
                start: Instant::now(),
            });
        } else {
            self.spawn_piece();
        }
    }

    fn update_score(&mut self, lines: u32) {
        let points = match lines {
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 0,
        };
        self.score += points;
        self.lines_cleared += lines;
        if self.lines_cleared > 0 && self.lines_cleared % 10 == 0 {
            self.fall_speed = self.fall_speed.saturating_sub(Duration::from_millis(50));
        }
    }

    fn clear_lines(&mut self, lines: &[usize]) {
        // For now, we only handle a single line clear at a time.
        let y = lines[0];
        let is_bottom_line = y == BOARD_HEIGHT - 1;

        if is_bottom_line {
            // Standard Tetris clear for the bottom line
            let num_cleared = lines.len();
            let mut sorted_lines = lines.to_vec();
            sorted_lines.sort_by(|a, b| b.cmp(a));
            for &line_y in &sorted_lines {
                self.board.remove(line_y);
            }
            for _ in 0..num_cleared {
                self.board.insert(0, vec![Cell::Empty; BOARD_WIDTH]);
            }
            self.update_score(num_cleared as u32);
            self.animation = None;
            self.spawn_piece();
        } else {
            // 1. Remove isolated blocks below the cleared line.
            self.remove_isolated_blocks(y);

            // 2. Count connected blocks for scoring (Step 4)
            self.blocks_to_score = count_connected_blocks(&self.board, y);

            // 3. Turn the cleared line to gray (Step 5)
            for x in 0..BOARD_WIDTH {
                self.board[y][x] = Cell::Occupied(Color::Grey);
            }

            // TODO: The next step will be to trigger the push-down animation (Step 6).
            self.animation = Some(Animation::PushDown {
                gray_line_y: y,
                start: Instant::now(),
            });
        }
    }

    fn remove_isolated_blocks(&mut self, cleared_line_y: usize) {
        let mut blocks_to_remove = Vec::new();

        // Iterate from the row below the cleared line to the bottom
        for y in (cleared_line_y + 1)..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                if let Cell::Occupied(color) = self.board[y][x] {
                    // Check neighbors
                    let mut is_isolated = true;
                    let neighbors = [
                        (x as i8 - 1, y as i8),
                        (x as i8 + 1, y as i8),
                        (x as i8, y as i8 - 1),
                        (x as i8, y as i8 + 1),
                    ];

                    for (nx, ny) in neighbors {
                        if nx >= 0
                            && nx < BOARD_WIDTH as i8
                            && ny >= 0
                            && ny < BOARD_HEIGHT as i8
                            && let Cell::Occupied(neighbor_color) =
                                self.board[ny as usize][nx as usize]
                            && neighbor_color == color
                        {
                            is_isolated = false;
                            break;
                        }
                    }

                    if is_isolated {
                        blocks_to_remove.push((x, y));
                    }
                }
            }
        }

        for (x, y) in blocks_to_remove {
            self.board[y][x] = Cell::Empty;
        }
    }

    fn handle_input(&mut self, code: KeyCode) {
        if self.current_piece.is_none() {
            return;
        }
        let mut piece = self.current_piece.clone().unwrap();

        match code {
            KeyCode::Left => piece = piece.moved(-1, 0),
            KeyCode::Right => piece = piece.moved(1, 0),
            KeyCode::Down => {
                piece = piece.moved(0, 1);
                self.score += 1;
            }
            KeyCode::Char(' ') => piece = piece.rotated(),
            KeyCode::Up => {
                while self.is_valid_position(&piece.moved(0, 1)) {
                    piece = piece.moved(0, 1);
                    self.score += 2;
                }
            }
            _ => return,
        }

        if self.is_valid_position(&piece) {
            self.current_piece = Some(piece);
        }
    }
}

fn count_connected_blocks(board: &Board, cleared_line_y: usize) -> Vec<(Point, u32)> {
    let mut results = Vec::new();
    let mut visited = vec![vec![false; BOARD_WIDTH]; BOARD_HEIGHT];

    for y in (cleared_line_y + 1)..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            if let Cell::Occupied(color) = board[y][x] {
                if visited[y][x] {
                    continue;
                }

                let mut component = Vec::new();
                let mut queue = VecDeque::new();

                visited[y][x] = true;
                queue.push_back((x, y));

                while let Some((qx, qy)) = queue.pop_front() {
                    component.push((qx, qy));

                    let neighbors = [
                        (qx as i8 - 1, qy as i8),
                        (qx as i8 + 1, qy as i8),
                        (qx as i8, qy as i8 - 1),
                        (qx as i8, qy as i8 + 1),
                    ];

                    for (nx, ny) in neighbors {
                        if nx >= 0 && nx < BOARD_WIDTH as i8 && ny >= 0 && ny < BOARD_HEIGHT as i8 {
                            let (nx, ny) = (nx as usize, ny as usize);
                            if !visited[ny][nx]
                                && let Cell::Occupied(neighbor_color) = board[ny][nx]
                                && neighbor_color == color
                            {
                                visited[ny][nx] = true;
                                queue.push_back((nx, ny));
                            }
                        }
                    }
                }

                let component_size = component.len() as u32;
                for &(px, py) in &component {
                    results.push(((px, py), component_size));
                }
            }
        }
    }

    results
}

fn draw_title_screen(stdout: &mut io::Stdout) -> io::Result<()> {
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

fn draw(stdout: &mut io::Stdout, prev_state: &GameState, state: &GameState) -> io::Result<()> {
    if prev_state == state {
        return Ok(());
    }

    match state.mode {
        GameMode::Title => { /* Do nothing, handled by draw_title_screen */ }
        GameMode::Playing => {
            if prev_state.mode != GameMode::Playing {
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
                execute!(stdout, MoveTo(ui_x, 7), Print("↑: Hard Drop"))?;
                execute!(stdout, MoveTo(ui_x, 8), Print("↓: Soft Drop"))?;
                execute!(stdout, MoveTo(ui_x, 9), Print("Space: Rotate"))?;
                execute!(stdout, MoveTo(ui_x, 10), Print("q: Quit"))?;
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
                && prev_state.animation.is_none()
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
            let blink_state =
                if let Some(Animation::LineBlink { lines, count, .. }) = &state.animation {
                    Some((lines, *count))
                } else {
                    None
                };

            for (y, row) in state.board.iter().enumerate() {
                // Handle blinking lines
                if let Some((blinking_lines, count)) = blink_state
                    && blinking_lines.contains(&y)
                {
                    let prev_anim_count =
                        if let Some(Animation::LineBlink { count, .. }) = prev_state.animation {
                            Some(count)
                        } else {
                            None
                        };

                    // Redraw if the blink on/off state has changed, or if animation just started.
                    if prev_anim_count.is_none() || (prev_anim_count.unwrap_or(0) % 2 != count % 2)
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
                        }
                    }
                }
            }

            if let Some(ghost) = &state.ghost_piece()
                && Some(ghost) != state.current_piece.as_ref()
            {
                for ((x, y), _) in ghost.iter_blocks() {
                    if y >= 0 && state.board[y as usize][x as usize] == Cell::Empty {
                        execute!(
                            stdout,
                            MoveTo((x as u16 * 2) + 1, y as u16 + 1),
                            SetForegroundColor(Color::Grey),
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
        }
        GameMode::GameOver => {
            if prev_state.mode != GameMode::GameOver {
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

fn handle_scoring(state: &mut GameState) {
    if state.blocks_to_score.is_empty() {
        return;
    }

    let mut total_score = 0;
    for (_, component_size) in &state.blocks_to_score {
        total_score += component_size * 10;
    }

    state.score += total_score;
    state.blocks_to_score.clear();
}

fn handle_animation(state: &mut GameState) {
    if let Some(anim) = state.animation.clone() {
        match anim {
            Animation::LineBlink {
                lines,
                count,
                start,
            } => {
                let steps_done =
                    (start.elapsed().as_millis() / BLINK_ANIMATION_STEP.as_millis()) as usize;

                if steps_done >= BLINK_COUNT_MAX {
                    state.clear_lines(&lines);
                } else if steps_done > count {
                    state.animation = Some(Animation::LineBlink {
                        lines,
                        count: steps_done,
                        start,
                    });
                }
            }
            Animation::PushDown { gray_line_y, start } => {
                let steps_to_move =
                    (start.elapsed().as_millis() / PUSH_DOWN_STEP_DURATION.as_millis()) as usize;

                if steps_to_move == 0 {
                    return;
                }

                let mut current_y = gray_line_y;
                for _ in 0..steps_to_move {
                    if current_y + 1 >= BOARD_HEIGHT {
                        break;
                    }
                    // Remove the row below the gray line
                    state.board.remove(current_y + 1);
                    // Add a new empty row at the top
                    state.board.insert(0, vec![Cell::Empty; BOARD_WIDTH]);
                    current_y += 1;
                }

                if current_y >= BOARD_HEIGHT - 1 {
                    // Animation finished
                    state.animation = None;
                    handle_scoring(state);
                    state.spawn_piece();
                    // Fill the bottom row with Solid cells
                    for x in 0..BOARD_WIDTH {
                        state.board[BOARD_HEIGHT - 1][x] = Cell::Solid;
                    }
                } else {
                    // Update the animation state with the new position
                    state.animation = Some(Animation::PushDown {
                        gray_line_y: current_y,
                        start: Instant::now(), // Reset timer for the next step
                    });
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    execute!(
        stdout,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;
    terminal::enable_raw_mode()?;

    let mut state = GameState::new();
    let mut prev_state = state.clone();
    let mut last_fall = Instant::now();

    draw_title_screen(&mut stdout)?;

    loop {
        if state.mode != GameMode::Title {
            draw(&mut stdout, &prev_state, &state)?;
        }
        prev_state = state.clone();

        match state.mode {
            GameMode::Title => {
                if event::poll(Duration::from_millis(100))?
                    && let Event::Key(key) = event::read()?
                    && key.kind == KeyEventKind::Press
                {
                    match key.code {
                        KeyCode::Enter => {
                            state = GameState::new();
                            state.mode = GameMode::Playing;
                            state.spawn_piece();
                        }
                        KeyCode::Char('q') => break,
                        _ => {}
                    }
                }
            }
            GameMode::Playing => {
                // アニメーション処理
                if state.animation.is_some() {
                    handle_animation(&mut state);
                    continue;
                }

                // 入力処理 (ノンブロッキング)
                let mut last_key_press: Option<KeyCode> = None;
                while event::poll(Duration::ZERO)? {
                    if let Event::Key(key) = event::read()?
                        && key.kind == KeyEventKind::Press
                    {
                        last_key_press = Some(key.code);
                    }
                }
                if let Some(key_code) = last_key_press {
                    if key_code == KeyCode::Char('q') {
                        state.mode = GameMode::GameOver;
                    } else {
                        state.handle_input(key_code);
                    }
                }

                // 落下処理
                if last_fall.elapsed() >= state.fall_speed {
                    if let Some(piece) = &state.current_piece {
                        let moved_down = piece.moved(0, 1);
                        if state.is_valid_position(&moved_down) {
                            state.current_piece = Some(moved_down);
                        } else {
                            state.lock_piece();
                        }
                    } else {
                        state.spawn_piece();
                    }
                    last_fall = Instant::now();
                }

                // ループの速度を調整
                thread::sleep(Duration::from_millis(16));
            }
            GameMode::GameOver => {
                if event::poll(Duration::from_millis(50))?
                    && let Event::Key(key) = event::read()?
                {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                    if key.code == KeyCode::Enter {
                        state = GameState::new();
                        draw_title_screen(&mut stdout)?;
                    }
                }
            }
        }
    }
    execute!(stdout, PopKeyboardEnhancementFlags)?;
    execute!(stdout, Show, LeaveAlternateScreen, ResetColor)?;
    terminal::disable_raw_mode()
}

const SHAPES: [[[(i8, i8); 4]; 4]; 7] = [
    [
        [(1, 0), (1, 1), (1, 2), (1, 3)],
        [(0, 2), (1, 2), (2, 2), (3, 2)],
        [(2, 0), (2, 1), (2, 2), (2, 3)],
        [(0, 1), (1, 1), (2, 1), (3, 1)],
    ],
    [
        [(1, 1), (2, 1), (1, 2), (2, 2)],
        [(1, 1), (2, 1), (1, 2), (2, 2)],
        [(1, 1), (2, 1), (1, 2), (2, 2)],
        [(1, 1), (2, 1), (1, 2), (2, 2)],
    ],
    [
        [(1, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (1, 2)],
        [(1, 0), (0, 1), (1, 1), (1, 2)],
    ],
    [
        [(2, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (1, 2), (2, 2)],
        [(0, 1), (1, 1), (2, 1), (0, 2)],
        [(0, 0), (1, 0), (1, 1), (1, 2)],
    ],
    [
        [(0, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (2, 2)],
        [(1, 0), (1, 1), (0, 2), (1, 2)],
    ],
    [
        [(1, 0), (2, 0), (0, 1), (1, 1)],
        [(1, 0), (1, 1), (2, 1), (2, 2)],
        [(1, 1), (2, 1), (0, 2), (1, 2)],
        [(0, 0), (0, 1), (1, 1), (1, 2)],
    ],
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

    #[test]
    fn test_game_starts_in_title_mode() {
        let state = GameState::new();
        assert_eq!(state.mode, GameMode::Title);
    }

    #[test]
    fn test_line_clear_triggers_blink_animation() {
        let mut state = GameState::new();
        state.mode = GameMode::Playing;

        // Create a full line at the bottom
        for x in 0..BOARD_WIDTH {
            state.board[BOARD_HEIGHT - 1][x] = Cell::Occupied(Color::Blue);
        }

        // Create a piece to lock and trigger the line clear
        let piece = Tetromino::from_shape(
            TetrominoShape::I,
            [Color::Red, Color::Red, Color::Red, Color::Red],
        );
        state.current_piece = Some(piece);

        state.lock_piece();

        assert!(matches!(state.animation, Some(Animation::LineBlink { .. })));
    }

    #[test]
    fn test_bottom_line_is_cleared_normally() {
        let mut state = GameState::new();
        state.mode = GameMode::Playing;

        // Create a full line at the bottom
        for x in 0..BOARD_WIDTH {
            state.board[BOARD_HEIGHT - 1][x] = Cell::Occupied(Color::Blue);
        }
        // Add a marker block on the row above
        state.board[BOARD_HEIGHT - 2][0] = Cell::Occupied(Color::Red);

        // Clear the bottom line
        state.clear_lines(&[BOARD_HEIGHT - 1]);

        // Assert that the marker block has moved down into the bottom row
        assert_eq!(state.board[BOARD_HEIGHT - 1][0], Cell::Occupied(Color::Red));
        // Assert that the top row is now empty
        assert!(state.board[0].iter().all(|&c| c == Cell::Empty));
        // Assert score and line count
        assert_eq!(state.lines_cleared, 1);
        assert_eq!(state.score, 100);
    }

    #[test]
    fn test_isolated_blocks_are_removed_on_non_bottom_clear() {
        let mut state = GameState::new();
        state.mode = GameMode::Playing;

        let clear_line_y = BOARD_HEIGHT - 5;

        // 1. Create a full line at a non-bottom row
        for x in 0..BOARD_WIDTH {
            state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
        }

        // 2. Place an isolated block and a non-isolated group below the line
        let isolated_block_pos = (5, clear_line_y + 2);
        state.board[isolated_block_pos.1][isolated_block_pos.0] = Cell::Occupied(Color::Red);

        let non_isolated_group_pos1 = (2, clear_line_y + 3);
        let non_isolated_group_pos2 = (3, clear_line_y + 3);
        state.board[non_isolated_group_pos1.1][non_isolated_group_pos1.0] =
            Cell::Occupied(Color::Green);
        state.board[non_isolated_group_pos2.1][non_isolated_group_pos2.0] =
            Cell::Occupied(Color::Green);

        // 3. Call the line clear logic
        state.clear_lines(&[clear_line_y]);

        // 4. Assert that the isolated block is gone
        assert_eq!(
            state.board[isolated_block_pos.1][isolated_block_pos.0],
            Cell::Empty,
            "Isolated block should be removed"
        );

        // 5. Assert that the non-isolated group remains
        assert_ne!(
            state.board[non_isolated_group_pos1.1][non_isolated_group_pos1.0],
            Cell::Empty,
            "Non-isolated block should remain"
        );
        assert_ne!(
            state.board[non_isolated_group_pos2.1][non_isolated_group_pos2.0],
            Cell::Empty,
            "Non-isolated block should remain"
        );
    }

    #[test]
    fn test_counts_connected_blocks() {
        let mut board = vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        let cleared_line_y = 15;

        // Setup a 2x2 group of green blocks
        let green_group = [
            (2, cleared_line_y + 2),
            (3, cleared_line_y + 2),
            (2, cleared_line_y + 3),
            (3, cleared_line_y + 3),
        ];
        for &(x, y) in &green_group {
            board[y][x] = Cell::Occupied(Color::Green);
        }

        // Setup a single isolated red block
        let red_block = (7, cleared_line_y + 1);
        board[red_block.1][red_block.0] = Cell::Occupied(Color::Red);

        let mut results = count_connected_blocks(&board, cleared_line_y);
        results.sort_by_key(|k| (k.0.1, k.0.0)); // Sort for consistent order

        let mut expected = vec![
            (red_block, 1),
            (green_group[0], 4),
            (green_group[1], 4),
            (green_group[2], 4),
            (green_group[3], 4),
        ];
        expected.sort_by_key(|k| (k.0.1, k.0.0));

        assert_eq!(results, expected);
    }

    #[test]
    fn test_cleared_non_bottom_line_turns_gray() {
        let mut state = GameState::new();
        let clear_line_y = BOARD_HEIGHT - 5;

        // Create a full line at a non-bottom row
        for x in 0..BOARD_WIDTH {
            state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
        }

        // Call the line clear logic
        state.clear_lines(&[clear_line_y]);

        // Assert that the cleared line has turned gray
        for x in 0..BOARD_WIDTH {
            assert_eq!(state.board[clear_line_y][x], Cell::Occupied(Color::Grey));
        }
    }

    #[test]
    fn test_non_bottom_clear_triggers_pushdown() {
        let mut state = GameState::new();
        let clear_line_y = BOARD_HEIGHT - 5;

        // Create a full line at a non-bottom row
        for x in 0..BOARD_WIDTH {
            state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
        }

        // Call the line clear logic
        state.clear_lines(&[clear_line_y]);

        // Assert that the correct animation has been triggered
        assert!(matches!(state.animation, Some(Animation::PushDown { .. })));
    }

    #[test]
    fn test_pushdown_animation_moves_line() {
        let mut state = GameState::new();
        let clear_line_y = BOARD_HEIGHT - 5;
        let marker_y = clear_line_y + 1;
        let marker_x = 3;

        // Create a full line at a non-bottom row
        for x in 0..BOARD_WIDTH {
            state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
        }
        // Create a marker block in the row below the cleared line
        state.board[marker_y][marker_x] = Cell::Occupied(Color::Red);

        // 1. Trigger the line clear and subsequent pushdown animation
        state.clear_lines(&[clear_line_y]);

        // The line should now be gray
        assert_eq!(state.board[clear_line_y][0], Cell::Occupied(Color::Grey));

        // 2. Manually handle the animation step
        // In a real game loop, this would be called repeatedly.
        handle_animation(&mut state);
        thread::sleep(PUSH_DOWN_STEP_DURATION);
        handle_animation(&mut state);

        // 3. Assert the board state has changed
        // The gray line should have moved down one step
        assert_eq!(
            state.board[clear_line_y + 1][0],
            Cell::Occupied(Color::Grey)
        );
        // The original gray line row should now be empty
        assert_eq!(state.board[clear_line_y][0], Cell::Empty);
        // The marker block should be gone because its row was deleted and replaced by the gray line
        assert_eq!(
            state.board[clear_line_y + 1][marker_x],
            Cell::Occupied(Color::Grey)
        );
    }

    #[test]
    fn test_scoring_after_pushdown() {
        let mut state = GameState::new();
        let clear_line_y = BOARD_HEIGHT - 5;

        // Setup a 2x2 group of green blocks below the clear line
        let green_group = [
            (2, clear_line_y + 2),
            (3, clear_line_y + 2),
            (2, clear_line_y + 3),
            (3, clear_line_y + 3),
        ];
        for &(x, y) in &green_group {
            state.board[y][x] = Cell::Occupied(Color::Green);
        }

        // The `blocks_to_score` is populated by `clear_lines`
        state.blocks_to_score = count_connected_blocks(&state.board, clear_line_y);
        assert_eq!(state.blocks_to_score.len(), 4); // Sanity check

        // Manually call the scoring logic
        handle_scoring(&mut state);

        // Each of the 4 blocks is in a component of size 4, so 4 * (4 * 10) = 160
        assert_eq!(state.score, 160);
        // The scoring list should be cleared after processing
        assert!(state.blocks_to_score.is_empty());
    }

    #[test]
    fn test_solid_cell_is_collision() {
        let mut state = GameState::new();
        let solid_pos = (4, 5);
        state.board[solid_pos.1][solid_pos.0] = Cell::Solid;

        let mut piece = Tetromino::from_shape(TetrominoShape::I, COLOR_PALETTE);
        // Position the piece to overlap with the solid cell
        piece.pos = (solid_pos.0 as i8 - 1, solid_pos.1 as i8 - 1);

        assert!(!state.is_valid_position(&piece));
    }

    #[test]
    fn test_pushdown_finishes_with_solid_line() {
        let mut state = GameState::new();
        let clear_line_y = BOARD_HEIGHT - 2; // Clear line near bottom

        // Create a full line at a non-bottom row
        for x in 0..BOARD_WIDTH {
            state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
        }

        // Trigger the line clear and subsequent pushdown animation
        state.clear_lines(&[clear_line_y]);

        // Manually advance the gray line to the bottom
        if let Some(Animation::PushDown {
            gray_line_y: _,
            start,
        }) = &mut state.animation
        {
            *start =
                Instant::now() - PUSH_DOWN_STEP_DURATION * (BOARD_HEIGHT - 1 - clear_line_y) as u32;
        }

        // Call handle_animation to process the final step
        handle_animation(&mut state);

        // Assert that the bottom row is now solid
        for x in 0..BOARD_WIDTH {
            assert_eq!(state.board[BOARD_HEIGHT - 1][x], Cell::Solid);
        }
        // Assert that the animation is finished
        assert!(state.animation.is_none());
    }
}
