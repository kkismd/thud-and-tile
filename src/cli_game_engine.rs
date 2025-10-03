//! CLI版ゲームエンジン実装

use std::time::Duration;
use crate::unified_engine::{UnifiedGameEngine, GameStateAccess};
use crate::unified_scheduler::GameEvent;
use crate::game_input::GameInput;
use crate::{GameState, GameMode, Animation};
use crate::unified_scheduler::TimeProvider;

/// CLI版ゲームエンジン
pub struct CliGameEngine {
    state: GameState,
    needs_render: bool,
    last_fall: Duration,
}

impl CliGameEngine {
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
            needs_render: true,
            last_fall: Duration::ZERO,
        }
    }
    
    pub fn get_state(&self) -> &GameState {
        &self.state
    }
    
    pub fn get_state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }
}

impl UnifiedGameEngine for CliGameEngine {
    fn update_frame(&mut self, _delta_time: Duration, events: Vec<GameEvent>) -> Vec<GameEvent> {
        let mut new_events = Vec::new();
        
        for event in events {
            match event {
                GameEvent::AutoFall => {
                    if let Some(piece) = &self.state.current_piece {
                        let moved_down = piece.moved(0, 1);
                        if self.state.is_valid_position(&moved_down) {
                            self.state.current_piece = Some(moved_down);
                        } else {
                            // ピースを固定し、タイマーを使って後処理
                            self.state.lock_piece(&crate::test_time_provider::ControllableTimeProvider::new());
                        }
                    } else {
                        self.state.spawn_piece();
                    }
                    self.needs_render = true;
                }
                GameEvent::AnimationUpdate => {
                    if !self.state.animation.is_empty() {
                        crate::handle_animation(&mut self.state, &crate::test_time_provider::ControllableTimeProvider::new());
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
                if self.state.mode == GameMode::Playing {
                    self.state.mode = GameMode::GameOver;
                } else {
                    // アプリケーション終了の指示をイベントで返す
                    return vec![GameEvent::ApplicationExit];
                }
            }
            GameInput::Restart => {
                match self.state.mode {
                    GameMode::Title => {
                        self.state = GameState::new();
                        self.state.mode = GameMode::Playing;
                        self.state.spawn_piece();
                        // 自動落下タイマーを開始
                        return vec![GameEvent::StartAutoFall];
                    }
                    GameMode::GameOver => {
                        self.state = GameState::new();
                        return vec![GameEvent::ShowTitle];
                    }
                    _ => {}
                }
            }
            _ => {
                if self.state.mode == GameMode::Playing {
                    self.state.handle_input(input);
                }
            }
        }
        
        self.needs_render = true;
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

impl GameStateAccess for CliGameEngine {
    fn get_game_mode(&self) -> u8 {
        match self.state.mode {
            GameMode::Title => 0,
            GameMode::Playing => 1,
            GameMode::GameOver => 2,
        }
    }
    
    fn get_board_state(&self) -> Vec<u8> {
        // 簡単な実装 - より詳細な実装は後で追加
        vec![]
    }
    
    fn get_current_piece_info(&self) -> Vec<i32> {
        // 簡単な実装 - より詳細な実装は後で追加
        vec![]
    }
    
    fn get_score(&self) -> u32 {
        // CustomScoreSystemから総スコアを計算
        0 // 簡単な実装 - より詳細な実装は後で追加
    }
    
    fn has_animation(&self) -> bool {
        !self.state.animation.is_empty()
    }
}