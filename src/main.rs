use std::io::{self, stdout, Write};
use std::time::{Duration, Instant};
use std::thread;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::{self, Rng};
use rand::seq::SliceRandom;

// --- 定数 ---
const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;
const FALL_SPEED_START: Duration = Duration::from_millis(800);
const LINE_CLEAR_ANIMATION_DELAY: Duration = Duration::from_millis(40);
const COLOR_PALETTE: [Color; 4] = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];

// --- データ構造 ---

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cell {
    Empty,
    Occupied(Color),
}

type Board = Vec<Vec<Cell>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TetrominoShape {
    I, O, T, L, J, S, Z,
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
struct LineClearAnimation {
    lines: Vec<usize>,
    step: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct GameState {
    board: Board,
    current_piece: Option<Tetromino>,
    animation: Option<LineClearAnimation>,
    game_over: bool,
    score: u32,
    lines_cleared: u32,
    fall_speed: Duration,
}

impl Tetromino {
    fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        let shape = match rng.gen_range(0..7) {
            0 => TetrominoShape::I, 1 => TetrominoShape::O, 2 => TetrominoShape::T,
            3 => TetrominoShape::L, 4 => TetrominoShape::J, 5 => TetrominoShape::S,
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
            _shape: shape, matrix,
            pos: ((BOARD_WIDTH as i8) / 2 - 2, 0),
            colors, rotation: 0,
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
            board: vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: None,
            animation: None,
            game_over: false,
            score: 0,
            lines_cleared: 0,
            fall_speed: FALL_SPEED_START,
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
            self.game_over = true;
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
        
        let lines_to_clear: Vec<usize> = self.board.iter().enumerate()
            .filter(|(_, row)| row.iter().all(|&cell| matches!(cell, Cell::Occupied(_))))
            .map(|(y, _)| y)
            .collect();

        if !lines_to_clear.is_empty() {
            self.animation = Some(LineClearAnimation { lines: lines_to_clear, step: 0 });
        } else {
            self.spawn_piece();
        }
    }

    fn update_score(&mut self, lines: u32) {
        let points = match lines {
            1 => 100, 2 => 300, 3 => 500, 4 => 800, _ => 0,
        };
        self.score += points;
        self.lines_cleared += lines;
        if self.lines_cleared > 0 && self.lines_cleared % 10 == 0 {
            self.fall_speed = self.fall_speed.saturating_sub(Duration::from_millis(50));
        }
    }

    fn handle_input(&mut self, code: KeyCode) {
        if self.current_piece.is_none() { return; }
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

fn draw(stdout: &mut io::Stdout, prev_state: &GameState, state: &GameState) -> io::Result<()> {
    if prev_state == state { return Ok(()); }

    // --- 消去フェーズ ---
    // 古いゴーストとピースがあった場所をクリア
    if let Some(ghost) = &prev_state.ghost_piece() {
        if Some(ghost) != prev_state.current_piece.as_ref() {
            for ((x, y), _) in ghost.iter_blocks() {
                if y >= 0 { execute!(stdout, MoveTo((x as u16 * 2) + 1, y as u16 + 1), Print("  "))?; }
            }
        }
    }
    if let Some(piece) = &prev_state.current_piece {
        if prev_state.animation.is_none() {
            for ((x, y), _) in piece.iter_blocks() {
                if y >= 0 { execute!(stdout, MoveTo((x as u16 * 2) + 1, y as u16 + 1), Print("  "))?; }
            }
        }
    }

    // --- 描画フェーズ ---

    // ボードの差分描画 (クリアした部分を元に戻すため、差分だけでなく前のピース/ゴーストがあった場所も再描画)
    for (y, row) in state.board.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            let pos = (x as i8, y as i8);
            let was_ghost = prev_state.ghost_piece().as_ref().map_or(false, |g| g.iter_blocks().any(|(p, _)| p == pos));
            let was_piece = prev_state.current_piece.as_ref().map_or(false, |p| p.iter_blocks().any(|(p, _)| p == pos));

            if cell != prev_state.board[y][x] || ((was_ghost || was_piece) && cell != Cell::Empty) {
                execute!(stdout, MoveTo((x as u16 * 2) + 1, y as u16 + 1))?;
                match cell {
                    Cell::Empty => execute!(stdout, Print("  "))?,
                    Cell::Occupied(color) => execute!(stdout, SetForegroundColor(color), Print("[]"), ResetColor)?,
                }
            }
        }
    }
    
    // ゴーストピースを描画
    if let Some(ghost) = &state.ghost_piece() {
        if Some(ghost) != state.current_piece.as_ref() {
            for ((x, y), _) in ghost.iter_blocks() {
                if y >= 0 && state.board[y as usize][x as usize] == Cell::Empty {
                    execute!(stdout, MoveTo((x as u16 * 2) + 1, y as u16 + 1), SetForegroundColor(Color::Grey), Print("::"))?;
                }
            }
        }
    }

    // 現在のピースを描画
    if let Some(piece) = &state.current_piece {
        for ((x, y), color) in piece.iter_blocks() {
            if y >= 0 {
                execute!(stdout, MoveTo((x as u16 * 2) + 1, y as u16 + 1), SetForegroundColor(color), Print("[]"), ResetColor)?;
            }
        }
    }

    // UIの差分描画
    let ui_x = (BOARD_WIDTH * 2 + 4) as u16;
    if prev_state.score != state.score {
        execute!(stdout, SetForegroundColor(Color::White), MoveTo(ui_x, 2), Print(format!("Score: {:<6}", state.score)))?;
    }
    if prev_state.lines_cleared != state.lines_cleared {
        execute!(stdout, MoveTo(ui_x, 3), Print(format!("Lines: {:<6}", state.lines_cleared)))?;
    }

    if state.game_over && !prev_state.game_over {
        let msg = "GAME OVER";
        let x = (BOARD_WIDTH * 2 + 3 - msg.len()) as u16 / 2;
        let y = (BOARD_HEIGHT / 2) as u16;
        execute!(stdout, SetForegroundColor(Color::Red), MoveTo(x, y), Print(msg))?;
    }

    stdout.flush()
}

fn main() -> io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?; 
    execute!(stdout, PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES))?;
    terminal::enable_raw_mode()?;

    // 静的要素の初期描画
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    execute!(stdout, SetForegroundColor(Color::Grey))?;
    execute!(stdout, MoveTo(0, 0), Print("┌"))?;
    execute!(stdout, MoveTo((BOARD_WIDTH * 2) as u16 + 1, 0), Print("┐"))?;
    execute!(stdout, MoveTo(0, BOARD_HEIGHT as u16 + 1), Print("└"))?;
    execute!(stdout, MoveTo((BOARD_WIDTH * 2) as u16 + 1, BOARD_HEIGHT as u16 + 1), Print("┘"))?;
    for y in 1..=BOARD_HEIGHT {
        execute!(stdout, MoveTo(0, y as u16), Print("│"))?;
        execute!(stdout, MoveTo((BOARD_WIDTH * 2) as u16 + 1, y as u16), Print("│"))?;
    }
    for x in 0..BOARD_WIDTH {
        execute!(stdout, MoveTo((x * 2) as u16 + 1, 0), Print("──"))?;
        execute!(stdout, MoveTo((x * 2) as u16 + 1, BOARD_HEIGHT as u16 + 1), Print("──"))?;
    }
    execute!(stdout, ResetColor)?;
    let ui_x = (BOARD_WIDTH * 2 + 4) as u16;
    execute!(stdout, SetForegroundColor(Color::White), MoveTo(ui_x, 2), Print("Score: 0     "))?;
    execute!(stdout, MoveTo(ui_x, 3), Print("Lines: 0     "))?;
    execute!(stdout, MoveTo(ui_x, 5), Print("Controls:"))?;
    execute!(stdout, MoveTo(ui_x, 6), Print("←/→: Move"))?;
    execute!(stdout, MoveTo(ui_x, 7), Print("↑: Hard Drop"))?;
    execute!(stdout, MoveTo(ui_x, 8), Print("↓: Soft Drop"))?;
    execute!(stdout, MoveTo(ui_x, 9), Print("Space: Rotate"))?;
    execute!(stdout, MoveTo(ui_x, 10), Print("q: Quit"))?;
    
    let mut state = GameState::new();
    let mut prev_state = state.clone();
    state.spawn_piece();
    let mut last_fall = Instant::now();

    loop {
        draw(&mut stdout, &prev_state, &state)?;
        prev_state = state.clone();

        if state.game_over {
            break;
        }

        // アニメーション処理
        if let Some(mut anim) = state.animation.clone() {
            thread::sleep(LINE_CLEAR_ANIMATION_DELAY);
            for &y in &anim.lines {
                state.board[y][anim.step] = Cell::Empty;
            }
            anim.step += 1;

            if anim.step >= BOARD_WIDTH {
                let num_cleared = anim.lines.len();
                let mut sorted_lines = anim.lines.clone();
                sorted_lines.sort_by(|a, b| b.cmp(a));
                for &y in &sorted_lines {
                    state.board.remove(y);
                }
                for _ in 0..num_cleared {
                    state.board.insert(0, vec![Cell::Empty; BOARD_WIDTH]);
                }
                state.update_score(num_cleared as u32);
                state.animation = None;
                state.spawn_piece();
            } else {
                state.animation = Some(anim);
            }
            continue;
        }

        // 入力処理 (ノンブロッキング)
        let mut last_key_press: Option<KeyCode> = None;
        while event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    last_key_press = Some(key.code);
                }
            }
        }
        if let Some(key_code) = last_key_press {
            if key_code == KeyCode::Char('q') {
                state.game_over = true;
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

    // ゲームオーバーループ
    loop {
        if event::poll(Duration::from_millis(50))? {
           if let Event::Key(key) = event::read()? {
               if key.code == KeyCode::Char('q') { break; }
           }
       }
   }

    execute!(stdout, PopKeyboardEnhancementFlags)?;
    execute!(stdout, Show, LeaveAlternateScreen, ResetColor)?;
    terminal::disable_raw_mode()
}

const SHAPES: [[[(i8, i8); 4]; 4]; 7] = [
    [[(1, 0), (1, 1), (1, 2), (1, 3)], [(0, 2), (1, 2), (2, 2), (3, 2)], [(2, 0), (2, 1), (2, 2), (2, 3)], [(0, 1), (1, 1), (2, 1), (3, 1)]],
    [[(1, 1), (2, 1), (1, 2), (2, 2)], [(1, 1), (2, 1), (1, 2), (2, 2)], [(1, 1), (2, 1), (1, 2), (2, 2)], [(1, 1), (2, 1), (1, 2), (2, 2)]],
    [[(1, 0), (0, 1), (1, 1), (2, 1)], [(1, 0), (1, 1), (2, 1), (1, 2)], [(0, 1), (1, 1), (2, 1), (1, 2)], [(1, 0), (0, 1), (1, 1), (1, 2)]],
    [[(2, 0), (0, 1), (1, 1), (2, 1)], [(1, 0), (1, 1), (1, 2), (2, 2)], [(0, 1), (1, 1), (2, 1), (0, 2)], [(0, 0), (1, 0), (1, 1), (1, 2)]],
    [[(0, 0), (0, 1), (1, 1), (2, 1)], [(1, 0), (2, 0), (1, 1), (1, 2)], [(0, 1), (1, 1), (2, 1), (2, 2)], [(1, 0), (1, 1), (0, 2), (1, 2)]],
    [[(1, 0), (2, 0), (0, 1), (1, 1)], [(1, 0), (1, 1), (2, 1), (2, 2)], [(1, 1), (2, 1), (0, 2), (1, 2)], [(0, 0), (0, 1), (1, 1), (1, 2)]],
    [[(0, 0), (1, 0), (1, 1), (2, 1)], [(2, 0), (1, 1), (2, 1), (1, 2)], [(0, 1), (1, 1), (1, 2), (2, 2)], [(1, 0), (0, 1), (1, 1), (0, 2)]],
];
