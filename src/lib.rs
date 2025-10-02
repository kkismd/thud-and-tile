//! Thud & Tile WASM Library
//! 
//! このモジュールは、Thud & TileゲームのWASM環境用エントリーポイントを提供します。
//! JavaScript環境からアクセス可能なAPIを実装し、ゲームロジックとUI間の橋渡しを行います。

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// JavaScript console.log への出力用マクロ
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
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
use random::{RandomProviderImpl, create_default_random_provider};
use cell::Cell;

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
    current_piece: Option<tetromino::Tetromino>,
    next_piece: Option<tetromino::Tetromino>,
    game_mode: u8, // 0: Title, 1: Playing, 2: GameOver
    random_provider: RandomProviderImpl,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmGameState {
    /// 新しいゲーム状態を作成
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGameState {
        console_log!("Creating new WasmGameState");
        
        WasmGameState {
            board: vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            score: 0,
            current_piece: None,
            next_piece: None,
            game_mode: 0, // Title
            random_provider: create_default_random_provider(),
        }
    }
    
    /// ゲームを開始
    #[wasm_bindgen]
    pub fn start_game(&mut self) {
        console_log!("Starting new game");
        self.game_mode = 1; // Playing
        self.score = 0;
        self.board = vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        self.spawn_piece();
    }
    
    /// 新しいピースを生成
    #[wasm_bindgen]
    pub fn spawn_piece(&mut self) {
        console_log!("Spawning new piece");
        self.current_piece = Some(tetromino::Tetromino::new_random());
        self.next_piece = Some(tetromino::Tetromino::new_random());
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
        
        // 簡単な入力処理（実際のゲームロジックは後で実装）
        match game_input {
            GameInput::Restart => {
                self.start_game();
                true
            }
            GameInput::Quit => {
                self.game_mode = 0; // Title
                true
            }
            _ => {
                // その他の入力処理は後で実装
                false
            }
        }
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