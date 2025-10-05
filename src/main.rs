use crossterm::{
    cursor::{Hide, Show},
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    style::ResetColor,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self};
use std::time::{Duration, Instant};

mod animation;
mod config;
mod game_color;
mod game_input;
mod random;
mod scheduler; // 共通アニメーション処理モジュール
use config::*;
use game_color::GameColor;
use game_input::{CrosstermInputProvider, GameInput, InputProvider};
use scheduler::{create_default_scheduler, Scheduler};

mod render;

// --- 時間管理 ---
pub trait TimeProvider {
    fn now(&self) -> Duration;
}

pub struct SystemTimeProvider {
    start: Instant,
}

impl SystemTimeProvider {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
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

mod cell;
use cell::{Board, Cell};

mod scoring;
use scoring::{CustomScoreSystem, calculate_line_clear_total_score, calculate_chain_increases};

mod tetromino;
use tetromino::Tetromino;

mod board_logic;

use animation::{
    update_animations, Animation, PushDownStepResult, process_push_down_step,
    count_solid_lines_from_bottom, determine_erase_line_count
}; // 共通アニメーション関数

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameMode {
    Title,
    Playing,
    GameOver,
}

#[derive(Clone, Debug, PartialEq)]
struct GameState {
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

        // CHAIN-BONUS自動更新: MAX-CHAIN更新前の値を保存
        let old_max_chains = self.custom_score_system.max_chains.clone();

        // Update MAX-CHAIN based on current connected block counts
        self.update_max_chains();
        
        // CHAIN-BONUS自動更新: 増加分を計算してCHAIN-BONUSに加算
        let total_increase = calculate_chain_increases(&old_max_chains, &self.custom_score_system.max_chains);
        self.custom_score_system.max_chains.add_chain_bonus(total_increase);

        // Calculate scores for lines to be cleared (before clearing)
        for &line_y in &lines_to_clear {
            // OLD SYSTEM: Keep existing scores system
            let scores = animation::calculate_line_clear_score(
                &self.board,
                line_y,
                &self.custom_score_system.max_chains,
            );
            for (color, points) in scores {
                self.custom_score_system.scores.add(color, points);
            }
            
            // NEW SYSTEM: Add total_score calculation in parallel
            let total_score_points = calculate_line_clear_total_score(
                &self.board,
                line_y,
                &self.custom_score_system.max_chains,
            );
            self.custom_score_system.add_total_score(total_score_points);
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
            // OLD SYSTEM: Keep existing scores system
            let scores = animation::calculate_line_clear_score(
                &self.board,
                line_y,
                &self.custom_score_system.max_chains,
            );
            for (color, points) in scores {
                self.custom_score_system.scores.add(color, points);
            }
            
            // NEW SYSTEM: Add total_score calculation in parallel
            let total_score_points = calculate_line_clear_total_score(
                &self.board,
                line_y,
                &self.custom_score_system.max_chains,
            );
            self.custom_score_system.add_total_score(total_score_points);
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
        let (bottom_lines_cleared, non_bottom_lines_cleared) = completed_lines
            .iter()
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
        let non_bottom_lines_cleared: Vec<usize> = completed_lines
            .iter()
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
        match process_push_down_step(
            &mut state.board,
            &mut state.current_board_height,
            gray_line_y,
        ) {
            PushDownStepResult::Completed => {
                // Push down completed - update connected blocks as board structure changed
                state.update_all_connected_block_counts();

                // PushDown完了時：EraseLineアニメーション判定
                let solid_count = count_solid_lines_from_bottom(&state.board);
                let chain_bonus = state.custom_score_system.max_chains.chain_bonus;
                let erasable_lines = determine_erase_line_count(chain_bonus, solid_count);
                
                if erasable_lines > 0 {
                    let board_height = state.board.len();
                    let target_lines: Vec<usize> = (0..erasable_lines)
                        .map(|i| board_height - 1 - i)
                        .collect();
                    
                    state.animation.push(Animation::EraseLine {
                        target_solid_lines: target_lines,
                        current_step: 0,
                        last_update: current_time,
                        chain_bonus_consumed: 0,
                    });
                }

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

fn main() -> io::Result<()> {
    let mut renderer = render::CrosstermRenderer::new();
    execute!(renderer.stdout, EnterAlternateScreen, Hide)?;
    execute!(
        renderer.stdout,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;
    terminal::enable_raw_mode()?;

    let time_provider = SystemTimeProvider::new();
    let mut input_provider = CrosstermInputProvider::new();
    let scheduler = create_default_scheduler();
    let mut state = GameState::new();
    let mut prev_state = state.clone();
    let mut last_fall = time_provider.now();

    render::draw_title_screen(&mut renderer)?;

    loop {
        if state.mode != GameMode::Title {
            render::draw(&mut renderer, &prev_state, &state)?;
        }
        prev_state = state.clone();

        match state.mode {
            GameMode::Title => {
                if input_provider.poll_input(100)? {
                    if let Some(input) = input_provider.read_input()? {
                        match input {
                            GameInput::Restart => {
                                state = GameState::new();
                                state.mode = GameMode::Playing;
                                state.spawn_piece();
                            }
                            GameInput::Quit => break,
                            _ => {}
                        }
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
                let inputs = input_provider.read_all_pending()?;
                for input in inputs {
                    match input {
                        GameInput::Quit => {
                            state.mode = GameMode::GameOver;
                            break;
                        }
                        _ => state.handle_input(input),
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
                scheduler.wait_for_next_frame();
            }
            GameMode::GameOver => {
                if input_provider.poll_input(50)? {
                    if let Some(input) = input_provider.read_input()? {
                        match input {
                            GameInput::Quit => break,
                            GameInput::Restart => {
                                state = GameState::new();
                                render::draw_title_screen(&mut renderer)?;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    execute!(renderer.stdout, PopKeyboardEnhancementFlags)?;
    execute!(renderer.stdout, Show, LeaveAlternateScreen, ResetColor)?;
    terminal::disable_raw_mode()
}

#[cfg(test)]
mod tests;
