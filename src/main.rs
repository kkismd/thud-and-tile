use crossterm::{
    cursor::{Hide, Show},
    event::{
        self, Event, KeyCode, KeyEventKind, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    style::{Color, ResetColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self};
use std::thread;
use std::time::{Duration, Instant};

mod config;
use config::*;

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
use scoring::CustomScoreSystem;

mod tetromino;
use tetromino::Tetromino;

mod board_logic;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameMode {
    Title,
    Playing,
    GameOver,
}

use crate::render::Animation; // Use Animation from render module

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
        if let Some(piece) = &self.current_piece
            && !self.is_valid_position(piece)
        {
            self.mode = GameMode::GameOver;
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
                                if !visited[ny_usize][nx_usize]
                                    && let Some(neighbor_color) =
                                        match self.board[ny_usize][nx_usize] {
                                            Cell::Occupied(c) => Some(c),
                                            Cell::Connected { color: c, count: _ } => Some(c),
                                            _ => None,
                                        }
                                    && neighbor_color == color
                                {
                                    visited[ny_usize][nx_usize] = true;
                                    queue.push_back((nx_usize, ny_usize));
                                    component.push((nx_usize, ny_usize));
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

            // Count colors before clearing lines for custom scoring
            // Use new scoring formula: block_count × MAX-CHAIN × 10 points
            for &line_y in &sorted_lines {
                for x in 0..BOARD_WIDTH {
                    match self.board[line_y][x] {
                        Cell::Occupied(color) => {
                            // Occupied blocks have count=1
                            let points = self.custom_score_system.max_chains.get(color) * 10;
                            self.custom_score_system.scores.add(color, points);
                        }
                        Cell::Connected { color, count } => {
                            // Connected blocks use their actual count value
                            let points = (count as u32)
                                * self.custom_score_system.max_chains.get(color)
                                * 10;
                            self.custom_score_system.scores.add(color, points);
                        }
                        _ => {} // Empty cells and other types are ignored
                    }
                }
            }

            for &line_y in &sorted_lines {
                self.board.remove(line_y);
            }
            for _ in 0..num_cleared {
                self.board.insert(0, vec![Cell::Empty; BOARD_WIDTH]);
            }
            // Note: Traditional score tracking removed in favor of custom color-based scoring

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
            board_logic::remove_isolated_blocks(self, y);

            // 2. Count colors before clearing lines for custom scoring
            // Use new scoring formula: block_count × MAX-CHAIN × 10 points
            for x in 0..BOARD_WIDTH {
                match self.board[y][x] {
                    Cell::Occupied(color) => {
                        // Occupied blocks have count=1
                        let points = self.custom_score_system.max_chains.get(color) * 10;
                        self.custom_score_system.scores.add(color, points);
                    }
                    Cell::Connected { color, count } => {
                        // Connected blocks use their actual count value
                        let points = (count as u32)
                            * self.custom_score_system.max_chains.get(color)
                            * 10;
                        self.custom_score_system.scores.add(color, points);
                    }
                    _ => {} // Empty cells and other types are ignored
                }
            }

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
            }
            _ => return,
        }

        if self.is_valid_position(&piece) {
            self.current_piece = Some(piece);
        }
    }
}

fn handle_line_blink_animation(
    state: &mut GameState,
    time_provider: &dyn TimeProvider,
    anim: Animation,
) -> Vec<Animation> {
    let mut still_animating = Vec::new();
    if let Animation::LineBlink {
        lines,
        count: _,
        start_time,
    } = anim
    {
        let now = time_provider.now();
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
    still_animating
}

fn handle_push_down_animation(
    state: &mut GameState,
    time_provider: &dyn TimeProvider,
    anim: Animation,
) -> (Vec<Animation>, bool) {
    let mut still_animating = Vec::new();
    let mut finished = false;
    if let Animation::PushDown {
        gray_line_y,
        start_time,
    } = anim
    {
        let now = time_provider.now();
        if now - start_time >= PUSH_DOWN_STEP_DURATION {
            let target_y = gray_line_y + 1;

            // Check if the animation should finish
            if target_y >= state.current_board_height || state.board[target_y][0] == Cell::Solid {
                // Finalize the line as Solid
                for x in 0..BOARD_WIDTH {
                    state.board[gray_line_y][x] = Cell::Solid;
                }
                state.current_board_height = state.current_board_height.saturating_sub(1);
                // Note: Scoring now handled immediately when lines are cleared, not after animation

                // Update connected block counts after animation completion (Bug fix)
                // This ensures that after line clear animations complete, the connected blocks
                // are properly recounted to reflect any changes in connectivity
                // Use full board update since animation affects the entire board structure
                state.update_all_connected_block_counts();
                finished = true;
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
    (still_animating, finished)
}

fn handle_animation(state: &mut GameState, time_provider: &dyn TimeProvider) {
    if state.animation.is_empty() {
        return;
    }

    let mut still_animating_this_cycle = Vec::new();
    let mut animations_finished_this_cycle = false;

    // Take ownership to process
    for anim in std::mem::take(&mut state.animation) {
        match anim {
            Animation::LineBlink { .. } => {
                still_animating_this_cycle.extend(handle_line_blink_animation(
                    state,
                    time_provider,
                    anim,
                ));
            }
            Animation::PushDown { .. } => {
                let (remaining_animations, finished) =
                    handle_push_down_animation(state, time_provider, anim);
                still_animating_this_cycle.extend(remaining_animations);
                animations_finished_this_cycle = finished;
            }
        }
    }

    state.animation = still_animating_this_cycle;

    // Spawn a new piece only if all animations have completed in this cycle.
    if animations_finished_this_cycle && state.animation.is_empty() {
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
                        render::draw_title_screen(&mut renderer)?;
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
