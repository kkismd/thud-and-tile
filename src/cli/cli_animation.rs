//! CLI専用アニメーション管理
//! 
//! Layer 1のアニメーションロジックを使用したCLI特化アニメーション処理
//! 描画同期、タイミング調整、フレームレート管理を含む

use crate::core::animation_logic::{AnimationState, AnimationType, should_render_line_with_animation};
use crate::core::game_state::CoreGameState;
use std::time::Duration;

/// CLI版アニメーション管理
pub struct CliAnimationManager {
    /// フレームレート管理
    target_fps: u32,
    frame_duration: Duration,
    
    /// アニメーション描画キャッシュ
    last_render_time: u64,
    cached_visible_lines: Vec<bool>,
}

impl CliAnimationManager {
    /// 新しいアニメーション管理を作成
    pub fn new(target_fps: u32) -> Self {
        Self {
            target_fps,
            frame_duration: Duration::from_millis(1000 / target_fps as u64),
            last_render_time: 0,
            cached_visible_lines: vec![true; 20], // BOARD_HEIGHT
        }
    }
    
    /// デフォルト60FPSでアニメーション管理作成
    pub fn new_default() -> Self {
        Self::new(60)
    }
    
    /// CLI特化: ライン描画判定（キャッシュ付き）
    pub fn should_render_line(&mut self, line_y: usize, core_state: &CoreGameState, current_time_ms: u64) -> bool {
        // キャッシュ無効化チェック
        if current_time_ms != self.last_render_time {
            self.update_visibility_cache(core_state, current_time_ms);
            self.last_render_time = current_time_ms;
        }
        
        self.cached_visible_lines.get(line_y).copied().unwrap_or(true)
    }
    
    /// CLI特化: フレームレート調整が必要かチェック
    pub fn should_limit_framerate(&self, frame_delta: Duration) -> bool {
        frame_delta < self.frame_duration
    }
    
    /// CLI特化: フレームレート調整のためのスリープ時間計算
    pub fn calculate_sleep_duration(&self, frame_delta: Duration) -> Option<Duration> {
        if frame_delta < self.frame_duration {
            Some(self.frame_duration - frame_delta)
        } else {
            None
        }
    }
    
    /// アクティブなアニメーション一覧取得
    pub fn get_active_animations(&self, core_state: &CoreGameState) -> Vec<AnimationType> {
        let mut active_types = Vec::new();
        
        for i in 0..core_state.animations_count {
            let animation = &core_state.animations[i];
            if !animation.is_completed {
                if !active_types.contains(&animation.animation_type) {
                    active_types.push(animation.animation_type);
                }
            }
        }
        
        active_types
    }
    
    /// CLI特化: アニメーション効果の強度計算（描画用）
    pub fn get_animation_intensity(&self, animation_type: AnimationType, current_time_ms: u64) -> f32 {
        match animation_type {
            AnimationType::LineBlink => {
                // 点滅の強度を計算
                let blink_cycle = 240; // ms
                let phase = (current_time_ms % blink_cycle) as f32 / blink_cycle as f32;
                (phase * std::f32::consts::PI * 2.0).sin().abs()
            }
            AnimationType::PushDown => {
                // 押し下げの強度を固定
                0.8
            }
            AnimationType::EraseLine => {
                // 消去の強度を固定
                1.0
            }
        }
    }
    
    /// 描画可視性キャッシュ更新
    fn update_visibility_cache(&mut self, core_state: &CoreGameState, current_time_ms: u64) {
        // 全ライン初期化
        self.cached_visible_lines.fill(true);
        
        // Layer 1の関数を使用してアニメーション影響を計算
        let active_animations: Vec<AnimationState> = core_state.animations[..core_state.animations_count].to_vec();
        
        for line_y in 0..self.cached_visible_lines.len() {
            self.cached_visible_lines[line_y] = should_render_line_with_animation(
                line_y,
                &active_animations,
                current_time_ms,
            );
        }
    }
    
    /// パフォーマンス統計取得
    pub fn get_performance_stats(&self) -> CliAnimationStats {
        CliAnimationStats {
            target_fps: self.target_fps,
            frame_duration_ms: self.frame_duration.as_millis() as u32,
        }
    }
}

/// CLI版アニメーション統計情報
#[derive(Debug, Clone)]
pub struct CliAnimationStats {
    pub target_fps: u32,
    pub frame_duration_ms: u32,
}

impl Default for CliAnimationManager {
    fn default() -> Self {
        Self::new_default()
    }
}