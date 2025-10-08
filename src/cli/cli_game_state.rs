//! CLI専用ゲーム状態管理
//! 
//! Layer 1 (core) の共通ロジックを使用したCLI特化実装
//! 時間管理、描画状態、入力処理などCLI固有の機能を追加

use crate::core::game_state::{CoreGameState, CoreGameEvent};
use crate::core::animation_logic::AnimationType;
use crate::game_input::GameInput;
use std::time::{Duration, Instant};

/// CLI版時間管理
#[derive(Debug, Clone)]
pub struct CliTimeProvider {
    start_time: Instant,
    last_frame_time: Instant,
}

impl CliTimeProvider {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame_time: now,
        }
    }
    
    /// 現在時刻をミリ秒で取得
    pub fn now_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
    
    /// フレーム間隔を更新
    pub fn update_frame_time(&mut self) -> Duration {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        delta
    }
}

/// CLI版描画状態
#[derive(Debug, Clone)]
pub struct CliRenderState {
    pub needs_full_redraw: bool,
    pub needs_board_redraw: bool,
    pub needs_ui_redraw: bool,
    pub active_animations: Vec<AnimationType>,
}

impl Default for CliRenderState {
    fn default() -> Self {
        Self {
            needs_full_redraw: true,
            needs_board_redraw: false,
            needs_ui_redraw: false,
            active_animations: Vec::new(),
        }
    }
}

/// CLI版ゲーム状態ラッパー
/// Layer 1のCoreGameStateを内包し、CLI特化機能を追加
#[derive(Debug, Clone)]
pub struct CliGameState {
    /// Layer 1: 共通ゲーム状態
    pub core: CoreGameState,
    
    /// CLI特化: 時間管理
    pub time_provider: CliTimeProvider,
    
    /// CLI特化: 描画状態管理
    pub render_state: CliRenderState,
    
    /// CLI特化: パフォーマンス情報
    pub frame_count: u64,
    pub last_fps: f64,
    
    /// CLI特化: 入力イベント統計
    pub input_event_count: u64,
}

impl CliGameState {
    /// 新しいCLIゲーム状態を作成
    pub fn new() -> Self {
        Self {
            core: CoreGameState::new(),
            time_provider: CliTimeProvider::new(),
            render_state: CliRenderState::default(),
            frame_count: 0,
            last_fps: 0.0,
            input_event_count: 0,
        }
    }
    
    /// Layer 1を使用したアニメーション更新
    pub fn update_animations(&mut self) -> Vec<CoreGameEvent> {
        let current_time_ms = self.time_provider.now_ms();
        
        // Layer 1の純粋関数を使用（所有権移動を避けるためにclone）
        let update_result = self.core.clone().update_with_time(current_time_ms);
        self.core = update_result.new_state;
        
        // CLI特化: 描画状態更新
        self.update_render_state_from_events(&update_result.events);
        
        update_result.events
    }
    
    /// CLI特化: 入力処理
    pub fn handle_input(&mut self, input: GameInput) -> Vec<CoreGameEvent> {
        let current_time_ms = self.time_provider.now_ms();
        
        // Layer 1の入力処理を使用（所有権移動を避けるためにclone）
        use crate::core::input_handler::process_input;
        let input_result = process_input(self.core.clone(), input, current_time_ms);
        
        self.core = input_result.new_state;
        
        // CLI特化: 描画状態更新
        self.update_render_state_from_events(&input_result.events);
        
        input_result.events
    }
    
    /// CLI特化: フレーム更新
    pub fn update_frame(&mut self) -> Duration {
        self.frame_count += 1;
        let delta = self.time_provider.update_frame_time();
        
        // FPS計算（1秒ごと）
        if self.frame_count % 60 == 0 {
            self.last_fps = 1000.0 / delta.as_millis() as f64;
        }
        
        delta
    }
    
    /// 描画状態を更新
    fn update_render_state_from_events(&mut self, events: &[CoreGameEvent]) {
        for event in events {
            match event {
                CoreGameEvent::AnimationStarted { animation_type } => {
                    self.render_state.active_animations.push(*animation_type);
                    self.render_state.needs_board_redraw = true;
                }
                CoreGameEvent::AnimationCompleted { .. } => {
                    self.render_state.needs_board_redraw = true;
                }
                CoreGameEvent::LinesCleared { .. } => {
                    self.render_state.needs_full_redraw = true;
                }
                CoreGameEvent::ScoreUpdated { .. } => {
                    self.render_state.needs_ui_redraw = true;
                }
                CoreGameEvent::GameModeChanged { .. } => {
                    self.render_state.needs_full_redraw = true;
                }
                _ => {}
            }
        }
    }
    
    /// 描画が必要かチェック
    pub fn needs_redraw(&self) -> bool {
        self.render_state.needs_full_redraw 
            || self.render_state.needs_board_redraw 
            || self.render_state.needs_ui_redraw
            || self.core.has_animations()
    }
    
    /// 描画状態をリセット
    pub fn mark_rendered(&mut self) {
        self.render_state.needs_full_redraw = false;
        self.render_state.needs_board_redraw = false;
        self.render_state.needs_ui_redraw = false;
    }
    
    /// ゲームイベント処理（main_phase2a用）
    pub fn process_game_event(&mut self, _event: CoreGameEvent) {
        self.input_event_count += 1;
        self.render_state.needs_full_redraw = true;
    }
}

impl Default for CliGameState {
    fn default() -> Self {
        Self::new()
    }
}