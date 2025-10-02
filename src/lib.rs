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
            x: BOARD_WIDTH / 2 - 1,
            y: 0,
            rotation: 0,
            color: colors[color_index],
            shape,
        }
    }
    
    pub fn get_blocks(&self) -> Vec<(i8, i8)> {
        match self.shape {
            0 => vec![(0, 0), (1, 0), (2, 0), (3, 0)], // I piece
            1 => vec![(0, 0), (1, 0), (0, 1), (1, 1)], // O piece
            2 => vec![(1, 0), (0, 1), (1, 1), (2, 1)], // T piece
            3 => vec![(0, 0), (0, 1), (0, 2), (1, 2)], // L piece
            4 => vec![(1, 0), (1, 1), (1, 2), (0, 2)], // J piece
            5 => vec![(1, 0), (2, 0), (0, 1), (1, 1)], // S piece
            6 => vec![(0, 0), (1, 0), (1, 1), (2, 1)], // Z piece
            _ => vec![(0, 0)], // Default
        }
    }
    
    pub fn get_blocks_at_rotation(&self, rotation: u8) -> Vec<(i8, i8)> {
        let base_blocks = self.get_blocks();
        if rotation == 0 || self.shape == 1 { // O piece doesn't rotate
            return base_blocks;
        }
        
        // 簡単な90度回転実装
        base_blocks.iter().map(|(x, y)| {
            match rotation {
                1 => (-y, *x),
                2 => (-x, -y),
                3 => (*y, -x),
                _ => (*x, *y),
            }
        }).collect()
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
        self.spawn_piece();
    }
    
    /// 新しいピースを生成
    #[wasm_bindgen]
    pub fn spawn_piece(&mut self) {
        console_log!("Spawning new piece");
        let piece = SimpleTetromino::new_random();
        self.current_piece = Some(piece);
        
        if self.next_piece.is_none() {
            self.next_piece = Some(SimpleTetromino::new_random());
        }
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
    
    /// 現在のピースを回転
    #[wasm_bindgen]
    pub fn rotate_current_piece(&mut self, clockwise: bool) -> bool {
        if let Some(ref piece) = self.current_piece {
            let new_rotation = if clockwise {
                (piece.rotation + 1) % 4
            } else {
                (piece.rotation + 3) % 4
            };
            
            if self.is_valid_position(piece, piece.x as i8, piece.y as i8, new_rotation) {
                if let Some(ref mut piece) = self.current_piece {
                    piece.rotation = new_rotation;
                }
                console_log!("Rotated piece to rotation {}", new_rotation);
                return true;
            }
        }
        false
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
        if let Some(piece) = &self.current_piece {
            // ピースをボードに配置
            let blocks = piece.get_blocks();
            for (dx, dy) in blocks {
                let board_x = piece.x as i8 + dx;
                let board_y = piece.y as i8 + dy;
                
                if board_x >= 0 && board_x < BOARD_WIDTH as i8 && 
                   board_y >= 0 && board_y < BOARD_HEIGHT as i8 {
                    self.board[board_y as usize][board_x as usize] = Cell::Occupied(piece.color);
                }
            }
            
            // 新しいピースを生成
            self.current_piece = self.next_piece.take();
            self.next_piece = Some(SimpleTetromino::new_random());
            
            // ライン消去チェック
            self.clear_lines();
            
            console_log!("Piece locked and new piece spawned");
        }
    }
    
    /// ラインクリア処理
    #[wasm_bindgen]
    pub fn clear_lines(&mut self) {
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
        
        // ラインを消去してスコア加算
        for &y in &lines_to_clear {
            self.board.remove(y);
            self.board.insert(0, vec![Cell::Empty; BOARD_WIDTH]);
            self.score += 100; // 基本スコア
        }
        
        if !lines_to_clear.is_empty() {
            console_log!("Cleared {} lines, score: {}", lines_to_clear.len(), self.score);
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
            
            // 境界チェック
            if board_x < 0 || board_x >= BOARD_WIDTH as i8 ||
               board_y < 0 || board_y >= BOARD_HEIGHT as i8 {
                return false;
            }
            
            // 衝突チェック
            if !matches!(self.board[board_y as usize][board_x as usize], Cell::Empty) {
                return false;
            }
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
}