//! WASM版ゲームエンジン実装

use std::time::Duration;
use crate::unified_engine::{UnifiedGameEngine, GameStateAccess};
use crate::unified_scheduler::GameEvent;
use crate::game_input::GameInput;
use crate::{Cell, BOARD_WIDTH, BOARD_HEIGHT};
use crate::cell::Board;
use crate::game_color::GameColor;

/// WASM版用簡易Tetromino
#[derive(Clone, Debug)]
pub struct SimpleTetromino {
    pub x: i32,
    pub y: i32,
    pub shape: u8,
    pub rotation_state: u8,
    pub color: GameColor,
}

/// WASM版ゲームエンジン
pub struct WasmGameEngine {
    board: Board,
    current_piece: Option<SimpleTetromino>,
    next_piece: Option<SimpleTetromino>,
    game_mode: u8, // 0: Title, 1: Playing, 2: GameOver
    needs_render: bool,
    score: u32,
    animation: Vec<crate::Animation>,
}

impl WasmGameEngine {
    pub fn new() -> Self {
        Self {
            board: vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: None,
            next_piece: None,
            game_mode: 0, // Title
            needs_render: true,
            score: 0,
            animation: Vec::new(),
        }
    }
    
    pub fn get_board(&self) -> &Board {
        &self.board
    }
    
    pub fn get_current_piece(&self) -> &Option<SimpleTetromino> {
        &self.current_piece
    }
    
    pub fn set_game_mode(&mut self, mode: u8) {
        self.game_mode = mode;
        self.needs_render = true;
    }
    
    fn spawn_piece(&mut self) {
        // 簡単な実装 - より詳細な実装は後で追加
        self.current_piece = Some(SimpleTetromino {
            x: 4,
            y: 0,
            shape: 0, // I-piece
            rotation_state: 0,
            color: GameColor::Red,
        });
        self.needs_render = true;
    }
}

impl UnifiedGameEngine for WasmGameEngine {
    fn update_frame(&mut self, _delta_time: Duration, events: Vec<GameEvent>) -> Vec<GameEvent> {
        let mut new_events = Vec::new();
        
        for event in events {
            match event {
                GameEvent::AutoFall => {
                    if self.current_piece.is_some() {
                        // 簡単な落下処理
                        if let Some(ref mut piece) = self.current_piece {
                            piece.y += 1;
                            if piece.y > 20 {
                                // 底に到達したらピースを固定
                                self.current_piece = None;
                                self.spawn_piece();
                            }
                        }
                    } else {
                        self.spawn_piece();
                    }
                    self.needs_render = true;
                }
                GameEvent::AnimationUpdate => {
                    if !self.animation.is_empty() {
                        // アニメーション処理の簡単な実装
                        self.animation.clear();
                        self.needs_render = true;
                    }
                }
                GameEvent::Render => {
                    self.needs_render = true;
                }
                _ => {}
            }
        }
        
        new_events
    }
    
    fn handle_input(&mut self, input: GameInput) -> Vec<GameEvent> {
        match input {
            GameInput::Quit => {
                return vec![GameEvent::ApplicationExit];
            }
            GameInput::Restart => {
                match self.game_mode {
                    0 => { // Title
                        self.set_game_mode(1); // Playing
                        self.spawn_piece();
                        return vec![GameEvent::StartAutoFall];
                    }
                    2 => { // GameOver
                        self.set_game_mode(0); // Title
                        return vec![GameEvent::ShowTitle];
                    }
                    _ => {}
                }
            }
            GameInput::MoveLeft => {
                if let Some(ref mut piece) = self.current_piece {
                    piece.x -= 1;
                    self.needs_render = true;
                }
            }
            GameInput::MoveRight => {
                if let Some(ref mut piece) = self.current_piece {
                    piece.x += 1;
                    self.needs_render = true;
                }
            }
            GameInput::SoftDrop => {
                if let Some(ref mut piece) = self.current_piece {
                    piece.y += 1;
                    self.needs_render = true;
                }
            }
            _ => {}
        }
        
        Vec::new()
    }
    
    fn get_game_state(&self) -> &dyn GameStateAccess {
        self
    }
    
    fn needs_render(&self) -> bool {
        self.needs_render
    }
    
    fn render_complete(&mut self) {
        self.needs_render = false;
    }
}

impl GameStateAccess for WasmGameEngine {
    fn get_game_mode(&self) -> u8 {
        self.game_mode
    }
    
    fn get_board_state(&self) -> Vec<u8> {
        let mut board_state = Vec::new();
        for row in &self.board {
            for cell in row {
                match cell {
                    Cell::Empty => board_state.push(0),
                    Cell::Occupied(color) => board_state.push(color.to_u8()),
                    _ => board_state.push(0),
                }
            }
        }
        board_state
    }
    
    fn get_current_piece_info(&self) -> Vec<i32> {
        if let Some(ref piece) = self.current_piece {
            vec![piece.x, piece.y, piece.shape as i32, piece.rotation_state as i32]
        } else {
            vec![]
        }
    }
    
    fn get_current_piece_blocks(&self) -> Vec<(i32, i32, u8)> {
        // WASM版では簡単な実装
        if let Some(ref piece) = self.current_piece {
            vec![(piece.x, piece.y, piece.color.to_u8())]
        } else {
            vec![]
        }
    }
    
    fn get_ghost_piece_blocks(&self) -> Vec<(i32, i32)> {
        // WASM版では簡単な実装
        vec![]
    }
    
    fn get_score(&self) -> u32 {
        self.score
    }
    
    fn has_animation(&self) -> bool {
        !self.animation.is_empty()
    }
}