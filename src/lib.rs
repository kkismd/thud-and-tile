//! Thud & Tile WASM Library
//! 
//! このモジュールは、Thud & TileゲームのWASM環境用エントリーポイントを提供します。
//! JavaScript環境からアクセス可能なAPIを実装し、ゲームロジックとUI間の橋渡しを行います。

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use std::time::Duration;

// JavaScript console.log への出力用マクロ
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    // JavaScript Date.now()へのアクセス
    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn js_date_now() -> f64;
}

#[cfg(target_arch = "wasm32")]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! console_log {
    ($($t:tt)*) => {
        println!($($t)*);
    }
}

// --- 時間管理（WASM対応） ---
pub trait TimeProvider {
    fn now(&self) -> Duration;
}

#[cfg(target_arch = "wasm32")]
pub struct WasmTimeProvider {
    start_time: f64,
}

#[cfg(target_arch = "wasm32")]
impl WasmTimeProvider {
    pub fn new() -> Self {
        Self {
            start_time: js_date_now(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for WasmTimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "wasm32")]
impl TimeProvider for WasmTimeProvider {
    fn now(&self) -> Duration {
        let current_time = js_date_now();
        let elapsed_ms = current_time - self.start_time;
        Duration::from_millis(elapsed_ms as u64)
    }
}

// テスト用モック（WASM環境でも使用可能）
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

// モジュールのインポート
mod config;
mod game_color;
mod game_input;
mod random;
mod scheduler;
mod cell;
mod board_logic;
mod tetromino;
mod scoring;

use config::*;
use game_color::GameColor;
use game_input::GameInput;
use random::{RandomProvider, create_default_random_provider};
use cell::Cell;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Web版用のTetrominoShape列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TetrominoShape {
    I = 0,
    O = 1,
    T = 2,
    L = 3,
    J = 4,
    S = 5,
    Z = 6,
}

impl TetrominoShape {
    fn all_shapes() -> Vec<TetrominoShape> {
        vec![
            TetrominoShape::I,
            TetrominoShape::O,
            TetrominoShape::T,
            TetrominoShape::L,
            TetrominoShape::J,
            TetrominoShape::S,
            TetrominoShape::Z,
        ]
    }
}

/// Web版用の7-bag実装
pub struct WebTetrominoBag {
    bag: Vec<TetrominoShape>,
}

impl WebTetrominoBag {
    pub fn new() -> Self {
        let mut bag = TetrominoShape::all_shapes();
        let mut provider = create_default_random_provider();
        provider.shuffle(&mut bag);
        WebTetrominoBag { bag }
    }

    pub fn next(&mut self) -> TetrominoShape {
        if self.bag.is_empty() {
            self.bag = TetrominoShape::all_shapes();
            let mut provider = create_default_random_provider();
            provider.shuffle(&mut self.bag);
        }
        self.bag.pop().unwrap()
    }
}

/// 簡易テトロミノ（WASM用）
#[derive(Debug, Clone)]
pub struct SimpleTetromino {
    pub x: usize,
    pub y: usize,
    pub rotation: u8,
    pub color: GameColor,
    pub shape: u8, // 0=I, 1=O, 2=T, 3=L, 4=J, 5=S, 6=Z
}

impl SimpleTetromino {
    pub fn new_random() -> Self {
        let mut provider = create_default_random_provider();
        let shape = provider.gen_range(0, 7) as u8;
        let color_index = provider.gen_range(0, 4);
        let colors = [GameColor::Red, GameColor::Green, GameColor::Blue, GameColor::Yellow];
        
        SimpleTetromino {
            x: (BOARD_WIDTH / 2 - 2) as usize, // CLI版と同じ位置 (x=3)
            y: 0,
            rotation: 0,
            color: colors[color_index],
            shape,
        }
    }
    
    pub fn from_shape(shape: TetrominoShape) -> Self {
        let mut provider = create_default_random_provider();
        let color_index = provider.gen_range(0, 4);
        let colors = [GameColor::Red, GameColor::Green, GameColor::Blue, GameColor::Yellow];
        
        SimpleTetromino {
            x: (BOARD_WIDTH / 2 - 2) as usize, // CLI版と同じ位置 (x=3)
            y: 0,
            rotation: 0,
            color: colors[color_index],
            shape: shape as u8,
        }
    }
    
    pub fn get_blocks(&self) -> Vec<(i8, i8)> {
        match self.shape {
            0 => vec![(0, 1), (1, 1), (2, 1), (3, 1)], // I piece - SRS standard
            1 => vec![(1, 1), (2, 1), (1, 2), (2, 2)], // O piece - SRS standard
            2 => vec![(1, 0), (0, 1), (1, 1), (2, 1)], // T piece - SRS standard
            3 => vec![(2, 0), (0, 1), (1, 1), (2, 1)], // L piece - SRS standard
            4 => vec![(0, 0), (0, 1), (1, 1), (2, 1)], // J piece - SRS standard
            5 => vec![(1, 0), (2, 0), (0, 1), (1, 1)], // S piece - SRS standard
            6 => vec![(0, 0), (1, 0), (1, 1), (2, 1)], // Z piece - SRS standard
            _ => vec![(0, 0)], // Default
        }
    }
    
    pub fn get_blocks_at_rotation(&self, rotation: u8) -> Vec<(i8, i8)> {
        // SRS標準座標系による4つの回転状態
        match self.shape {
            0 => { // I piece - SRS standard
                match rotation {
                    0 => vec![(0, 1), (1, 1), (2, 1), (3, 1)], // horizontal
                    1 => vec![(2, 3), (2, 2), (2, 1), (2, 0)], // vertical
                    2 => vec![(3, 2), (2, 2), (1, 2), (0, 2)], // horizontal
                    3 => vec![(1, 0), (1, 1), (1, 2), (1, 3)], // vertical
                    _ => vec![(0, 1), (1, 1), (2, 1), (3, 1)],
                }
            },
            1 => { // O piece - no rotation
                vec![(1, 1), (2, 1), (1, 2), (2, 2)]
            },
            2 => { // T piece - SRS standard
                match rotation {
                    0 => vec![(1, 0), (0, 1), (1, 1), (2, 1)], // upward T
                    1 => vec![(2, 1), (1, 0), (1, 1), (1, 2)], // rightward T
                    2 => vec![(1, 2), (2, 1), (1, 1), (0, 1)], // downward T
                    3 => vec![(0, 1), (1, 2), (1, 1), (1, 0)], // leftward T
                    _ => vec![(1, 0), (0, 1), (1, 1), (2, 1)],
                }
            },
            3 => { // L piece - SRS standard
                match rotation {
                    0 => vec![(2, 0), (0, 1), (1, 1), (2, 1)],
                    1 => vec![(2, 2), (1, 0), (1, 1), (1, 2)],
                    2 => vec![(0, 2), (2, 1), (1, 1), (0, 1)],
                    3 => vec![(0, 0), (1, 2), (1, 1), (1, 0)],
                    _ => vec![(2, 0), (0, 1), (1, 1), (2, 1)],
                }
            },
            4 => { // J piece - SRS standard
                match rotation {
                    0 => vec![(0, 0), (0, 1), (1, 1), (2, 1)],
                    1 => vec![(2, 0), (1, 0), (1, 1), (1, 2)],
                    2 => vec![(2, 2), (2, 1), (1, 1), (0, 1)],
                    3 => vec![(0, 2), (1, 2), (1, 1), (1, 0)],
                    _ => vec![(0, 0), (0, 1), (1, 1), (2, 1)],
                }
            },
            5 => { // S piece - SRS standard
                match rotation {
                    0 => vec![(1, 0), (2, 0), (0, 1), (1, 1)],
                    1 => vec![(2, 1), (2, 2), (1, 0), (1, 1)],
                    2 => vec![(1, 2), (0, 2), (2, 1), (1, 1)],
                    3 => vec![(0, 1), (0, 0), (1, 2), (1, 1)],
                    _ => vec![(1, 0), (2, 0), (0, 1), (1, 1)],
                }
            },
            6 => { // Z piece - SRS standard
                match rotation {
                    0 => vec![(0, 0), (1, 0), (1, 1), (2, 1)],
                    1 => vec![(2, 0), (1, 1), (2, 1), (1, 2)],
                    2 => vec![(0, 1), (1, 1), (1, 2), (2, 2)],
                    3 => vec![(1, 0), (0, 1), (1, 1), (0, 2)],
                    _ => vec![(0, 0), (1, 0), (1, 1), (2, 1)],
                }
            },
            _ => vec![(0, 0)], // Default
        }
    }
}

// SRS Standard Wall Kick Offset Tables
/// SRS offset table for J, L, T, S, Z tetrominoes
/// Index corresponds to transition: [0->1, 1->0, 1->2, 2->1, 2->3, 3->2, 3->0, 0->3]
const SRS_JLTSZ_OFFSETS: [[[i8; 2]; 5]; 8] = [
    // 0->1 transition
    [[0, 0], [-1, 0], [-1, 1], [0, -2], [-1, -2]],
    // 1->0 transition
    [[0, 0], [1, 0], [1, -1], [0, 2], [1, 2]],
    // 1->2 transition
    [[0, 0], [1, 0], [1, -1], [0, 2], [1, 2]],
    // 2->1 transition
    [[0, 0], [-1, 0], [-1, 1], [0, -2], [-1, -2]],
    // 2->3 transition
    [[0, 0], [1, 0], [1, 1], [0, -2], [1, -2]],
    // 3->2 transition
    [[0, 0], [-1, 0], [-1, -1], [0, 2], [-1, 2]],
    // 3->0 transition
    [[0, 0], [-1, 0], [-1, -1], [0, 2], [-1, 2]],
    // 0->3 transition
    [[0, 0], [1, 0], [1, 1], [0, -2], [1, -2]],
];

/// SRS offset table for I tetromino
const SRS_I_OFFSETS: [[[i8; 2]; 5]; 8] = [
    // 0->1 transition
    [[0, 0], [-2, 0], [1, 0], [-2, -1], [1, 2]],
    // 1->0 transition
    [[0, 0], [2, 0], [-1, 0], [2, 1], [-1, -2]],
    // 1->2 transition
    [[0, 0], [-1, 0], [2, 0], [-1, 2], [2, -1]],
    // 2->1 transition
    [[0, 0], [1, 0], [-2, 0], [1, -2], [-2, 1]],
    // 2->3 transition
    [[0, 0], [2, 0], [-1, 0], [2, 1], [-1, -2]],
    // 3->2 transition
    [[0, 0], [-2, 0], [1, 0], [-2, -1], [1, 2]],
    // 3->0 transition
    [[0, 0], [1, 0], [-2, 0], [1, -2], [-2, 1]],
    // 0->3 transition
    [[0, 0], [-1, 0], [2, 0], [-1, 2], [2, -1]],
];

/// Convert rotation state transition to offset table index
const fn get_transition_index(from_state: u8, to_state: u8) -> usize {
    match (from_state, to_state) {
        (0, 1) => 0,
        (1, 0) => 1,
        (1, 2) => 2,
        (2, 1) => 3,
        (2, 3) => 4,
        (3, 2) => 5,
        (3, 0) => 6,
        (0, 3) => 7,
        _ => 0, // Default fallback
    }
}

// WASM初期化関数
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("Thud & Tile WASM module initialized");
}

/// ゲーム状態を表すWASMエクスポート用構造体
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmGameState {
    board: Vec<Vec<Cell>>,
    score: u32,
    current_piece: Option<SimpleTetromino>,
    next_piece: Option<SimpleTetromino>,
    game_mode: u8, // 0: Title, 1: Playing, 2: GameOver
    fall_speed: Duration,
    last_fall_time: Duration,
    time_provider: WasmTimeProvider,
    tetromino_bag: WebTetrominoBag, // 7-bag実装
    // アニメーション関連
    clearing_lines: Vec<usize>, // 現在アニメーション中のライン
    animation_start_time: Option<Duration>, // アニメーション開始時刻
    animation_phase: u8, // 0: なし, 1: 点滅中, 2: 落下中
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmGameState {
    /// 新しいゲーム状態を作成
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGameState {
        console_log!("Creating new WasmGameState");
        let time_provider = WasmTimeProvider::new();
        let current_time = time_provider.now();
        
        WasmGameState {
            board: vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            score: 0,
            current_piece: None,
            next_piece: None,
            game_mode: 0, // Title
            fall_speed: FALL_SPEED_START,
            last_fall_time: current_time,
            time_provider,
            tetromino_bag: WebTetrominoBag::new(),
            clearing_lines: Vec::new(),
            animation_start_time: None,
            animation_phase: 0,
        }
    }
    
    /// ゲームを開始
    #[wasm_bindgen]
    pub fn start_game(&mut self) {
        console_log!("Starting new game");
        self.game_mode = 1; // Playing
        self.score = 0;
        self.board = vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        self.fall_speed = FALL_SPEED_START;
        let current_time = self.time_provider.now();
        self.last_fall_time = current_time;
        // アニメーション状態もリセット
        self.clearing_lines.clear();
        self.animation_start_time = None;
        self.animation_phase = 0;
        
        // CLI版と同じピース初期化ロジック
        // 1. 最初にnext_pieceのみを生成（CLI版のnew()と同じ）
        self.current_piece = None;
        self.next_piece = Some(self.new_random_piece());
        // 2. spawn_pieceを呼び出してnext_piece → current_pieceの転送を行う
        self.spawn_piece();
        
        console_log!("Game started: current_piece from initial next_piece, new next_piece generated");
    }
    
    /// 7-bagを使った新しいピース生成
    fn new_random_piece(&mut self) -> SimpleTetromino {
        let shape = self.tetromino_bag.next();
        SimpleTetromino::from_shape(shape)
    }
    
    /// 新しいピースをスポーン（CLI版と同じロジック）
    pub fn spawn_piece(&mut self) {
        console_log!("spawn_piece called");
        
        // next_pieceをcurrent_pieceにする
        self.current_piece = self.next_piece.take();
        // 新しいnext_pieceを生成する（7-bag使用）
        self.next_piece = Some(self.new_random_piece());
        
        // current_pieceが有効な位置にあるかチェック
        if let Some(piece) = &self.current_piece {
            if !self.is_valid_position(piece, piece.x as i8, piece.y as i8, piece.rotation) {
                self.game_mode = 2; // GameOver
                console_log!("Game Over: piece cannot spawn");
            }
        }
        
        console_log!("spawn_piece completed: next → current, new next generated");
    }
    
    /// 現在のスコアを取得
    #[wasm_bindgen]
    pub fn get_score(&self) -> u32 {
        self.score
    }
    
    /// ゲームモードを取得
    #[wasm_bindgen]
    pub fn get_game_mode(&self) -> u8 {
        self.game_mode
    }
    
    /// ボードの状態を取得（JavaScriptで扱いやすい形式）
    #[wasm_bindgen]
    pub fn get_board_state(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for row in &self.board {
            for cell in row {
                match cell {
                    Cell::Empty => result.push(0),
                    Cell::Occupied(color) => result.push(color_to_u8(*color) + 1),
                    Cell::Connected { color, count: _ } => result.push(color_to_u8(*color) + 10),
                    Cell::Solid => result.push(21),
                }
            }
        }
        result
    }
    
    /// 入力を処理
    #[wasm_bindgen]
    pub fn handle_input(&mut self, input_code: u8) -> bool {
        let game_input = match input_code {
            0 => GameInput::MoveLeft,
            1 => GameInput::MoveRight,
            2 => GameInput::SoftDrop,
            3 => GameInput::RotateClockwise,
            4 => GameInput::RotateCounterClockwise,
            5 => GameInput::HardDrop,
            6 => GameInput::Restart,
            7 => GameInput::Quit,
            _ => GameInput::Unknown,
        };
        
        console_log!("Handling input: {:?}", game_input);
        
        match game_input {
            GameInput::Restart => {
                self.start_game();
                true
            }
            GameInput::Quit => {
                self.game_mode = 0; // Title
                true
            }
            GameInput::MoveLeft => {
                if self.game_mode == 1 {
                    self.move_current_piece(-1, 0)
                } else {
                    false
                }
            }
            GameInput::MoveRight => {
                if self.game_mode == 1 {
                    self.move_current_piece(1, 0)
                } else {
                    false
                }
            }
            GameInput::SoftDrop => {
                if self.game_mode == 1 {
                    self.move_current_piece(0, 1)
                } else {
                    false
                }
            }
            GameInput::RotateClockwise => {
                if self.game_mode == 1 {
                    self.rotate_current_piece(true)
                } else {
                    false
                }
            }
            GameInput::RotateCounterClockwise => {
                if self.game_mode == 1 {
                    self.rotate_current_piece(false)
                } else {
                    false
                }
            }
            GameInput::HardDrop => {
                if self.game_mode == 1 {
                    self.hard_drop()
                } else {
                    false
                }
            }
            _ => false
        }
    }
    
    /// 現在のピースを移動
    #[wasm_bindgen]
    pub fn move_current_piece(&mut self, dx: i8, dy: i8) -> bool {
        if let Some(ref piece) = self.current_piece {
            let new_x = piece.x as i8 + dx;
            let new_y = piece.y as i8 + dy;
            
            if self.is_valid_position(piece, new_x, new_y, piece.rotation) {
                if let Some(ref mut piece) = self.current_piece {
                    piece.x = new_x as usize;
                    piece.y = new_y as usize;
                }
                console_log!("Moved piece to ({}, {})", new_x, new_y);
                return true;
            }
        }
        false
    }
    
    /// 現在のピースを回転（SRS準拠）
    #[wasm_bindgen]
    pub fn rotate_current_piece(&mut self, clockwise: bool) -> bool {
        if let Some(ref piece) = self.current_piece {
            let current_rotation = piece.rotation;
            let new_rotation = if clockwise {
                (current_rotation + 1) % 4
            } else {
                (current_rotation + 3) % 4
            };
            
            // SRS wall kick offsets を試行
            let offsets = self.get_srs_wall_kick_offsets(piece.shape, current_rotation, new_rotation);
            
            for &[offset_x, offset_y] in offsets {
                let test_x = piece.x as i8 + offset_x;
                let test_y = piece.y as i8 + offset_y;
                
                if self.is_valid_position(piece, test_x, test_y, new_rotation) {
                    // 回転成功
                    if let Some(ref mut piece) = self.current_piece {
                        piece.rotation = new_rotation;
                        piece.x = test_x as usize;
                        piece.y = test_y as usize;
                    }
                    console_log!("SRS rotation successful: rotation {} at ({}, {})", new_rotation, test_x, test_y);
                    return true;
                }
            }
            
            console_log!("SRS rotation failed: no valid position found");
        }
        false
    }
    
    /// SRS standard wall kick offsetsを取得
    fn get_srs_wall_kick_offsets(&self, shape: u8, from_rotation: u8, to_rotation: u8) -> &'static [[i8; 2]; 5] {
        let index = get_transition_index(from_rotation, to_rotation);
        
        match shape {
            0 => &SRS_I_OFFSETS[index], // I piece
            1 => {
                // O piece doesn't need wall kicks (rotates in place)
                static O_OFFSETS: [[i8; 2]; 5] = [[0, 0], [0, 0], [0, 0], [0, 0], [0, 0]];
                &O_OFFSETS
            }
            _ => &SRS_JLTSZ_OFFSETS[index], // J, L, T, S, Z pieces
        }
    }
    
    /// ハードドロップ
    #[wasm_bindgen]
    pub fn hard_drop(&mut self) -> bool {
        if let Some(ref piece) = self.current_piece {
            let mut drop_distance = 0;
            
            // 最大落下距離を計算
            while self.is_valid_position(piece, piece.x as i8, piece.y as i8 + drop_distance + 1, piece.rotation) {
                drop_distance += 1;
            }
            
            if drop_distance > 0 {
                if let Some(ref mut piece) = self.current_piece {
                    piece.y = (piece.y as i8 + drop_distance) as usize;
                }
                console_log!("Hard dropped piece by {} spaces", drop_distance);
                self.lock_piece();
                return true;
            }
        }
        false
    }
    
    /// ピースを固定
    #[wasm_bindgen]
    pub fn lock_piece(&mut self) {
        if let Some(piece) = self.current_piece.take() {
            // ピースをボードに配置（CLI版と同様の処理）
            let blocks = piece.get_blocks_at_rotation(piece.rotation);
            for (dx, dy) in blocks {
                let board_x = piece.x as i8 + dx;
                let board_y = piece.y as i8 + dy;
                
                if board_x >= 0 && board_x < BOARD_WIDTH as i8 && 
                   board_y >= 0 && board_y < BOARD_HEIGHT as i8 {
                    // CLI版と同じようにOccupied状態で配置
                    self.board[board_y as usize][board_x as usize] = Cell::Occupied(piece.color);
                }
            }
            
            console_log!("Piece locked at position ({}, {})", piece.x, piece.y);
            
            // TODO: 隣接ブロック処理（Phase 2Cで実装予定）
            // board_logic::find_and_connect_adjacent_blocks(&mut self.board, &lines_to_clear);
            // self.update_connected_block_counts();
            // self.update_max_chains();
            
            // ライン消去チェック（アニメーション対応）
            self.clear_lines();
            
            // アニメーション中でなければ新しいピースをスポーン
            if self.animation_phase == 0 {
                self.spawn_piece();
            }
            
            console_log!("Piece locked, next piece spawned or animation started");
        }
    }
    
    /// ラインクリア処理（アニメーション対応）
    #[wasm_bindgen]
    pub fn clear_lines(&mut self) {
        // すでにアニメーション中の場合は何もしない
        if self.animation_phase != 0 {
            return;
        }
        
        let mut lines_to_clear = Vec::new();
        
        // 完成したラインを見つける
        for y in 0..BOARD_HEIGHT {
            let mut line_complete = true;
            for x in 0..BOARD_WIDTH {
                if matches!(self.board[y][x], Cell::Empty) {
                    line_complete = false;
                    break;
                }
            }
            if line_complete {
                lines_to_clear.push(y);
            }
        }
        
        if !lines_to_clear.is_empty() {
            // アニメーションを開始
            self.clearing_lines = lines_to_clear;
            self.animation_start_time = Some(self.time_provider.now());
            self.animation_phase = 1; // 点滅フェーズ開始
            console_log!("Starting line clear animation for {} lines", self.clearing_lines.len());
        }
    }
    
    /// 現在のピース情報を取得（JavaScript用）
    #[wasm_bindgen]
    pub fn get_current_piece_info(&self) -> Vec<i32> {
        if let Some(ref piece) = self.current_piece {
            vec![piece.x as i32, piece.y as i32, piece.rotation as i32, piece.shape as i32]
        } else {
            vec![]
        }
    }
    
    /// 位置が有効かチェック
    fn is_valid_position(&self, piece: &SimpleTetromino, x: i8, y: i8, rotation: u8) -> bool {
        let blocks = piece.get_blocks_at_rotation(rotation);
        
        for (dx, dy) in blocks {
            let board_x = x + dx;
            let board_y = y + dy;
            
            // 水平境界チェック
            if board_x < 0 || board_x >= BOARD_WIDTH as i8 {
                return false;
            }
            
            // 下方境界チェック
            if board_y >= BOARD_HEIGHT as i8 {
                return false;
            }
            
            // 衝突チェック（ボード内の可視領域のみ）
            if board_y >= 0 && !matches!(self.board[board_y as usize][board_x as usize], Cell::Empty) {
                return false;
            }
            
            // y < 0 (ボード上部の見えない領域) は許可（スポーン時の回転用）
        }
        
        true
    }

    /// ゴーストピースの位置（現在のテトロミノが真下に落下する最終位置）を計算
    fn calculate_ghost_position(&self, piece: &SimpleTetromino) -> Option<(i8, i8)> {
        let mut ghost_y = piece.y as i8;
        
        // 現在位置から下方向に1つずつ降りて、有効でない位置の直前を見つける
        loop {
            if !self.is_valid_position(piece, piece.x as i8, ghost_y + 1, piece.rotation) {
                break;
            }
            ghost_y += 1;
        }
        
        // 現在位置と同じ場合はゴーストピースを表示しない
        if ghost_y == piece.y as i8 {
            None
        } else {
            Some((piece.x as i8, ghost_y))
        }
    }
    
    /// 自動落下処理 - JavaScriptから定期的に呼び出される
    #[wasm_bindgen]
    pub fn auto_fall(&mut self) -> bool {
        if self.game_mode != 1 { // Playingモードでない場合はスキップ
            return false;
        }
        
        // アニメーション処理を実行
        self.update_animation();
        
        // アニメーション中は新しいピースの処理を停止
        if self.animation_phase != 0 {
            return true;
        }
        
        let current_time = self.time_provider.now();
        
        // 落下時間チェック
        if current_time - self.last_fall_time >= self.fall_speed {
            if let Some(ref piece) = self.current_piece {
                let new_y = piece.y as i8 + 1;
                
                // 下に移動可能かチェック
                if self.is_valid_position(piece, piece.x as i8, new_y, piece.rotation) {
                    // 移動実行
                    if let Some(ref mut piece) = self.current_piece {
                        piece.y = new_y as usize;
                    }
                    console_log!("Auto-fall: piece moved down to y={}", new_y);
                } else {
                    // 移動不可 - ピースをロック
                    self.lock_piece();
                    console_log!("Auto-fall: piece locked, spawning new piece");
                }
            } else {
                // 現在のピースがない場合は新しいピースを生成
                self.spawn_piece();
            }
            
            self.last_fall_time = current_time;
            return true;
        }
        
        false
    }
    
    /// 自動落下速度を取得（ミリ秒）
    #[wasm_bindgen]
    pub fn get_fall_speed_ms(&self) -> u32 {
        self.fall_speed.as_millis() as u32
    }
    
    /// 自動落下速度を設定（ミリ秒）
    #[wasm_bindgen]
    pub fn set_fall_speed_ms(&mut self, ms: u32) {
        self.fall_speed = Duration::from_millis(ms as u64);
        console_log!("Fall speed set to {}ms", ms);
    }
}

/// GameColorをu8に変換するヘルパー関数
fn color_to_u8(color: GameColor) -> u8 {
    match color {
        GameColor::Red => 1,
        GameColor::Green => 2,
        GameColor::Blue => 3,
        GameColor::Yellow => 4,
        GameColor::Magenta => 5,
        GameColor::Cyan => 6,
        GameColor::White => 7,
        GameColor::Black => 8,
        GameColor::DarkGrey => 9,
        _ => 0, // その他の色
    }
}

/// バージョン情報を返す
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_version() -> String {
    "Thud & Tile WASM v0.1.0".to_string()
}

/// ボード寸法を返す
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_board_dimensions() -> Vec<usize> {
    vec![BOARD_WIDTH, BOARD_HEIGHT]
}

/// 現在のテトロミノの全ブロック座標を取得
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmGameState {
    pub fn get_current_piece_blocks(&self) -> Vec<i32> {
        if let Some(ref piece) = self.current_piece {
            let blocks = piece.get_blocks_at_rotation(piece.rotation);
            let mut result = Vec::new();
            
            for (dx, dy) in blocks {
                let board_x = piece.x as i8 + dx;
                let board_y = piece.y as i8 + dy;
                result.push(board_x as i32);
                result.push(board_y as i32);
                result.push(piece.color as i32); // 色情報も含める
            }
            
            result
        } else {
            vec![]
        }
    }

    /// 次のテトロミノの情報を取得 [x, y, rotation, color, shape]
    pub fn get_next_piece_info(&self) -> Vec<i32> {
        if let Some(ref piece) = self.next_piece {
            vec![
                piece.x as i32,
                piece.y as i32, 
                piece.rotation as i32,
                piece.color as i32,
                piece.shape as i32
            ]
        } else {
            vec![]
        }
    }

    /// 次のテトロミノの全ブロック座標を取得（プレビュー用）
    pub fn get_next_piece_blocks(&self) -> Vec<i32> {
        if let Some(ref piece) = self.next_piece {
            let blocks = piece.get_blocks_at_rotation(piece.rotation);
            let mut result = Vec::new();
            
            for (dx, dy) in blocks {
                // 次ピース表示用なので固定位置（0,0基準）で座標を返す
                result.push(dx as i32);
                result.push(dy as i32);
                result.push(piece.color as i32);
            }
            
            result
        } else {
            vec![]
        }
    }

    /// ゴーストピースのブロック座標を取得
    pub fn get_ghost_piece_blocks(&self) -> Vec<i32> {
        if let Some(ref piece) = self.current_piece {
            if let Some((ghost_x, ghost_y)) = self.calculate_ghost_position(piece) {
                let blocks = piece.get_blocks_at_rotation(piece.rotation);
                let mut result = Vec::new();
                
                for (dx, dy) in blocks {
                    let board_x = ghost_x + dx;
                    let board_y = ghost_y + dy;
                    result.push(board_x as i32);
                    result.push(board_y as i32);
                    result.push(piece.color as i32); // 同じ色で表示（半透明化はフロントエンド側で処理）
                }
                
                result
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
    
    /// アニメーション処理を実行
    #[wasm_bindgen]
    pub fn update_animation(&mut self) {
        if self.animation_phase == 0 {
            return;
        }
        
        let current_time = self.time_provider.now();
        let start_time = self.animation_start_time.unwrap();
        let elapsed = current_time - start_time;
        
        if self.animation_phase == 1 {
            // 点滅フェーズ（500ms）
            if elapsed >= Duration::from_millis(500) {
                // 点滅終了、実際にラインを消去
                for &y in &self.clearing_lines {
                    self.board.remove(y);
                    self.board.insert(0, vec![Cell::Empty; BOARD_WIDTH]);
                    self.score += 100; // 基本スコア
                }
                
                console_log!("Cleared {} lines, score: {}", self.clearing_lines.len(), self.score);
                
                // アニメーション終了
                self.clearing_lines.clear();
                self.animation_start_time = None;
                self.animation_phase = 0;
                
                // 新しいピースをスポーン
                self.spawn_piece();
            }
        }
    }
    
    /// アニメーション情報を取得（JavaScript用）
    #[wasm_bindgen]
    pub fn get_animation_info(&self) -> Vec<i32> {
        if self.animation_phase == 0 {
            return vec![];
        }
        
        let mut result = vec![self.animation_phase as i32];
        
        if let Some(start_time) = self.animation_start_time {
            let current_time = self.time_provider.now();
            let elapsed_ms = (current_time - start_time).as_millis() as i32;
            result.push(elapsed_ms);
            
            // 点滅アニメーションの場合、対象ライン情報も追加
            if self.animation_phase == 1 {
                result.push(self.clearing_lines.len() as i32);
                for &line in &self.clearing_lines {
                    result.push(line as i32);
                }
            }
        }
        
        result
    }
}