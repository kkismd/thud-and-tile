use crossterm::{
    cursor::{Hide, Show},
    event::{
        KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    style::{ResetColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self};
use std::time::{Duration, Instant};

mod config;
mod game_color;
mod game_input;
mod random;
mod scheduler;
mod animation; // 共通アニメーション処理モジュール
mod game_core; // 共通ゲームコア
mod unified_scheduler; // 統一タイマー管理
mod unified_engine; // 統一ゲームエンジン
mod test_time_provider; // テスト用TimeProvider
mod cli_game_engine; // CLI版ゲームエンジン
use config::*;
use game_color::GameColor;
use game_input::{GameInput, InputProvider, CrosstermInputProvider};
use scheduler::{Scheduler, create_default_scheduler};
use unified_scheduler::{TimeProvider, NativeTimeProvider}; // 統一TimeProvider使用
use unified_engine::UnifiedGameController;
use cli_game_engine::CliGameEngine;

// テスト用のMockTimeProvider互換性エイリアス  
#[cfg(test)]
pub use test_time_provider::ControllableTimeProvider as MockTimeProvider;
pub struct TestTimeProvider {
    current_time: Duration,
}

#[cfg(test)]
impl TestTimeProvider {
    pub fn new() -> Self {
        Self {
            current_time: Duration::ZERO,
        }
    }
    
    pub fn advance(&mut self, duration: Duration) {
        self.current_time += duration;
    }
}

#[cfg(test)]
impl TimeProvider for TestTimeProvider {
    fn now(&self) -> Duration {
        self.current_time
    }
}

mod render;

// --- データ構造 ---

mod cell;
use cell::{Board, Cell};

mod scoring;
use scoring::CustomScoreSystem;

mod tetromino;
use tetromino::Tetromino;

mod board_logic;

use animation::{Animation, update_animations, process_push_down_step, PushDownStepResult}; // 共通アニメーション関数

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameMode { // test_adapter用にpublicにする
    Title,
    Playing,
    GameOver,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GameState { // test_adapter用にpublicにする
    mode: GameMode,
    board: Board,
    current_piece: Option<Tetromino>,
    next_piece: Option<Tetromino>,
    animation: Vec<Animation>,
    lines_cleared: u32,
    fall_speed: Duration,
    current_board_height: usize,
    custom_score_system: CustomScoreSystem,
}

impl GameState {
    fn new() -> Self {
        Self {
            mode: GameMode::Title,
            board: vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: None,
            next_piece: Some(Tetromino::new_random()), // next_pieceを初期化
            animation: Vec::new(),
            lines_cleared: 0,
            fall_speed: FALL_SPEED_START,
            current_board_height: BOARD_HEIGHT,
            custom_score_system: CustomScoreSystem::new(),
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
        if let Some(piece) = &self.current_piece {
            if !self.is_valid_position(piece) {
                self.mode = GameMode::GameOver;
            }
        }
    }

    fn is_valid_position(&self, piece: &Tetromino) -> bool {
        for ((x, y), _) in piece.iter_blocks() {
            // Check horizontal boundaries
            if x < 0 || x >= BOARD_WIDTH as i8 {
                return false;
            }
            // Check bottom boundary
            if y >= self.current_board_height as i8 {
                return false;
            }
            // Check collision with existing blocks, but only for visible part of the board (y >= 0)
            if y >= 0 && self.board[y as usize][x as usize] != Cell::Empty {
                return false;
            }
            // Allow blocks to be at y < 0 (above the visible board) without being invalid
            // as long as they don't collide with existing blocks (which are only at y >= 0)
        }
        true
    }

    fn lock_piece(&mut self, time_provider: &dyn TimeProvider) {
        if let Some(piece) = self.current_piece.take() {
            for ((x, y), color) in piece.iter_blocks() {
                if y >= 0 && y < BOARD_HEIGHT as i8 {
                    // ここを修正
                    self.board[y as usize][x as usize] = Cell::Occupied(color);
                }
            }
        }

        // ここから隣接色スキャンロジックを追加
        // find_and_connect_adjacent_blocks の前に lines_to_clear を計算
        let mut lines_to_clear: Vec<usize> = self.board[0..self.current_board_height]
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                row.iter().all(|&cell| {
                    matches!(cell, Cell::Occupied(_))
                        || matches!(cell, Cell::Connected { color: _, count: _ })
                })
            })
            .map(|(y, _)| y)
            .collect();
        lines_to_clear.sort_by(|a, b| b.cmp(a)); // Sort in descending order to clear from bottom up

        board_logic::find_and_connect_adjacent_blocks(&mut self.board, &lines_to_clear); // lines_to_clear を渡す

        self.update_connected_block_counts();

        // Update MAX-CHAIN based on current connected block counts
        self.update_max_chains();

        // Calculate scores for lines to be cleared (before clearing)
        for &line_y in &lines_to_clear {
            let scores = animation::calculate_line_clear_score(&self.board, line_y, &self.custom_score_system.max_chains);
            for (color, points) in scores {
                self.custom_score_system.scores.add(color, points);
            }
        }

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

    fn update_connected_block_counts(&mut self) {
        let connected_counts = board_logic::count_connected_blocks(&self.board, 0);
        for ((x, y), count) in connected_counts {
            if let Cell::Connected { color, count: _ } = self.board[y][x] {
                self.board[y][x] = Cell::Connected {
                    color,
                    count: count as u8,
                };
            }
        }
    }

    fn update_max_chains(&mut self) {
        // Scan the entire board to find the maximum connected block count for each color
        for y in 0..self.current_board_height {
            for x in 0..BOARD_WIDTH {
                if let Cell::Connected { color, count } = self.board[y][x] {
                    self.custom_score_system
                        .max_chains
                        .update_max(color, count as u32);
                }
            }
        }
    }

    fn update_all_connected_block_counts(&mut self) {
        // For full board update, we need to check all rows from 0 to current_board_height
        // Use current_board_height as the limit and start from 0
        let mut results = Vec::new();
        let mut visited = vec![vec![false; BOARD_WIDTH]; BOARD_HEIGHT];

        for y in 0..self.current_board_height {
            for x in 0..BOARD_WIDTH {
                if let Some(color) = match self.board[y][x] {
                    Cell::Occupied(c) => Some(c),
                    Cell::Connected { color: c, count: _ } => Some(c),
                    _ => None,
                } {
                    if visited[y][x] {
                        continue;
                    }

                    let mut component = Vec::new();
                    let mut queue = std::collections::VecDeque::new();

                    visited[y][x] = true;
                    queue.push_back((x, y));
                    component.push((x, y));

                    while let Some((qx, qy)) = queue.pop_front() {
                        let neighbors = [
                            (qx as i8 - 1, qy as i8),
                            (qx as i8 + 1, qy as i8),
                            (qx as i8, qy as i8 - 1),
                            (qx as i8, qy as i8 + 1),
                        ];

                        for (nx, ny) in neighbors {
                            if nx >= 0
                                && nx < BOARD_WIDTH as i8
                                && ny >= 0
                                && ny < self.current_board_height as i8
                            {
                                let (nx_usize, ny_usize) = (nx as usize, ny as usize);
                                if !visited[ny_usize][nx_usize] {
                                    let neighbor_color = match self.board[ny_usize][nx_usize] {
                                        Cell::Occupied(c) => Some(c),
                                        Cell::Connected { color: c, count: _ } => Some(c),
                                        _ => None,
                                    };
                                    if let Some(neighbor_color) = neighbor_color {
                                        if neighbor_color == color {
                                            visited[ny_usize][nx_usize] = true;
                                            queue.push_back((nx_usize, ny_usize));
                                            component.push((nx_usize, ny_usize));
                                        }
                                    }
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

        // Update the board with the new counts
        for ((x, y), count) in results {
            if let Cell::Connected { color, count: _ } = self.board[y][x] {
                self.board[y][x] = Cell::Connected {
                    color,
                    count: count as u8,
                };
            }
        }
    }

    fn clear_lines(&mut self, lines: &[usize], time_provider: &dyn TimeProvider) -> Vec<Animation> {
        let mut new_animations = Vec::new();
        let mut bottom_lines_cleared = Vec::new();
        let mut non_bottom_lines_cleared = Vec::new();

        // Calculate scores for lines to be cleared (before clearing)
        for &line_y in lines {
            let scores = animation::calculate_line_clear_score(&self.board, line_y, &self.custom_score_system.max_chains);
            for (color, points) in scores {
                self.custom_score_system.scores.add(color, points);
            }
        }

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

            // Update connected block counts after bottom line clear
            // This ensures connected blocks are properly recounted even for standard clears
            // Use full board update since bottom line clear affects the entire board structure
            self.update_all_connected_block_counts();

            // No animation for standard clear, just spawn new piece
            self.spawn_piece();
        }

        // Handle custom clear for non-bottom lines
        for &y in &non_bottom_lines_cleared {
            // 1. Remove isolated blocks below the cleared line.
            board_logic::remove_isolated_blocks(&mut self.board, y);

            // 2. Turn the cleared line to gray (Step 5)
            for x in 0..BOARD_WIDTH {
                self.board[y][x] = Cell::Occupied(GameColor::Grey);
            }

            // Trigger the push-down animation (Step 6).
            new_animations.push(Animation::PushDown {
                gray_line_y: y,
                start_time: time_provider.now(),
            });
        }

        // Update connected blocks after any line clears (both bottom and non-bottom)
        if !bottom_lines_cleared.is_empty() || !non_bottom_lines_cleared.is_empty() {
            self.update_all_connected_block_counts();
        }

        // If no non-bottom lines were cleared, and no bottom lines were cleared, spawn a new piece
        if bottom_lines_cleared.is_empty() && non_bottom_lines_cleared.is_empty() {
            self.spawn_piece();
        }
        new_animations
    }

    fn handle_input(&mut self, input: GameInput) {
        if self.current_piece.is_none() {
            return;
        }
        let mut piece = self.current_piece.clone().unwrap();

        match input {
            GameInput::MoveLeft => piece = piece.moved(-1, 0),
            GameInput::MoveRight => piece = piece.moved(1, 0),
            GameInput::HardDrop => {
                // Hard Drop: 即座に着地
                while self.is_valid_position(&piece.moved(0, 1)) {
                    piece = piece.moved(0, 1);
                }
            }
            GameInput::RotateClockwise => {
                // Clockwise Rotation
                piece = piece.rotated();
            }
            GameInput::RotateCounterClockwise => {
                // Counter-Clockwise Rotation
                piece = piece.rotated_counter_clockwise();
            }
            GameInput::SoftDrop => {
                // Soft Drop
                piece = piece.moved(0, 1);
            }
            _ => return, // その他の入力は無視
        }

        if self.is_valid_position(&piece) {
            self.current_piece = Some(piece);
        }
    }
}

fn handle_animation(state: &mut GameState, time_provider: &dyn TimeProvider) {
    if state.animation.is_empty() {
        return;
    }

    // Use the common animation update logic from animation.rs
    let current_time = time_provider.now();
    let result = update_animations(&mut state.animation, current_time);

    // Handle completed line clears
    for completed_lines in result.completed_line_blinks.clone() {
        // Process line clear using shared logic
        let (bottom_lines_cleared, non_bottom_lines_cleared) = completed_lines.iter()
            .partition::<Vec<_>, _>(|&&line_y| line_y == state.current_board_height - 1);

        let has_bottom_clears = !bottom_lines_cleared.is_empty();
        let has_non_bottom_clears = !non_bottom_lines_cleared.is_empty();

        // Handle bottom lines (standard Tetris clear)
        if has_bottom_clears {
            let num_cleared = bottom_lines_cleared.len();
            let mut sorted_lines: Vec<usize> = bottom_lines_cleared.into_iter().cloned().collect();
            sorted_lines.sort_by(|a, b| b.cmp(a));

            for &line_y in &sorted_lines {
                state.board.remove(line_y);
            }
            for _ in 0..num_cleared {
                state.board.insert(0, vec![Cell::Empty; BOARD_WIDTH]);
            }

            // Update connected block counts after bottom line clear
            state.update_all_connected_block_counts();
            state.spawn_piece();
        }

        // Handle non-bottom lines (custom clear with gray conversion)
        for &&y in &non_bottom_lines_cleared {
            // Remove isolated blocks
            board_logic::remove_isolated_blocks(&mut state.board, y);

            // Turn line to gray
            for x in 0..BOARD_WIDTH {
                state.board[y][x] = Cell::Occupied(GameColor::Grey);
            }
        }

        // Update connected blocks after any line clears
        if has_bottom_clears || has_non_bottom_clears {
            state.update_all_connected_block_counts();
        }
    }

    // Set continuing animations first
    state.animation = result.continuing_animations;

    // Add push down animations for non-bottom lines
    for completed_lines in result.completed_line_blinks {
        let non_bottom_lines_cleared: Vec<usize> = completed_lines.iter()
            .filter(|&&line_y| line_y != state.current_board_height - 1)
            .cloned()
            .collect();

        for y in non_bottom_lines_cleared {
            // Trigger push-down animation
            state.animation.push(Animation::PushDown {
                gray_line_y: y,
                start_time: current_time,
            });
        }
    }

    // Handle completed push downs
    for gray_line_y in result.completed_push_downs {
        // Process push down step
        match process_push_down_step(&mut state.board, &mut state.current_board_height, gray_line_y) {
            PushDownStepResult::Completed => {
                // Push down completed - update connected blocks as board structure changed
                state.update_all_connected_block_counts();
                
                // Potentially spawn new piece
                if state.animation.is_empty() {
                    state.spawn_piece();
                }
            }
            PushDownStepResult::Moved { new_gray_line_y } => {
                // Board structure changed - update connected blocks
                state.update_all_connected_block_counts();
                
                // Continue push down animation at new position
                state.animation.push(Animation::PushDown {
                    gray_line_y: new_gray_line_y,
                    start_time: current_time,
                });
            }
        }
    }

    // If all animations completed, spawn new piece
    if state.animation.is_empty() {
        state.spawn_piece();
    }
}

/// 統一アーキテクチャベースのメイン関数（将来的にはこちらがメインに）
fn main_unified() -> io::Result<()> {
    let mut renderer = render::CrosstermRenderer::new();
    execute!(
        renderer.stdout,
        EnterAlternateScreen,
        Hide,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;
    terminal::enable_raw_mode()?;

    // 統一アーキテクチャでゲームを初期化
    let time_provider = Box::new(NativeTimeProvider::new());
    let engine = Box::new(CliGameEngine::new());
    let mut controller = UnifiedGameController::new(engine, time_provider);
    
    let mut input_provider = CrosstermInputProvider::new();
    
    render::draw_title_screen(&mut renderer)?;

    loop {
        // イベント駆動更新
        let update_result = controller.update();
        
        // 入力処理（ノンブロッキング）
        let inputs = input_provider.read_all_pending()?;
        for input in inputs {
            if input == GameInput::Quit {
                break;  // メインループから脱出
            }
            controller.handle_input(input);
        }
        
        // 描画処理
        if update_result.needs_render {
            match update_result.game_mode {
                0 => render::draw_title_screen(&mut renderer)?, // Title
                1 => {
                    // Playing - 現在の実装では詳細な描画は後で実装
                    render::draw_title_screen(&mut renderer)?;
                }
                2 => {
                    // GameOver - 現在の実装では詳細な描画は後で実装
                    render::draw_title_screen(&mut renderer)?;
                }
                _ => {}
            }
            controller.render_complete();
        }
        
        // CPU負荷軽減のための最小スリープ
        std::thread::sleep(Duration::from_millis(16)); // ~60fps
    }
    
    execute!(renderer.stdout, PopKeyboardEnhancementFlags)?;
    execute!(renderer.stdout, Show, LeaveAlternateScreen, ResetColor)?;
    terminal::disable_raw_mode()
}

fn main() -> io::Result<()> {
    // 統一アーキテクチャのテスト
    main_unified()
}

#[cfg(test)]
mod tests;
