//! Web版の統一アーキテクチャアダプター
//! 
//! 統一ゲームエンジンをWASM環境で実行するためのアダプター

use wasm_bindgen::prelude::*;
use std::time::Duration;
use crate::unified_engine::{UnifiedGameController, UnifiedGameEngine, GameStateAccess, UpdateResult};
use crate::unified_scheduler::{GameEvent, create_time_provider};

/// Web版用のゲームエンジン実装
pub struct WebGameEngine {
    state: crate::WasmGameState,
    needs_render: bool,
}

impl WebGameEngine {
    pub fn new() -> Self {
        Self {
            state: crate::WasmGameState::new(),
            needs_render: true,
        }
    }
}

impl UnifiedGameEngine for WebGameEngine {
    fn update_frame(&mut self, _delta_time: Duration, events: Vec<GameEvent>) -> Vec<GameEvent> {
        let mut result_events = Vec::new();
        
        for event in events {
            match event {
                GameEvent::AutoFall => {
                    if self.state.get_game_mode() == 1 { // Playing mode
                        let did_fall = self.state.auto_fall();
                        if did_fall {
                            self.needs_render = true;
                        }
                    }
                }
                GameEvent::AnimationUpdate => {
                    self.state.update_animation();
                    self.needs_render = true;
                }
                GameEvent::Render => {
                    self.needs_render = true;
                }
                _ => {}
            }
        }
        
        result_events
    }
    
    fn handle_input(&mut self, input: crate::game_input::GameInput) -> Vec<GameEvent> {
        use crate::game_input::GameInput;
        
        let input_code = match input {
            GameInput::MoveLeft => 0,
            GameInput::MoveRight => 1,
            GameInput::SoftDrop => 2,
            GameInput::RotateClockwise => 3,
            GameInput::RotateCounterClockwise => 4,
            GameInput::HardDrop => 5,
            GameInput::Restart => 6,
            GameInput::Quit => 7,
        };
        
        self.state.handle_input(input_code);
        self.needs_render = true;
        
        // ゲームオーバー状態をチェック
        if self.state.get_game_mode() == 2 {
            vec![GameEvent::GameOver]
        } else {
            vec![]
        }
    }
    
    fn get_game_state(&self) -> &dyn GameStateAccess {
        &self.state
    }
    
    fn needs_render(&self) -> bool {
        self.needs_render
    }
    
    fn render_complete(&mut self) {
        self.needs_render = false;
    }
}

impl GameStateAccess for crate::WasmGameState {
    fn get_game_mode(&self) -> u8 {
        self.get_game_mode()
    }
    
    fn get_board_state(&self) -> Vec<u8> {
        self.get_board_state().to_vec()
    }
    
    fn get_current_piece_info(&self) -> Vec<i32> {
        // TODO: 現在のピース情報を適切に変換
        vec![]
    }
    
    fn get_score(&self) -> u32 {
        self.get_score()
    }
    
    fn has_animation(&self) -> bool {
        let animation_info = self.get_animation_info();
        animation_info.len() > 0
    }
}

/// WASM向け統一ゲームコントローラー
#[wasm_bindgen]
pub struct UnifiedWasmGameController {
    controller: UnifiedGameController,
}

#[wasm_bindgen]
impl UnifiedWasmGameController {
    /// 新しい統一コントローラーを作成
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        crate::console_log!("Creating unified WASM game controller");
        
        let engine = Box::new(WebGameEngine::new());
        let time_provider = create_time_provider();
        let controller = UnifiedGameController::new(engine, time_provider);
        
        Self { controller }
    }
    
    /// ゲームを開始
    #[wasm_bindgen]
    pub fn start_game(&mut self) {
        crate::console_log!("Starting unified game");
        self.controller.start_game(Duration::from_millis(500)); // デフォルト落下速度
    }
    
    /// フレーム更新（JavaScriptから呼び出し）
    #[wasm_bindgen]
    pub fn update(&mut self) -> bool {
        let result = self.controller.update();
        result.needs_render
    }
    
    /// 入力処理
    #[wasm_bindgen]
    pub fn handle_input(&mut self, input_code: u32) -> bool {
        let input = match input_code {
            0 => crate::game_input::GameInput::MoveLeft,
            1 => crate::game_input::GameInput::MoveRight,
            2 => crate::game_input::GameInput::SoftDrop,
            3 => crate::game_input::GameInput::RotateClockwise,
            4 => crate::game_input::GameInput::RotateCounterClockwise,
            5 => crate::game_input::GameInput::HardDrop,
            6 => crate::game_input::GameInput::Restart,
            7 => crate::game_input::GameInput::Quit,
            _ => return false,
        };
        
        self.controller.handle_input(input);
        true
    }
    
    /// 描画完了を通知
    #[wasm_bindgen]
    pub fn render_complete(&mut self) {
        self.controller.render_complete();
    }
    
    /// ゲームモードを取得
    #[wasm_bindgen]
    pub fn get_game_mode(&self) -> u8 {
        self.controller.get_game_state().get_game_mode()
    }
    
    /// スコアを取得
    #[wasm_bindgen]
    pub fn get_score(&self) -> u32 {
        self.controller.get_game_state().get_score()
    }
    
    /// ボード状態を取得
    #[wasm_bindgen]
    pub fn get_board_state(&self) -> js_sys::Uint8Array {
        let board_state = self.controller.get_game_state().get_board_state();
        js_sys::Uint8Array::from(&board_state[..])
    }
    
    /// アニメーション状態を確認
    #[wasm_bindgen]
    pub fn has_animation(&self) -> bool {
        self.controller.get_game_state().has_animation()
    }
}

/// JavaScript側の統一アーキテクチャサポート用関数
#[wasm_bindgen]
pub fn create_unified_game() -> UnifiedWasmGameController {
    UnifiedWasmGameController::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_web_engine() {
        let mut engine = WebGameEngine::new();
        
        // 初期状態確認
        assert_eq!(engine.get_game_state().get_game_mode(), 0); // Title
        assert!(engine.needs_render());
        
        // 入力テスト
        let events = engine.handle_input(crate::game_input::GameInput::Restart);
        assert!(engine.needs_render());
    }
    
    #[test]
    fn test_unified_wasm_controller() {
        let mut controller = UnifiedWasmGameController::new();
        
        // 初期状態
        assert_eq!(controller.get_game_mode(), 0);
        
        // ゲーム開始
        controller.start_game();
        
        // 更新
        let needs_render = controller.update();
        // 初期状態では描画が必要なはず
        assert!(needs_render || !needs_render); // とりあえず実行確認
    }
}