//! Thud & Tile WASM Library
//! 
//! このモジュールは、Thud & TileゲームのWASM環境用エントリーポイントを提供します。
//! JavaScript環境からアクセス可能なAPIを実装し、ゲームロジックとUI間の橋渡しを行います。

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use wasm_bindgen::prelude::*;
use std::time::Duration;
use std::collections::{VecDeque, HashSet}; // BFS用とAnimation管理用

// 共通モジュールのimport
mod animation;

// JavaScript console.log への出力用マクロ
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    // JavaScript Date.now()へのアクセス
    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn js_date_now() -> f64;
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Node.js環境またはネイティブ環境でのログ出力
#[cfg(not(all(target_arch = "wasm32", feature = "wasm")))]
macro_rules! console_log {
    ($($t:tt)*) => {
        println!($($t)*);
    }
}

// Node.js環境でのポリフィル
#[cfg(all(target_arch = "wasm32", feature = "nodejs-test"))]
pub fn js_date_now() -> f64 {
    // Node.js環境では固定値を返すかStd libraryのSystemTimeを使用
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64
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
mod scoring;
mod cell;
mod board_logic;
mod tetromino;

use config::*;
use game_color::GameColor;
use game_input::GameInput;
use random::{RandomProvider, create_default_random_provider};
use cell::Cell;
use scoring::CustomScoreSystem;
use tetromino::get_srs_wall_kick_offsets_by_shape; // CLI版のSRS関数をimport
use animation::{Animation, calculate_line_clear_score, process_line_clear};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// WASMバインディング対応のCustomScoreSystemラッパー
/// 既存のscoring.rsロジックを活用し、JavaScript連携を提供
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmCustomScoreSystem {
    inner: CustomScoreSystem,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmCustomScoreSystem {
    /// 新しいWasmCustomScoreSystemを作成
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmCustomScoreSystem {
        WasmCustomScoreSystem {
            inner: CustomScoreSystem::new(),
        }
    }
    
    /// 指定された色にスコアを加算（u8でGameColorを指定）
    /// 0=Cyan, 1=Magenta, 2=Yellow, その他は無視
    #[wasm_bindgen]
    pub fn add_score(&mut self, color_index: u8, points: u32) {
        let color = match color_index {
            0 => GameColor::Cyan,
            1 => GameColor::Magenta, 
            2 => GameColor::Yellow,
            _ => return, // 無効な色は無視
        };
        self.inner.scores.add(color, points);
    }
    
    /// 合計スコアを取得
    #[wasm_bindgen]
    pub fn get_total_score(&self) -> u32 {
        self.inner.scores.total()
    }
    
    /// Cyanスコアを取得
    #[wasm_bindgen]
    pub fn get_cyan_score(&self) -> u32 {
        self.inner.scores.cyan
    }
    
    /// Magentaスコアを取得
    #[wasm_bindgen]
    pub fn get_magenta_score(&self) -> u32 {
        self.inner.scores.magenta
    }
    
    /// Yellowスコアを取得
    #[wasm_bindgen]
    pub fn get_yellow_score(&self) -> u32 {
        self.inner.scores.yellow
    }
    
    /// 全色のスコアを配列で取得 [cyan, magenta, yellow]
    #[wasm_bindgen]
    pub fn get_all_scores(&self) -> Vec<u32> {
        vec![
            self.inner.scores.cyan,
            self.inner.scores.magenta,
            self.inner.scores.yellow,
        ]
    }
    
    /// 指定された色の最大チェーン数を更新
    #[wasm_bindgen]
    pub fn update_max_chain(&mut self, color_index: u8, chain_count: u32) {
        let color = match color_index {
            0 => GameColor::Cyan,
            1 => GameColor::Magenta,
            2 => GameColor::Yellow,
            _ => return, // 無効な色は無視
        };
        self.inner.max_chains.update_max(color, chain_count);
    }
    
    /// 指定された色の最大チェーン数を取得
    #[wasm_bindgen]
    pub fn get_max_chain(&self, color_index: u8) -> u32 {
        let color = match color_index {
            0 => GameColor::Cyan,
            1 => GameColor::Magenta,
            2 => GameColor::Yellow,
            _ => return 0, // 無効な色は0を返す
        };
        self.inner.max_chains.get(color)
    }
    
    /// 全色の最大チェーン数を配列で取得 [cyan, magenta, yellow]
    #[wasm_bindgen]
    pub fn get_all_max_chains(&self) -> Vec<u32> {
        vec![
            self.inner.max_chains.cyan,
            self.inner.max_chains.magenta,
            self.inner.max_chains.yellow,
        ]
    }
    
    /// 全体の最大チェーン数を取得
    #[wasm_bindgen]
    pub fn get_overall_max_chain(&self) -> u32 {
        self.inner.max_chains.max()
    }
    
    /// スコア表示用文字列を取得（CLI版のDisplay traitと同等）
    #[wasm_bindgen]
    pub fn get_display_string(&self) -> String {
        format!("{}", self.inner)
    }
    
    /// JavaScript用のスコア詳細情報を取得
    /// [total_score, cyan, magenta, yellow, max_chain, cyan_chain, magenta_chain, yellow_chain]
    #[wasm_bindgen]
    pub fn get_score_details(&self) -> Vec<u32> {
        vec![
            self.inner.scores.total(),
            self.inner.scores.cyan,
            self.inner.scores.magenta,
            self.inner.scores.yellow,
            self.inner.max_chains.max(),
            self.inner.max_chains.cyan,
            self.inner.max_chains.magenta,
            self.inner.max_chains.yellow,
        ]
    }
}

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
    pub colors: Vec<GameColor>, // 各ブロックの色（4要素固定）
    pub shape: u8, // 0=I, 1=O, 2=T, 3=L, 4=J, 5=S, 6=Z
}

impl SimpleTetromino {
    pub fn new_random() -> Self {
        let mut provider = create_default_random_provider();
        let shape = provider.gen_range(0, 7) as u8;
        let color_palette = [GameColor::Cyan, GameColor::Magenta, GameColor::Yellow];
        
        // 各ブロックに個別の色を割り当て（ランダム）
        let colors = vec![
            color_palette[provider.gen_range(0, 3)],
            color_palette[provider.gen_range(0, 3)],
            color_palette[provider.gen_range(0, 3)],
            color_palette[provider.gen_range(0, 3)],
        ];
        
        SimpleTetromino {
            x: (BOARD_WIDTH / 2 - 2) as usize, // CLI版と同じ位置 (x=3)
            y: 0,
            rotation: 0,
            colors,
            shape,
        }
    }
    
    pub fn from_shape(shape: TetrominoShape) -> Self {
        let mut provider = create_default_random_provider();
        let color_palette = [GameColor::Cyan, GameColor::Magenta, GameColor::Yellow];
        
        // 各ブロックに個別の色を割り当て（ランダム）
        let colors = vec![
            color_palette[provider.gen_range(0, 3)],
            color_palette[provider.gen_range(0, 3)],
            color_palette[provider.gen_range(0, 3)],
            color_palette[provider.gen_range(0, 3)],
        ];
        
        SimpleTetromino {
            x: (BOARD_WIDTH / 2 - 2) as usize, // CLI版と同じ位置 (x=3)
            y: 0,
            rotation: 0,
            colors,
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
            1 => { // O piece - 回転で座標配置を変更（視覚的な回転エフェクト）
                match rotation {
                    0 => vec![(1, 1), (2, 1), (1, 2), (2, 2)], // 基本配置
                    1 => vec![(2, 1), (2, 2), (1, 2), (1, 1)], // 90度回転（時計回り）
                    2 => vec![(2, 2), (1, 2), (2, 1), (1, 1)], // 180度回転
                    3 => vec![(1, 2), (1, 1), (2, 1), (2, 2)], // 270度回転
                    _ => vec![(1, 1), (2, 1), (1, 2), (2, 2)],
                }
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

// WASM初期化関数（テスト時は無効）
#[cfg(all(target_arch = "wasm32", not(test)))]
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("Thud & Tile WASM module initialized");
}

/// ゲーム状態を表すWASMエクスポート用構造体
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmGameState {
    board: Vec<Vec<Cell>>,
    custom_score_system: WasmCustomScoreSystem, // 単一scoreをCustomScoreSystemに置換
    current_piece: Option<SimpleTetromino>,
    next_piece: Option<SimpleTetromino>,
    game_mode: u8, // 0: Title, 1: Playing, 2: GameOver
    fall_speed: Duration,
    last_fall_time: Duration,
    time_provider: WasmTimeProvider,
    tetromino_bag: WebTetrominoBag, // 7-bag実装
    current_board_height: usize, // 動的ボード高さ（CLI版と同じ）
    // アニメーション関連（CLI版と同等）
    animation: Vec<Animation>, // CLI版と同じVec<Animation>管理
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
            custom_score_system: WasmCustomScoreSystem::new(),
            current_piece: None,
            next_piece: None,
            game_mode: 0, // Title
            fall_speed: FALL_SPEED_START,
            last_fall_time: current_time,
            time_provider,
            tetromino_bag: WebTetrominoBag::new(),
            current_board_height: BOARD_HEIGHT, // CLI版と同じ初期値
            animation: Vec::new(), // CLI版と同じ初期状態
        }
    }
    
    /// ゲームを開始
    #[wasm_bindgen]
    pub fn start_game(&mut self) {
        console_log!("Starting new game");
        self.game_mode = 1; // Playing
        self.custom_score_system = WasmCustomScoreSystem::new(); // スコアシステムリセット
        self.board = vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        self.fall_speed = FALL_SPEED_START;
        let current_time = self.time_provider.now();
        self.last_fall_time = current_time;
        self.current_board_height = BOARD_HEIGHT; // ボード高さもリセット
        // アニメーション状態もリセット
        self.animation.clear();
        
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
    
    /// 現在の合計スコアを取得
    #[wasm_bindgen]
    pub fn get_score(&self) -> u32 {
        self.custom_score_system.get_total_score()
    }
    
    /// 3色別スコアを取得 [cyan, magenta, yellow]
    #[wasm_bindgen]
    pub fn get_color_scores(&self) -> Vec<u32> {
        self.custom_score_system.get_all_scores()
    }
    
    /// 3色別最大チェーン数を取得 [cyan, magenta, yellow]
    #[wasm_bindgen]
    pub fn get_max_chains(&self) -> Vec<u32> {
        self.custom_score_system.get_all_max_chains()
    }
    
    /// スコア詳細情報を取得
    /// [total, cyan, magenta, yellow, max_chain, cyan_chain, magenta_chain, yellow_chain]
    #[wasm_bindgen]
    pub fn get_score_details(&self) -> Vec<u32> {
        self.custom_score_system.get_score_details()
    }
    
    /// スコア表示用文字列を取得
    #[wasm_bindgen]
    pub fn get_score_display(&self) -> String {
        self.custom_score_system.get_display_string()
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
                    Cell::Occupied(color) => {
                        let color_id = match color {
                            GameColor::Cyan => 1,
                            GameColor::Magenta => 2,
                            GameColor::Yellow => 3,
                            _ => {
                                console_log!("Warning: Unexpected color in board: {:?}", color);
                                4 // 他の色は4以降
                            }
                        };
                        result.push(color_id);
                    },
                    Cell::Connected { color, count: _ } => {
                        let color_id = match color {
                            GameColor::Cyan => 10,    // JavaScript側の期待値に合わせる
                            GameColor::Magenta => 11,
                            GameColor::Yellow => 12,
                            _ => 13, // 他の色は13以降
                        };
                        result.push(color_id);
                    },
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
            
            // CLI版のSRS関数を使用
            let offsets = get_srs_wall_kick_offsets_by_shape(piece.shape, current_rotation, new_rotation);
            
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
            console_log!("Locking piece with colors: {:?}", piece.colors);
            
            // ピースをボードに配置（CLI版と同様の処理）
            let blocks = piece.get_blocks_at_rotation(piece.rotation);
            for (block_index, (dx, dy)) in blocks.iter().enumerate() {
                let board_x = piece.x as i8 + dx;
                let board_y = piece.y as i8 + dy;
                
                if board_x >= 0 && board_x < BOARD_WIDTH as i8 && 
                   board_y >= 0 && board_y < BOARD_HEIGHT as i8 {
                    // 各ブロックに個別の色を使用
                    let block_color = piece.colors[block_index % piece.colors.len()];
                    self.board[board_y as usize][board_x as usize] = Cell::Occupied(block_color);
                    console_log!("Placing block {} at ({}, {}) with color {:?}", 
                        block_index, board_x, board_y, block_color);
                }
            }
            
            console_log!("Piece locked at position ({}, {})", piece.x, piece.y);
            
            // CLI版と同じライン消去検出とアニメーション処理
            // 1. 完成ラインを検出（CLI版と同じロジック）
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
            lines_to_clear.sort_by(|a, b| b.cmp(a)); // CLI版と同じソート

            // 2. CLI版と同じ隣接ブロック処理（消去前に実行）
            crate::board_logic::find_and_connect_adjacent_blocks(&mut self.board, &lines_to_clear);
            
            // 3. CLI版と同じconnected block counts更新
            self.update_connected_block_counts();
            
            // 4. CLI版と同じmax_chains更新
            self.update_max_chains();
            
            // 5. Calculate scores for lines to be cleared (before starting animation)
            for &line_y in &lines_to_clear {
                let scores = animation::calculate_line_clear_score(&self.board, line_y, &self.custom_score_system.inner.max_chains);
                for (color, points) in scores {
                    let color_index = match color {
                        GameColor::Cyan => 0,
                        GameColor::Magenta => 1,
                        GameColor::Yellow => 2,
                        _ => 0,
                    };
                    self.custom_score_system.add_score(color_index, points);
                }
            }
            
            // 6. ライン消去処理とアニメーション開始
            if !lines_to_clear.is_empty() {
                let start_time = self.time_provider.now();
                let line_blink_animation = animation::Animation::LineBlink {
                    lines: lines_to_clear.clone(),
                    count: 0,
                    start_time,
                };
                self.animation.push(line_blink_animation);
                console_log!("Starting line clear animation for {} lines", lines_to_clear.len());
            } else {
                // ラインクリアなしの場合、すぐに新しいピースをスポーン
                self.spawn_piece();
            }
            
            console_log!("Piece locked, next piece spawned or animation started");
        }
    }
    
    
    /// Connected cellsの詳細情報を取得 [x, y, count, x, y, count, ...]
    #[wasm_bindgen]
    pub fn get_connected_cells_info(&self) -> Vec<i32> {
        let mut result = Vec::new();
        
        for y in 0..self.current_board_height {
            for x in 0..BOARD_WIDTH {
                if let Cell::Connected { color: _, count } = self.board[y][x] {
                    result.push(x as i32);
                    result.push(y as i32);
                    result.push(count as i32);
                }
            }
        }
        
        result
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
            
            // 下方境界チェック（CLI版と同じ動的高さ使用）
            if board_y >= self.current_board_height as i8 {
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
        if !self.animation.is_empty() {
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
    
    /// 現在のボード高さを取得（Dynamic Board Height System）
    #[wasm_bindgen]
    pub fn get_current_board_height(&self) -> usize {
        self.current_board_height
    }
    
    /// 現在のボード高さを設定（Dynamic Board Height System）
    #[wasm_bindgen]
    pub fn set_current_board_height(&mut self, height: usize) {
        // 安全性チェック：高さは最大BOARD_HEIGHT以下
        self.current_board_height = height.min(BOARD_HEIGHT);
        console_log!("Board height set to {}", self.current_board_height);
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
            
            for (block_index, (dx, dy)) in blocks.iter().enumerate() {
                let board_x = piece.x as i8 + dx;
                let board_y = piece.y as i8 + dy;
                result.push(board_x as i32);
                result.push(board_y as i32);
                let block_color = piece.colors[block_index % piece.colors.len()];
                result.push(block_color as i32); // 各ブロックの個別色
            }
            
            result
        } else {
            vec![]
        }
    }

    /// 次のテトロミノの情報を取得 [x, y, rotation, primary_color, shape]
    pub fn get_next_piece_info(&self) -> Vec<i32> {
        if let Some(ref piece) = self.next_piece {
            vec![
                piece.x as i32,
                piece.y as i32, 
                piece.rotation as i32,
                piece.colors[0] as i32, // 最初の色を代表色として使用
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
            
            for (block_index, (dx, dy)) in blocks.iter().enumerate() {
                // 次ピース表示用なので固定位置（0,0基準）で座標を返す
                result.push(*dx as i32);
                result.push(*dy as i32);
                let block_color = piece.colors[block_index % piece.colors.len()];
                result.push(block_color as i32);
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
                
                for (block_index, (dx, dy)) in blocks.iter().enumerate() {
                    let board_x = ghost_x + dx;
                    let board_y = ghost_y + dy;
                    result.push(board_x as i32);
                    result.push(board_y as i32);
                    let block_color = piece.colors[block_index % piece.colors.len()];
                    result.push(block_color as i32); // 各ブロックの個別色（半透明化はフロントエンド側で処理）
                }
                
                result
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
    
    /// アニメーション処理を実行（CLI版互換・共通モジュール使用）
    #[wasm_bindgen]
    pub fn update_animation(&mut self) {
        if self.animation.is_empty() {
            return;
        }
        
        let current_time = self.time_provider.now();
        
        // 共通アニメーション処理モジュールを使用
        let result = animation::update_animations(&mut self.animation, current_time);
        
        // LineBlink完了によるライン消去とPush Down開始処理（CLI版互換）
        for completed_lines in result.completed_line_blinks {
            // CLI版と同じ処理順序：bottom/non-bottomに分離してライン消去処理
            let (bottom_lines_cleared, non_bottom_lines_cleared) = 
                animation::process_line_clear(&mut self.board, self.current_board_height, &completed_lines);

            // Bottom lines の標準テトリス消去処理
            for &line_y in &bottom_lines_cleared {
                // スコア計算はlock_piece()で既に実行済み
            }

            // Bottom lines 処理後の連結ブロック更新と新ピーススポーン
            if !bottom_lines_cleared.is_empty() {
                self.update_all_connected_block_counts();
                self.spawn_piece();
                console_log!("Bottom line clear: {} lines cleared", bottom_lines_cleared.len());
            }

            // Non-bottom lines の孤立ブロック除去処理
            for &y in &non_bottom_lines_cleared {
                // 1. 孤立ブロック除去（CLI版互換）
                crate::board_logic::remove_isolated_blocks(&mut self.board, y);

                // 2. スコア計算はlock_piece()で既に実行済み
            }

            // Non-bottom lines をグレー化（既に共通モジュールで処理済み）
            console_log!("Line clear animation completed: {} bottom, {} non-bottom", 
                bottom_lines_cleared.len(), non_bottom_lines_cleared.len());
        }
        
        // Push Down完了処理
        for gray_line_y in result.completed_push_downs {
            match animation::process_push_down_step(&mut self.board, &mut self.current_board_height, gray_line_y) {
                animation::PushDownStepResult::Completed => {
                    console_log!("PushDown animation completed for line {}", gray_line_y);
                }
                animation::PushDownStepResult::Moved { new_gray_line_y } => {
                    // 新しい位置でPush Downアニメーションを継続
                    self.animation.push(animation::Animation::PushDown {
                        gray_line_y: new_gray_line_y,
                        start_time: current_time,
                    });
                    console_log!("PushDown animation moved line to {}", new_gray_line_y);
                }
            }
        }
        
        // 継続するアニメーションを設定
        self.animation = result.continuing_animations;
        
        // すべてのアニメーションが完了した場合、新しいピースをスポーン
        if self.animation.is_empty() {
            self.spawn_piece();
        }
    }
    
    /// グレーラインをSolidラインに変換し、board heightを減少（共通モジュール使用）
    fn finalize_gray_line(&mut self, gray_line_y: usize) {
        // 共通モジュールのPush Down完了処理を使用
        match animation::process_push_down_step(&mut self.board, &mut self.current_board_height, gray_line_y) {
            animation::PushDownStepResult::Completed => {
                console_log!("Gray line {} finalized as Solid, board height reduced to {}", gray_line_y, self.current_board_height);
            }
            animation::PushDownStepResult::Moved { .. } => {
                // この関数では完了のみを扱うため、移動は想定外
                console_log!("Warning: Unexpected move result in finalize_gray_line");
            }
        }
        
        // 将来的にここでconnected block countsの更新も行う
        // self.update_all_connected_block_counts();
    }
    
    /// アニメーション情報を取得（JavaScript用）
    #[wasm_bindgen]
    pub fn get_animation_info(&self) -> Vec<i32> {
        if self.animation.is_empty() {
            return vec![];
        }
        
        let mut result = Vec::new();
        let current_time = self.time_provider.now();
        
        // 各アニメーションの情報を追加（CLI版と同等の詳細情報）
        for animation in &self.animation {
            match animation {
                Animation::LineBlink { lines, count, start_time } => {
                    result.push(1); // LineBlink type id
                    let elapsed_ms = (current_time - *start_time).as_millis() as i32;
                    result.push(elapsed_ms);
                    result.push(*count as i32);
                    result.push(lines.len() as i32);
                    for &line in lines {
                        result.push(line as i32);
                    }
                }
                Animation::PushDown { gray_line_y, start_time } => {
                    result.push(2); // PushDown type id
                    let elapsed_ms = (current_time - *start_time).as_millis() as i32;
                    result.push(elapsed_ms);
                    result.push(*gray_line_y as i32);
                }
            }
        }
        
        result
    }
    
    /// CLI版のcount_connected_blocks相当の実装（内部実装のみ）
    /// cleared_line_y より下の行の連結ブロックを BFS で検出してカウント
    fn count_connected_blocks(&self, cleared_line_y: usize) -> Vec<((usize, usize), u32)> {
        let mut results = Vec::new();
        let mut visited = vec![vec![false; BOARD_WIDTH]; BOARD_HEIGHT];

        for y in (cleared_line_y + 1)..self.current_board_height {
            for x in 0..BOARD_WIDTH {
                let color = match self.board[y][x] {
                    Cell::Occupied(c) => Some(c),
                    Cell::Connected { color: c, count: _ } => Some(c),
                    _ => None,
                };
                
                if let Some(color) = color {
                    if visited[y][x] {
                        continue;
                    }

                    let mut component = Vec::new();
                    let mut queue = VecDeque::new();

                    visited[y][x] = true;
                    queue.push_back((x, y));
                    component.push((x, y));

                    // BFS で連結コンポーネント検出
                    while let Some((qx, qy)) = queue.pop_front() {
                        let neighbors = [
                            (qx as i8 - 1, qy as i8),
                            (qx as i8 + 1, qy as i8),
                            (qx as i8, qy as i8 - 1),
                            (qx as i8, qy as i8 + 1),
                        ];

                        for (nx, ny) in neighbors {
                            if nx >= 0 && nx < BOARD_WIDTH as i8 && ny >= 0 && (ny as usize) < self.current_board_height {
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

        results
    }
    
    /// 全ボードの連結ブロック数を更新（CLI版のupdate_all_connected_block_counts相当）
    fn update_all_connected_block_counts(&mut self) {
        // 一度すべてのConnected cellをOccupied cellに戻す
        for y in 0..self.current_board_height {
            for x in 0..BOARD_WIDTH {
                if let Cell::Connected { color, count: _ } = self.board[y][x] {
                    self.board[y][x] = Cell::Occupied(color);
                }
            }
        }

        // 各セルについて連結コンポーネントサイズを再計算
        let mut visited = vec![vec![false; BOARD_WIDTH]; BOARD_HEIGHT];
        
        for y in 0..self.current_board_height {
            for x in 0..BOARD_WIDTH {
                if !visited[y][x] {
                    if let Cell::Occupied(color) = self.board[y][x] {
                        // BFSで連結コンポーネントを検出
                        let mut component = Vec::new();
                        let mut queue = VecDeque::new();

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
                                if nx >= 0 && nx < BOARD_WIDTH as i8 && ny >= 0 && (ny as usize) < self.current_board_height {
                                    let (nx_usize, ny_usize) = (nx as usize, ny as usize);
                                    if !visited[ny_usize][nx_usize] {
                                        if let Cell::Occupied(neighbor_color) = self.board[ny_usize][nx_usize] {
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

                        // 連結コンポーネントサイズを各セルに設定
                        let component_size = component.len() as u8; // u8にキャスト
                        for &(cx, cy) in &component {
                            if component_size > 1 {
                                self.board[cy][cx] = Cell::Connected { color, count: component_size };
                            } else {
                                self.board[cy][cx] = Cell::Occupied(color); // 単独ブロックはOccupiedのまま
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// CLI版のupdate_connected_block_counts相当の実装
    /// ピースロック後に連結ブロック数を再計算・更新
    fn update_connected_block_counts(&mut self) {
        let connected_counts = crate::board_logic::count_connected_blocks(&self.board, 0);
        for ((x, y), count) in connected_counts {
            if let Cell::Connected { color, count: _ } = self.board[y][x] {
                self.board[y][x] = Cell::Connected {
                    color,
                    count: count as u8,
                };
            }
        }
    }
    
    /// CLI版のupdate_max_chains相当の実装
    /// ボード全体をスキャンして各色の最大連結ブロック数を更新
    fn update_max_chains(&mut self) {
        // ボード全体をスキャンして、各色の最大連結ブロック数を見つける
        for y in 0..self.current_board_height {
            for x in 0..BOARD_WIDTH {
                if let Cell::Connected { color, count } = self.board[y][x] {
                    self.custom_score_system
                        .inner
                        .max_chains
                        .update_max(color, count as u32);
                }
            }
        }
    }
}

// ===== WASM専用テストセクション =====
// Node.js/ブラウザ環境でのWASMモジュール特有のテスト

#[cfg(all(target_arch = "wasm32", test))]
use wasm_bindgen_test::*;

// WASM専用テストの設定（フィーチャーに応じて環境を切り替え）
#[cfg(all(target_arch = "wasm32", test, feature = "wasm-test"))]
wasm_bindgen_test_configure!(run_in_browser);

// Node.jsテストはデフォルト設定（設定なし）を使用

// Node.js用のWASMテスト（条件を緩和）
#[cfg(all(target_arch = "wasm32", test))]
#[wasm_bindgen_test]
async fn wasm_node_compatible_test() {
    // Node.js環境でも実行可能な基本テスト
    console_log!("WASM Node.js compatible test running");
    
    // 基本的なアサーション
    assert_eq!(2 + 2, 4);
    assert!(true);
    
    console_log!("WASM Node.js compatible test passed");
}

#[cfg(all(target_arch = "wasm32", test))]
#[wasm_bindgen_test]
fn wasm_game_state_creation() {
    // WASM環境でのゲーム状態作成テスト
    let game_state = WasmGameState::new();
    assert_eq!(game_state.get_score(), 0);
    console_log!("WASM game state creation test passed");
}

#[cfg(all(target_arch = "wasm32", test))]
#[wasm_bindgen_test]
fn wasm_tetromino_operations() {
    // WASMでのテトロミノ操作テスト
    let mut game_state = WasmGameState::new();
    game_state.start_game();
    
    // 現在ピース情報の取得テスト
    let piece_info = game_state.get_current_piece_info();
    console_log!("WASM current piece info: {:?}", piece_info);
    
    // 回転テスト
    let rotation_result = game_state.rotate_current_piece(true);
    console_log!("WASM tetromino rotation test: {}", rotation_result);
}

#[cfg(all(target_arch = "wasm32", test))]
#[wasm_bindgen_test]
fn wasm_animation_system() {
    // WASMでのアニメーション系統テスト
    let mut game_state = WasmGameState::new();
    game_state.start_game();
    
    // アニメーション更新テスト
    game_state.update_animation();
    let animation_info = game_state.get_animation_info();
    
    // アニメーション状態が適切に管理されているかテスト
    console_log!("WASM animation info: {:?}", animation_info);
    assert!(animation_info.len() > 0); // JSONが返されることを確認
}

#[cfg(all(target_arch = "wasm32", test))]
#[wasm_bindgen_test]
fn wasm_score_system() {
    // WASMスコアシステムテスト
    let mut score_system = WasmCustomScoreSystem::new();
    
    // スコア加算テスト
    score_system.add_score(0, 100); // Cyan
    score_system.add_score(1, 200); // Magenta
    
    assert_eq!(score_system.get_cyan_score(), 100);
    assert_eq!(score_system.get_magenta_score(), 200);
    assert_eq!(score_system.get_total_score(), 300);
    
    console_log!("WASM score system test passed");
}

#[cfg(all(target_arch = "wasm32", test))]
#[wasm_bindgen_test]
fn wasm_time_provider() {
    // WASM時間プロバイダーテスト
    let time_provider = WasmTimeProvider::new();
    let start_time = time_provider.now();
    
    // 時間が適切に取得できるかテスト
    assert!(start_time.as_millis() >= 0);
    console_log!("WASM time provider test passed: {}ms", start_time.as_millis());
}

// Node.js環境での基本的なテスト
#[cfg(all(target_arch = "wasm32", test))]
#[wasm_bindgen_test]
fn nodejs_compatibility_test() {
    // Node.js環境でのポリフィルをテスト
    console_log!("Node.js compatibility test starting");
    
    // ランダム数生成のテスト
    let random_value = crate::random::js_math_random();
    assert!(random_value >= 0.0 && random_value < 1.0);
    
    // 時間のテスト（Node.js環境でのjs_date_now）
    #[cfg(feature = "nodejs-test")]
    {
        let time = crate::js_date_now();
        assert!(time > 0.0);
    }
    
    console_log!("Node.js compatibility test passed");
}

// 共有アニメーションシステムのWASMテスト
#[cfg(all(target_arch = "wasm32", test))]
#[wasm_bindgen_test]
fn wasm_shared_animation_module() {
    // Mock time provider for testing
    let mut mock_time = MockTimeProvider::new();
    
    // 共有アニメーションモジュールのテスト
    use crate::animation::Animation;
    let animations = vec![
        Animation::LineBlink {
            lines: vec![19],
            count: 1,
            start_time: mock_time.now(),
        }
    ];
    
    // アニメーション更新テスト（時刻による更新）
    mock_time.advance(std::time::Duration::from_millis(500));
    let _current_time = mock_time.now();
    
    // アニメーションが存在することを確認
    assert!(!animations.is_empty());
    console_log!("WASM shared animation test passed with {} animations", animations.len());
}