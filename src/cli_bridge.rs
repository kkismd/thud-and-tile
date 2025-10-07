// CLI integration bridge for Phase 1 Week 2
// 既存のCLI実装とcoreモジュールを統合するためのブリッジ

use crate::core::game_state::{CoreGameState, CoreGameMode, CoreGameEvent};
use crate::core::input_handler::process_input;
use crate::core::board_logic::FixedBoard;
use crate::game_input::GameInput;
use crate::cell::Cell;
use crate::tetromino::Tetromino;
use std::time::Duration;

// main.rsのGameModeとGameStateを参照するため
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CLIGameMode {
    Title,
    Playing,
    GameOver,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CLIGameState {
    pub mode: CLIGameMode,
    pub board: crate::cell::Board,
    pub current_piece: Option<Tetromino>,
    pub next_piece: Option<Tetromino>,
    pub animation: Vec<crate::animation::Animation>,
    pub lines_cleared: u32,
    pub fall_speed: Duration,
    pub current_board_height: usize,
    pub custom_score_system: crate::scoring::CustomScoreSystem,
    pub enable_erase_line: bool,
}

/// 既存のGameStateとCoreGameStateの変換ブリッジ
pub struct CLIBridge {
    pub core_state: CoreGameState,
    pub fall_speed: Duration,
    pub next_piece: Option<Tetromino>,
    pub enable_erase_line: bool,
}

impl CLIBridge {
    pub fn new() -> Self {
        Self {
            core_state: CoreGameState::new(),
            fall_speed: crate::config::FALL_SPEED_START,
            next_piece: Some(Tetromino::new_random()),
            enable_erase_line: false,
        }
    }

    /// 既存のBoard形式からFixedBoard形式への変換
    pub fn convert_board_to_fixed(board: &crate::cell::Board) -> FixedBoard {
        let mut fixed_board = [[Cell::Empty; crate::config::BOARD_WIDTH]; crate::config::BOARD_HEIGHT];
        for (y, row) in board.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                fixed_board[y][x] = cell;
            }
        }
        fixed_board
    }

    /// FixedBoard形式から既存のBoard形式への変換
    pub fn convert_fixed_to_board(fixed_board: &FixedBoard) -> crate::cell::Board {
        fixed_board
            .iter()
            .map(|row| row.to_vec())
            .collect()
    }

    /// coreモジュールの入力処理を使用した入力ハンドリング
    pub fn handle_input_core(&mut self, input: GameInput, current_time_ms: u64) -> Vec<CoreGameEvent> {
        let result = process_input(self.core_state.clone(), input, current_time_ms);
        self.core_state = result.new_state;
        result.events
    }

    /// ゲームモードの変換（CLI ←→ Core）
    pub fn convert_mode_to_cli(core_mode: CoreGameMode) -> CLIGameMode {
        match core_mode {
            CoreGameMode::Title => CLIGameMode::Title,
            CoreGameMode::Playing => CLIGameMode::Playing,
            CoreGameMode::GameOver => CLIGameMode::GameOver,
        }
    }

    pub fn convert_mode_to_core(cli_mode: CLIGameMode) -> CoreGameMode {
        match cli_mode {
            CLIGameMode::Title => CoreGameMode::Title,
            CLIGameMode::Playing => CoreGameMode::Playing,
            CLIGameMode::GameOver => CoreGameMode::GameOver,
        }
    }

    /// CLI側のGameStateを更新するためのヘルパー
    pub fn sync_to_cli_state(&self, cli_state: &mut CLIGameState) {
        cli_state.mode = Self::convert_mode_to_cli(self.core_state.game_mode);
        cli_state.board = Self::convert_fixed_to_board(&self.core_state.board);
        cli_state.current_piece = self.core_state.current_piece.clone();
        cli_state.lines_cleared = self.core_state.lines_cleared;
        cli_state.current_board_height = self.core_state.current_board_height;
        cli_state.enable_erase_line = self.core_state.enable_erase_line; // core側の値を使用
        // chain_bonus同期（EraseLineアニメーション用）
        cli_state.custom_score_system.max_chains.chain_bonus = self.core_state.chain_bonus;
        // 既存のCLI固有のフィールド
        cli_state.next_piece = self.next_piece.clone();
        cli_state.fall_speed = self.fall_speed;
    }

    /// CLI側のGameStateからcore状態を同期
    pub fn sync_from_cli_state(&mut self, cli_state: &CLIGameState) {
        self.core_state.game_mode = Self::convert_mode_to_core(cli_state.mode);
        self.core_state.board = Self::convert_board_to_fixed(&cli_state.board);
        self.core_state.current_piece = cli_state.current_piece.clone();
        self.core_state.lines_cleared = cli_state.lines_cleared;
        self.core_state.current_board_height = cli_state.current_board_height;
        self.core_state.enable_erase_line = cli_state.enable_erase_line; // CLI側の値を使用
        // chain_bonus同期（EraseLineアニメーション用）
        self.core_state.chain_bonus = cli_state.custom_score_system.max_chains.chain_bonus;
        // CLI固有のフィールドを保持
        self.next_piece = cli_state.next_piece.clone();
        self.fall_speed = cli_state.fall_speed;
        self.enable_erase_line = cli_state.enable_erase_line;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_conversion() {
        let cli_board = vec![
            vec![Cell::Empty; crate::config::BOARD_WIDTH]; 
            crate::config::BOARD_HEIGHT
        ];
        let fixed_board = CLIBridge::convert_board_to_fixed(&cli_board);
        let converted_back = CLIBridge::convert_fixed_to_board(&fixed_board);
        assert_eq!(cli_board, converted_back);
    }

    #[test]
    fn test_mode_conversion() {
        let core_modes = [CoreGameMode::Title, CoreGameMode::Playing, CoreGameMode::GameOver];
        let cli_modes = [CLIGameMode::Title, CLIGameMode::Playing, CLIGameMode::GameOver];
        
        for (core, cli) in core_modes.iter().zip(cli_modes.iter()) {
            assert_eq!(*cli, CLIBridge::convert_mode_to_cli(*core));
            assert_eq!(*core, CLIBridge::convert_mode_to_core(*cli));
        }
    }

    #[test]
    fn test_bridge_initialization() {
        let bridge = CLIBridge::new();
        assert_eq!(bridge.core_state.game_mode, CoreGameMode::Title);
        assert!(bridge.next_piece.is_some());
        assert!(!bridge.enable_erase_line);
    }
}