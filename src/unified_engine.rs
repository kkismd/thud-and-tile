//! 統一ゲームエンジン: CLI版とWeb版で共通のコアロジック
//! 
//! sleep依存を削除し、イベント駆動型のゲームループを提供します。
//! タイミング制御は外部のスケジューラーに委譲し、ゲームロジックは純粋関数として実装。

use std::time::Duration;
use crate::unified_scheduler::{GameEvent, UnifiedScheduler, TimeProvider};

/// 統一ゲームエンジンのコアトレイト
pub trait UnifiedGameEngine {
    /// フレーム更新処理（睡眠なし、イベント駆動）
    fn update_frame(&mut self, delta_time: Duration, events: Vec<GameEvent>) -> Vec<GameEvent>;
    
    /// 入力処理
    fn handle_input(&mut self, input: crate::game_input::GameInput) -> Vec<GameEvent>;
    
    /// 現在のゲーム状態を取得
    fn get_game_state(&self) -> &dyn GameStateAccess;
    
    /// 描画が必要かどうか
    fn needs_render(&self) -> bool;
    
    /// 描画完了を通知
    fn render_complete(&mut self);
}

/// ゲーム状態へのアクセスインターフェース
pub trait GameStateAccess {
    fn get_game_mode(&self) -> u8;
    fn get_board_state(&self) -> Vec<u8>;
    fn get_current_piece_info(&self) -> Vec<i32>;
    fn get_score(&self) -> u32;
    fn has_animation(&self) -> bool;
}

/// CLI版・Web版共通のゲームコントローラー
pub struct UnifiedGameController {
    engine: Box<dyn UnifiedGameEngine>,
    scheduler: UnifiedScheduler,
    time_provider: Box<dyn TimeProvider>,
    last_update: Duration,
    needs_render: bool,
    
    // タイマーID管理
    auto_fall_timer_id: Option<u32>,
    render_timer_id: Option<u32>,
    animation_timer_id: Option<u32>,
}

impl UnifiedGameController {
    pub fn new(engine: Box<dyn UnifiedGameEngine>, time_provider: Box<dyn TimeProvider>) -> Self {
        let mut scheduler = UnifiedScheduler::new();
        let current_time = time_provider.now();
        
        // 基本タイマーを設定
        let render_timer_id = scheduler.set_render_timer();
        
        Self {
            engine,
            scheduler,
            time_provider,
            last_update: current_time,
            needs_render: false,
            auto_fall_timer_id: None,
            render_timer_id: Some(render_timer_id),
            animation_timer_id: None,
        }
    }
    
    /// メインアップデート（CLI・Web共通）
    pub fn update(&mut self) -> UpdateResult {
        let current_time = self.time_provider.now();
        let delta_time = current_time - self.last_update;
        self.last_update = current_time;
        
        // スケジューラーからイベントを取得
        let events = self.scheduler.update(delta_time);
        
        // ゲームエンジンにイベントを送信
        let new_events = self.engine.update_frame(delta_time, events);
        
        // 新しいイベントを処理
        for event in new_events {
            self.handle_engine_event(event);
        }
        
        // 描画判定
        self.needs_render = self.engine.needs_render();
        
        UpdateResult {
            needs_render: self.needs_render,
            game_mode: self.engine.get_game_state().get_game_mode(),
        }
    }
    
    /// 入力処理
    pub fn handle_input(&mut self, input: crate::game_input::GameInput) {
        let events = self.engine.handle_input(input);
        for event in events {
            self.handle_engine_event(event);
        }
    }
    
    /// ゲーム開始時の初期化
    pub fn start_game(&mut self, fall_speed: Duration) {
        // 自動落下タイマーを設定
        if let Some(id) = self.auto_fall_timer_id {
            self.scheduler.remove_timer(id);
        }
        self.auto_fall_timer_id = Some(self.scheduler.set_auto_fall_timer(fall_speed));
        
        // アニメーションタイマーを設定
        if self.animation_timer_id.is_none() {
            self.animation_timer_id = Some(self.scheduler.set_animation_timer());
        }
    }
    
    /// 描画完了通知
    pub fn render_complete(&mut self) {
        self.engine.render_complete();
        self.needs_render = false;
    }
    
    /// ゲーム状態アクセス
    pub fn get_game_state(&self) -> &dyn GameStateAccess {
        self.engine.get_game_state()
    }
    
    /// エンジンイベントの処理
    fn handle_engine_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::GameOver => {
                // 自動落下タイマーを停止
                if let Some(id) = self.auto_fall_timer_id {
                    self.scheduler.remove_timer(id);
                    self.auto_fall_timer_id = None;
                }
            }
            GameEvent::ScoreUpdate => {
                self.needs_render = true;
            }
            _ => {}
        }
    }
}

/// アップデート結果
pub struct UpdateResult {
    pub needs_render: bool,
    pub game_mode: u8,
}

/// CLI版用の実装例
pub mod cli_adapter {
    use super::*;
    use crate::game_core::{GameCore, GameMode};
    
    // CLI版のGameStateはGameCoreをラップする構造に将来変更予定
    pub struct CliGameEngine {
        core: GameCore,
        needs_render: bool,
    }
    
    impl CliGameEngine {
        pub fn new() -> Self {
            Self {
                core: GameCore::new(),
                needs_render: true,
            }
        }
    }
    
    impl UnifiedGameEngine for CliGameEngine {
        fn update_frame(&mut self, _delta_time: Duration, events: Vec<GameEvent>) -> Vec<GameEvent> {
            let mut result_events = Vec::new();
            
            for event in events {
                match event {
                    GameEvent::AutoFall => {
                        if !self.core.animation.is_empty() {
                            continue; // アニメーション中はスキップ
                        }
                        
                        if let Some(ref piece) = self.core.current_piece {
                            let moved_down = piece.moved(0, 1);
                            if self.core.is_valid_position(&moved_down) {
                                self.core.current_piece = Some(moved_down);
                            } else {
                                // TODO: lock_piece実装
                                result_events.push(GameEvent::PieceLock);
                            }
                        } else {
                            self.core.spawn_piece();
                        }
                        self.needs_render = true;
                    }
                    GameEvent::AnimationUpdate => {
                        if !self.core.animation.is_empty() {
                            // TODO: handle_animation実装
                            self.needs_render = true;
                        }
                    }
                    GameEvent::Render => {
                        // 描画イベント（実際の描画は外部で実行）
                        self.needs_render = true;
                    }
                    _ => {}
                }
            }
            
            result_events
        }
        
        fn handle_input(&mut self, input: crate::game_input::GameInput) -> Vec<GameEvent> {
            use crate::game_input::GameInput;
            
            match input {
                GameInput::Quit => {
                    match self.core.mode {
                        GameMode::Playing => vec![GameEvent::GameOver],
                        _ => vec![],
                    }
                }
                GameInput::Restart => {
                    self.core = GameCore::new();
                    self.core.mode = GameMode::Playing;
                    self.core.spawn_piece();
                    vec![]
                }
                _ => {
                    // TODO: input handling implementation
                    self.needs_render = true;
                    vec![]
                }
            }
        }
        
        fn get_game_state(&self) -> &dyn GameStateAccess {
            &self.core
        }
        
        fn needs_render(&self) -> bool {
            self.needs_render
        }
        
        fn render_complete(&mut self) {
            self.needs_render = false;
        }
    }
    
    impl GameStateAccess for GameCore {
        fn get_game_mode(&self) -> u8 {
            self.get_game_mode_u8()
        }
        
        fn get_board_state(&self) -> Vec<u8> {
            self.get_board_state_u8()
        }
        
        fn get_current_piece_info(&self) -> Vec<i32> {
            // TODO: 現在ピース情報の変換実装
            vec![]
        }
        
        fn get_score(&self) -> u32 {
            self.get_total_score()
        }
        
        fn has_animation(&self) -> bool {
            self.has_animation()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_scheduler::{create_time_provider};
    
    #[test]
    fn test_unified_controller() {
        let engine = Box::new(cli_adapter::CliGameEngine::new());
        let time_provider = create_time_provider();
        let mut controller = UnifiedGameController::new(engine, time_provider);
        
        // ゲーム開始
        controller.start_game(Duration::from_millis(500));
        
        // アップデート
        let result = controller.update();
        assert_eq!(result.game_mode, 0); // Title mode
    }
}