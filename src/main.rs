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

// --- 時間管理 ---
pub trait TimeProvider {
    fn now(&self) -> Duration;
}

pub struct SystemTimeProvider {
    start: Instant,
}

impl SystemTimeProvider {
    pub fn new() -> Self {
        Self { start: Instant::now() }
    }
}

impl Default for SystemTimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeProvider for SystemTimeProvider {
    fn now(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
pub struct MockTimeProvider {
    current_time: Duration,
}

#[cfg(test)]
impl MockTimeProvider {
    pub fn new() -> Self {
        Self {
            current_time: Duration::from_secs(0),
        }
    }

    pub fn advance(&mut self, duration: Duration) {
        self.current_time += duration;
    }
}

#[cfg(test)]
impl TimeProvider for MockTimeProvider {
    fn now(&self) -> Duration {
        self.current_time
    }
}

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
        start_time: Duration,
    },
    PushDown {
        gray_line_y: usize,
        start_time: Duration,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct GameState {
    mode: GameMode,
    board: Board,
    current_piece: Option<Tetromino>,
    next_piece: Option<Tetromino>,
    animation: Vec<Animation>,
    score: u32,
    lines_cleared: u32,
    fall_speed: Duration,
    blocks_to_score: Vec<(Point, u32)>,
    current_board_height: usize,
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

    fn rotated_counter_clockwise(&self) -> Self {
        let mut new_piece = self.clone();
        new_piece.rotation = (self.rotation + 3) % 4;
        new_piece
    }
}

impl GameState {
    fn new() -> Self {
        Self {
            mode: GameMode::Title,
            board: vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: None,
            next_piece: Some(Tetromino::new_random()), // next_pieceを初期化
            animation: Vec::new(),
            score: 0,
            lines_cleared: 0,
            fall_speed: FALL_SPEED_START,
            blocks_to_score: Vec::new(),
            current_board_height: BOARD_HEIGHT,
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
        // next_pieceをcurrent_pieceにする
        self.current_piece = self.next_piece.take();
        // 新しいnext_pieceを生成する
        self.next_piece = Some(Tetromino::new_random());

        // current_pieceが有効な位置にあるかチェック
        if let Some(piece) = &self.current_piece
            && !self.is_valid_position(piece)
        {
            self.mode = GameMode::GameOver;
        }
    }

    fn is_valid_position(&self, piece: &Tetromino) -> bool {
        for ((x, y), _) in piece.iter_blocks() {
            if x < 0 || x >= BOARD_WIDTH as i8 || y < 0 || y >= self.current_board_height as i8 {
                return false;
            }
            if y >= 0 && self.board[y as usize][x as usize] != Cell::Empty {
                return false;
            }
        }
        true
    }

    fn lock_piece(&mut self, time_provider: &dyn TimeProvider) {
        if let Some(piece) = self.current_piece.take() {
            for ((x, y), color) in piece.iter_blocks() {
                if y >= 0 {
                    self.board[y as usize][x as usize] = Cell::Occupied(color);
                }
            }
        }

        let mut lines_to_clear: Vec<usize> = self.board[0..self.current_board_height]
            .iter()
            .enumerate()
            .filter(|(_, row)| row.iter().all(|&cell| matches!(cell, Cell::Occupied(_))))
            .map(|(y, _)| y)
            .collect();
        lines_to_clear.sort_by(|a, b| b.cmp(a)); // Sort in descending order to clear from bottom up

        if !lines_to_clear.is_empty() {
            self.animation.push(Animation::LineBlink {
                lines: lines_to_clear,
                count: 0,
                start_time: time_provider.now(),
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

    fn clear_lines(&mut self, lines: &[usize], time_provider: &dyn TimeProvider) -> Vec<Animation> {
        let mut new_animations = Vec::new();
        let mut bottom_lines_cleared = Vec::new();
        let mut non_bottom_lines_cleared = Vec::new();

        for &y in lines {
            if y == self.current_board_height - 1 {
                bottom_lines_cleared.push(y);
            } else {
                non_bottom_lines_cleared.push(y);
            }
        }

        // Handle standard Tetris clear for bottom lines first
        if !bottom_lines_cleared.is_empty() {
            let num_cleared = bottom_lines_cleared.len();
            let mut sorted_lines = bottom_lines_cleared.to_vec();
            sorted_lines.sort_by(|a, b| b.cmp(a));
            for &line_y in &sorted_lines {
                self.board.remove(line_y);
            }
            for _ in 0..num_cleared {
                self.board.insert(0, vec![Cell::Empty; BOARD_WIDTH]);
            }
            self.update_score(num_cleared as u32);
            // No animation for standard clear, just spawn new piece
            self.spawn_piece();
        }

        // Handle custom clear for non-bottom lines
        for &y in &non_bottom_lines_cleared {
            // 1. Remove isolated blocks below the cleared line.
            self.remove_isolated_blocks(y);

            // 2. Count connected blocks for scoring (Step 4)
            // This will be handled by handle_animation when PushDown finishes
            let connected_blocks = count_connected_blocks(&self.board, y);
            self.blocks_to_score.extend(connected_blocks);

            // 3. Turn the cleared line to gray (Step 5)
            for x in 0..BOARD_WIDTH {
                self.board[y][x] = Cell::Occupied(Color::Grey);
            }

            // Trigger the push-down animation (Step 6).
            new_animations.push(Animation::PushDown {
                gray_line_y: y,
                start_time: time_provider.now(),
            });
        }

        // If no non-bottom lines were cleared, and no bottom lines were cleared, spawn a new piece
        if bottom_lines_cleared.is_empty() && non_bottom_lines_cleared.is_empty() {
            self.spawn_piece();
        }
        new_animations
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

    fn handle_input(&mut self, key_event: event::KeyEvent) {
        if self.current_piece.is_none() {
            return;
        }
        let mut piece = self.current_piece.clone().unwrap();

        match key_event.code {
            KeyCode::Left => piece = piece.moved(-1, 0),
            KeyCode::Right => piece = piece.moved(1, 0),
            KeyCode::Down => {
                if key_event.modifiers.contains(event::KeyModifiers::SHIFT) {
                    // Hard Drop
                    while self.is_valid_position(&piece.moved(0, 1)) {
                        piece = piece.moved(0, 1);
                        self.score += 2;
                    }
                } else {
                    // Clockwise Rotation
                    piece = piece.rotated();
                }
            }
            KeyCode::Up => {
                // Counter-Clockwise Rotation
                piece = piece.rotated_counter_clockwise();
            }
            KeyCode::Char(' ') => {
                // Soft Drop
                piece = piece.moved(0, 1);
                self.score += 1;
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
                    let prev_anim_count = if let Some(Animation::LineBlink { count, .. }) = prev_state
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

            // NEXTミノの描画
            // 以前のNEXTミノをクリア
            if let Some(prev_next_piece) = &prev_state.next_piece
                && prev_state.next_piece != state.next_piece
            {
                // NEXTミノが変更された場合
                let next_piece_offset_x = ui_x;
                let next_piece_offset_y = 7;
                for ((x, y), _) in prev_next_piece.iter_blocks() {
                    let draw_x = next_piece_offset_x + (x as u16 * 2);
                    let draw_y = next_piece_offset_y + y as u16;
                    execute!(stdout, MoveTo(draw_x, draw_y), Print("  "))?;
                }
            }

            if let Some(next_piece) = &state.next_piece {
                execute!(stdout, SetForegroundColor(Color::White))?;
                execute!(stdout, MoveTo(ui_x, 5), Print("NEXT:"))?; // "NEXT:" ラベル
                let next_piece_offset_x = ui_x;
                let next_piece_offset_y = 7; // "NEXT:" の下あたりに描画

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

fn handle_animation(state: &mut GameState, time_provider: &dyn TimeProvider) {
    if state.animation.is_empty() {
        return;
    }

    let mut still_animating = Vec::new();
    let mut animations_finished = false;
    let now = time_provider.now();

    // Take ownership to process
    for anim in std::mem::take(&mut state.animation) {
        match anim {
            Animation::LineBlink { lines, count: _, start_time } => {
                let steps_done =
                    ((now - start_time).as_millis() / BLINK_ANIMATION_STEP.as_millis()) as usize;

                if steps_done >= BLINK_COUNT_MAX {
                    // Blinking finished, trigger the next stage (clearing)
                    still_animating.extend(state.clear_lines(&lines, time_provider));
                } else {
                    // Continue blinking
                    still_animating.push(Animation::LineBlink {
                        lines,
                        count: steps_done,
                        start_time,
                    });
                }
            }
            Animation::PushDown {
                gray_line_y,
                start_time,
            } => {
                if now - start_time >= PUSH_DOWN_STEP_DURATION {
                    let target_y = gray_line_y + 1;

                    // Check if the animation should finish
                    if target_y >= state.current_board_height
                        || state.board[target_y][0] == Cell::Solid
                    {
                        // Finalize the line as Solid
                        for x in 0..BOARD_WIDTH {
                            state.board[gray_line_y][x] = Cell::Solid;
                        }
                        state.current_board_height = state.current_board_height.saturating_sub(1);
                        handle_scoring(state); // Score after a line settles
                        animations_finished = true;
                        // Do not push the animation back
                    } else {
                        // Move the gray line and everything above it down by removing the line below it
                        // and inserting a new empty line at the top.
                        state.board.remove(target_y);
                        state.board.insert(0, vec![Cell::Empty; BOARD_WIDTH]);

                        // Push the animation back with updated state
                        still_animating.push(Animation::PushDown {
                            gray_line_y: target_y,
                            start_time: now, // Reset timer for the next step
                        });
                    }
                } else {
                    // Not time to move yet, keep it in the queue
                    still_animating.push(Animation::PushDown {
                        gray_line_y,
                        start_time,
                    });
                }
            }
        }
    }

    state.animation = still_animating;

    // Spawn a new piece only if all animations have completed in this cycle.
    if animations_finished && state.animation.is_empty() {
        state.spawn_piece();
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

    let time_provider = SystemTimeProvider::new();
    let mut state = GameState::new();
    let mut prev_state = state.clone();
    let mut last_fall = time_provider.now();

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
                if !state.animation.is_empty() {
                    handle_animation(&mut state, &time_provider);
                    continue;
                }

                // 入力処理 (ノンブロッキング)
                let mut last_key_event: Option<event::KeyEvent> = None;
                while event::poll(Duration::ZERO)? {
                    if let Event::Key(key) = event::read()?
                        && key.kind == KeyEventKind::Press
                    {
                        last_key_event = Some(key);
                    }
                }
                if let Some(key_event) = last_key_event {
                    if key_event.code == KeyCode::Char('q') {
                        state.mode = GameMode::GameOver;
                    } else {
                        state.handle_input(key_event);
                    }
                }

                // 落下処理
                if time_provider.now() - last_fall >= state.fall_speed {
                    if let Some(piece) = &state.current_piece {
                        let moved_down = piece.moved(0, 1);
                        if state.is_valid_position(&moved_down) {
                            state.current_piece = Some(moved_down);
                        } else {
                            state.lock_piece(&time_provider);
                        }
                    } else {
                        state.spawn_piece();
                    }
                    last_fall = time_provider.now();
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
        let time_provider = MockTimeProvider::new();
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

        state.lock_piece(&time_provider);

        assert!(
            state
                .animation
                .iter()
                .any(|anim| matches!(anim, Animation::LineBlink { .. }))
        );
    }

    #[test]
    fn test_bottom_line_is_cleared_normally() {
        let time_provider = MockTimeProvider::new();
        let mut state = GameState::new();
        state.mode = GameMode::Playing;

        // Create a full line at the bottom
        for x in 0..BOARD_WIDTH {
            state.board[BOARD_HEIGHT - 1][x] = Cell::Occupied(Color::Blue);
        }
        // Add a marker block on the row above
        state.board[BOARD_HEIGHT - 2][0] = Cell::Occupied(Color::Red);

        // Clear the bottom line
        let new_animations = state.clear_lines(&[BOARD_HEIGHT - 1], &time_provider);
        state.animation.extend(new_animations);

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
        let time_provider = MockTimeProvider::new();
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
        let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
        state.animation.extend(new_animations);

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
        let time_provider = MockTimeProvider::new();
        let mut state = GameState::new();
        let clear_line_y = BOARD_HEIGHT - 5;

        // Create a full line at a non-bottom row
        for x in 0..BOARD_WIDTH {
            state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
        }

        // Call the line clear logic
        let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
        state.animation.extend(new_animations);

        // Assert that the cleared line has turned gray
        for x in 0..BOARD_WIDTH {
            assert_eq!(state.board[clear_line_y][x], Cell::Occupied(Color::Grey));
        }
    }

    #[test]
    fn test_non_bottom_clear_triggers_pushdown() {
        let time_provider = MockTimeProvider::new();
        let mut state = GameState::new();
        let clear_line_y = BOARD_HEIGHT - 5;

        // Create a full line at a non-bottom row
        for x in 0..BOARD_WIDTH {
            state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
        }

        // Call the line clear logic and capture the resulting animations
        let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
        state.animation.extend(new_animations);

        assert!(state.animation.iter().any(|anim| matches!(
            anim,
            Animation::PushDown { gray_line_y, .. } if *gray_line_y == clear_line_y
        )));
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
        let mut time_provider = MockTimeProvider::new();
        let mut state = GameState::new();
        let clear_line_y = BOARD_HEIGHT - 2; // Clear line near bottom

        // Create a full line at a non-bottom row
        for x in 0..BOARD_WIDTH {
            state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
        }

        // Trigger the line clear and subsequent pushdown animation
        let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
        state.animation.extend(new_animations);

        // Loop until the animation is complete
        while !state.animation.is_empty() {
            time_provider.advance(PUSH_DOWN_STEP_DURATION);
            handle_animation(&mut state, &time_provider);
        }

        // Assert that the bottom row is now solid
        for x in 0..BOARD_WIDTH {
            assert_eq!(state.board[BOARD_HEIGHT - 1][x], Cell::Solid);
        }
    }

    #[test]
    fn test_lock_piece_ignores_solid_lines() {
        let mut time_provider = MockTimeProvider::new();
        let mut state = GameState::new();
        state.mode = GameMode::Playing;

        // Create a solid line at the bottom
        for x in 0..BOARD_WIDTH {
            state.board[BOARD_HEIGHT - 1][x] = Cell::Solid;
        }
        // Create an occupied line above it
        for x in 0..BOARD_WIDTH {
            state.board[BOARD_HEIGHT - 2][x] = Cell::Occupied(Color::Blue);
        }

        // Create a piece to lock and trigger the line clear
        let piece = Tetromino::from_shape(
            TetrominoShape::I,
            [Color::Red, Color::Red, Color::Red, Color::Red],
        );
        state.current_piece = Some(piece);

        state.lock_piece(&time_provider);

        // Manually advance the blink animation to completion
        time_provider.advance(BLINK_ANIMATION_STEP * BLINK_COUNT_MAX as u32);
        handle_animation(&mut state, &time_provider); // Line clear should now have happened

        // Assert that the solid line remains
        for x in 0..BOARD_WIDTH {
            assert_eq!(state.board[BOARD_HEIGHT - 1][x], Cell::Solid);
        }
        // Assert that the occupied line turned gray and triggered PushDown animation
        for x in 0..BOARD_WIDTH {
            assert_eq!(
                state.board[BOARD_HEIGHT - 2][x],
                Cell::Occupied(Color::Grey)
            );
        }
        let expected_gray_line_y = BOARD_HEIGHT - 2;
        assert!(state.animation.iter().any(|anim| {
            if let Animation::PushDown { gray_line_y: y, .. } = anim {
                *y == expected_gray_line_y
            } else {
                false
            }
        }));
        // Assert score and line count (no score yet, as PushDown animation is ongoing)
        assert_eq!(state.lines_cleared, 0);
        assert_eq!(state.score, 0);
    }

    #[test]
    fn test_pushdown_animation_moves_line() {
        // Setup: Time provider and initial state
        let mut time_provider = MockTimeProvider::new();
        let mut state = GameState::new();
        let clear_line_y = BOARD_HEIGHT - 5;

        // Create a full line
        for x in 0..BOARD_WIDTH {
            state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
        }

        // Trigger the animation
        let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
        state.animation.extend(new_animations);

        // Advance time and handle animation
        time_provider.advance(PUSH_DOWN_STEP_DURATION);
        handle_animation(&mut state, &time_provider);

        // Assert: The gray line has moved down one step
        assert_eq!(
            state.board[clear_line_y + 1][0],
            Cell::Occupied(Color::Grey),
            "Gray line should have moved down"
        );
        assert_eq!(
            state.board[clear_line_y][0],
            Cell::Empty,
            "Original gray line row should be empty"
        );
    }

    #[test]
    fn test_multiple_gray_lines_stack_and_reduce_board_height() {
        let mut time_provider = MockTimeProvider::new();
        let mut state = GameState::new();
        state.mode = GameMode::Playing;

        // 1. Clear a line at BOARD_HEIGHT - 5
        let clear_line_y1 = BOARD_HEIGHT - 5;
        for x in 0..BOARD_WIDTH {
            state.board[clear_line_y1][x] = Cell::Occupied(Color::Blue);
        }
        let new_animations = state.clear_lines(&[clear_line_y1], &time_provider);
        state.animation.extend(new_animations);

        // Loop until the first animation is complete
        while !state.animation.is_empty() {
            time_provider.advance(PUSH_DOWN_STEP_DURATION);
            handle_animation(&mut state, &time_provider);
        }

        // Assert first gray line is solid and board height reduced
        for x in 0..BOARD_WIDTH {
            assert_eq!(
                state.board[BOARD_HEIGHT - 1][x],
                Cell::Solid,
                "First gray line should be solid"
            );
        }
        assert_eq!(
            state.current_board_height,
            BOARD_HEIGHT - 1,
            "Board height should be reduced by 1 after first clear"
        );

        // 2. Clear a second line at a higher position
        let clear_line_y2 = BOARD_HEIGHT - 10;
        for x in 0..BOARD_WIDTH {
            state.board[clear_line_y2][x] = Cell::Occupied(Color::Green);
        }
        let new_animations = state.clear_lines(&[clear_line_y2], &time_provider);
        state.animation.extend(new_animations);

        // Loop until the second animation is complete
        while !state.animation.is_empty() {
            time_provider.advance(PUSH_DOWN_STEP_DURATION);
            handle_animation(&mut state, &time_provider);
        }

        // Assert second gray line is solid and board height reduced further
        // It should settle on top of the first solid line, at BOARD_HEIGHT - 2
        for x in 0..BOARD_WIDTH {
            assert_eq!(
                state.board[BOARD_HEIGHT - 2][x],
                Cell::Solid,
                "Second gray line should be solid on top of the first"
            );
        }
        assert_eq!(
            state.current_board_height,
            BOARD_HEIGHT - 2,
            "Board height should be reduced by 2 after second clear"
        );

        // Verify that new pieces would spawn above the solid lines
        state.spawn_piece(); // A new piece should have spawned automatically
        assert!(
            state.current_piece.is_some(),
            "A new piece should spawn after animations"
        );

        // Position a test piece to overlap with the solid lines
        let mut colliding_piece = Tetromino::from_shape(TetrominoShape::I, COLOR_PALETTE);
        colliding_piece.pos = (0, (state.current_board_height as i8) - 1); // Place it on the top solid line
        assert!(
            !state.is_valid_position(&colliding_piece),
            "Piece should not be valid on solid lines"
        );
    }

    #[test]
    fn test_blocks_above_follow_pushed_down_line() {
        let mut time_provider = MockTimeProvider::new();
        let mut state = GameState::new();
        let clear_line_y = BOARD_HEIGHT - 5;
        let marker_x = 3;
        let marker_y = clear_line_y - 1; // Place a marker block *above* the line to be cleared

        // Create a full line to be cleared
        for x in 0..BOARD_WIDTH {
            state.board[clear_line_y][x] = Cell::Occupied(Color::Blue);
        }
        // Place the marker block
        state.board[marker_y][marker_x] = Cell::Occupied(Color::Red);

        // Trigger the animation
        let new_animations = state.clear_lines(&[clear_line_y], &time_provider);
        state.animation.extend(new_animations);

        // Advance time and handle animation for one step
        time_provider.advance(PUSH_DOWN_STEP_DURATION);
        handle_animation(&mut state, &time_provider);

        // Assert: The marker block should have moved down with the gray line
        assert_eq!(
            state.board[marker_y + 1][marker_x],
            Cell::Occupied(Color::Red),
            "Block on top should move down"
        );
        assert_eq!(
            state.board[marker_y][marker_x],
            Cell::Empty,
            "Original position of marker block should be empty"
        );
    }
}
