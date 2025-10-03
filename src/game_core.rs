//! 共通ゲームコア: CLI版とWASM版で共有するゲーム状態とロジック
//! 
//! このモジュールは、プラットフォーム固有の機能（WASM bindings、CLI入力処理）を
//! 除外し、純粋なゲームロジックのみを提供します。

use std::time::Duration;
use crate::config::*;
use crate::cell::{Board, Cell};
use crate::game_color::GameColor;
use crate::scoring::CustomScoreSystem;
use crate::animation::Animation;
use crate::tetromino::Tetromino;
use crate::board_logic;

/// ゲームモード（CLI版とWASM版で共通）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameMode {
    Title,
    Playing,
    GameOver,
}

/// プラットフォーム独立なゲームコア状態
/// CLI版のGameStateとWASM版のWasmGameStateの共通部分を抽出
#[derive(Clone, Debug, PartialEq)]
pub struct GameCore {
    /// ゲームモード
    pub mode: GameMode,
    /// ゲームボード
    pub board: Board,
    /// 現在のテトロミノ
    pub current_piece: Option<Tetromino>,
    /// 次のテトロミノ  
    pub next_piece: Option<Tetromino>,
    /// アニメーション状態
    pub animation: Vec<Animation>,
    /// 消去したライン数
    pub lines_cleared: u32,
    /// 落下速度
    pub fall_speed: Duration,
    /// 現在のボード高さ（動的制御）
    pub current_board_height: usize,
    /// カスタムスコアリングシステム
    pub custom_score_system: CustomScoreSystem,
}

impl GameCore {
    /// 新しいゲームコアを作成
    pub fn new() -> Self {
        Self {
            mode: GameMode::Title,
            board: vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: None,
            next_piece: Some(Tetromino::new_random()),
            animation: Vec::new(),
            lines_cleared: 0,
            fall_speed: FALL_SPEED_START,
            current_board_height: BOARD_HEIGHT,
            custom_score_system: CustomScoreSystem::new(),
        }
    }

    /// ゲームを開始状態にリセット
    pub fn start_game(&mut self) {
        self.mode = GameMode::Playing;
        self.board = vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        self.fall_speed = FALL_SPEED_START;
        self.current_board_height = BOARD_HEIGHT;
        self.animation.clear();
        self.lines_cleared = 0;
        self.custom_score_system = CustomScoreSystem::new();
        
        // CLI版と同じピース初期化ロジック
        self.current_piece = None;
        self.next_piece = Some(Tetromino::new_random());
        self.spawn_piece();
    }

    /// ゴーストピースを計算（落下予測位置）
    pub fn ghost_piece(&self) -> Option<Tetromino> {
        self.current_piece.as_ref().map(|piece| {
            let mut ghost = piece.clone();
            while self.is_valid_position(&ghost.moved(0, 1)) {
                ghost = ghost.moved(0, 1);
            }
            ghost
        })
    }

    /// 新しいピースをスポーン
    pub fn spawn_piece(&mut self) {
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

    /// 指定位置が有効かチェック
    pub fn is_valid_position(&self, piece: &Tetromino) -> bool {
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

    /// ピースをボードに固定（TimeProviderは外部から注入）
    pub fn lock_piece<T: TimeProvider>(&mut self, time_provider: &T) {
        if let Some(piece) = self.current_piece.take() {
            for ((x, y), color) in piece.iter_blocks() {
                if y >= 0 && y < BOARD_HEIGHT as i8 {
                    self.board[y as usize][x as usize] = Cell::Occupied(color);
                }
            }
        }

        // 隣接色スキャンロジック
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
        lines_to_clear.sort_by(|a, b| b.cmp(a));

        board_logic::find_and_connect_adjacent_blocks(&mut self.board, &lines_to_clear);
        self.update_connected_block_counts();
        self.update_max_chains();

        // Calculate scores for lines to be cleared
        for &line_y in &lines_to_clear {
            let scores = crate::animation::calculate_line_clear_score(&self.board, line_y, &self.custom_score_system.max_chains);
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

    /// 接続ブロック数を更新
    pub fn update_connected_block_counts(&mut self) {
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

    /// 最大チェーン数を更新
    pub fn update_max_chains(&mut self) {
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

    /// ゲームモードを数値で取得（WASM binding互換）
    pub fn get_game_mode_u8(&self) -> u8 {
        match self.mode {
            GameMode::Title => 0,
            GameMode::Playing => 1,
            GameMode::GameOver => 2,
        }
    }

    /// ボード状態を数値配列で取得（WASM binding互換）
    pub fn get_board_state_u8(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for row in &self.board {
            for cell in row {
                let cell_value = match cell {
                    Cell::Empty => 0,
                    Cell::Occupied(color) => Self::color_to_u8(*color),
                    Cell::Connected { color, count: _ } => Self::color_to_u8(*color),
                    _ => 0,
                };
                result.push(cell_value);
            }
        }
        result
    }

    /// GameColorを数値に変換
    fn color_to_u8(color: GameColor) -> u8 {
        match color {
            GameColor::Cyan => 1,
            GameColor::Magenta => 2,
            GameColor::Yellow => 3,
            _ => 4,
        }
    }

    /// アニメーションが実行中かチェック
    pub fn has_animation(&self) -> bool {
        !self.animation.is_empty()
    }

    /// 合計スコアを取得
    pub fn get_total_score(&self) -> u32 {
        self.custom_score_system.scores.total()
    }
}

impl Default for GameCore {
    fn default() -> Self {
        Self::new()
    }
}

/// TimeProviderトレイトの一時的な定義（統一アーキテクチャ移行まで）
pub trait TimeProvider {
    fn now(&self) -> Duration;
}