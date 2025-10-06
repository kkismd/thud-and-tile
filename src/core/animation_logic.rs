//! Core Animation Logic - Pure Functions
//! 
//! CLI版とWASM版で共有するアニメーション処理の純粋関数群
//! 借用チェッカー競合を完全に回避するため、データコピーパターンを使用

use crate::cell::Cell;
use crate::config::{
    BLINK_ANIMATION_STEP, BLINK_COUNT_MAX, BOARD_HEIGHT, BOARD_WIDTH, PUSH_DOWN_STEP_DURATION,
};
use crate::game_color::GameColor;
use std::time::Duration;

/// アニメーション状態（コピー可能な固定データ）
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationState {
    pub animation_type: AnimationType,
    pub start_time_ms: u64,
    pub lines: Vec<usize>, // 対象ライン番号
    pub current_step: usize,
    pub is_completed: bool,
    pub metadata: AnimationMetadata,
}

/// アニメーション種別
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationType {
    LineBlink,
    PushDown,
    EraseLine,
}

/// アニメーション追加情報
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationMetadata {
    pub gray_line_y: Option<usize>, // PushDown用
    pub chain_bonus_consumed: usize, // EraseLine用
}

impl Default for AnimationMetadata {
    fn default() -> Self {
        Self {
            gray_line_y: None,
            chain_bonus_consumed: 0,
        }
    }
}

/// LineBlink状態更新結果
#[derive(Debug, Clone, PartialEq)]
pub struct LineBlinkUpdateResult {
    pub is_completed: bool,
    pub current_count: usize,
    pub is_visible: bool, // 現在の点滅状態
}

/// PushDown状態更新結果
#[derive(Debug, Clone, PartialEq)]
pub struct PushDownUpdateResult {
    pub is_completed: bool,
    pub should_move: bool,
}

/// EraseLine状態更新結果
#[derive(Debug, Clone, PartialEq)]
pub struct EraseLineUpdateResult {
    pub is_completed: bool,
    pub lines_erased: u32,
    pub should_process_step: bool,
}

/// アニメーション状態更新結果（全体）
#[derive(Debug, Clone)]
pub struct AnimationUpdateResult {
    pub updated_animations: Vec<AnimationState>,
    pub completed_line_blinks: Vec<Vec<usize>>,
    pub completed_push_downs: Vec<usize>,
    pub completed_erase_lines: Vec<(Vec<usize>, u32)>, // (lines, erased_count)
}

/// 【純粋関数】LineBlink状態の更新
/// 既存実装から抽出された核心ロジック
pub fn update_line_blink_state(
    lines: Vec<usize>,
    start_time_ms: u64,
    current_time_ms: u64,
) -> LineBlinkUpdateResult {
    let elapsed_ms = current_time_ms.saturating_sub(start_time_ms);
    let blink_step_ms = BLINK_ANIMATION_STEP.as_millis() as u64;
    let max_count = BLINK_COUNT_MAX;
    
    let count = (elapsed_ms / blink_step_ms) as usize;
    
    LineBlinkUpdateResult {
        is_completed: count >= max_count,
        current_count: count,
        is_visible: (count % 2) == 0,
    }
}

/// 【純粋関数】PushDown状態の更新
pub fn update_push_down_state(
    gray_line_y: usize,
    start_time_ms: u64,
    current_time_ms: u64,
) -> PushDownUpdateResult {
    let elapsed_ms = current_time_ms.saturating_sub(start_time_ms);
    let step_duration_ms = PUSH_DOWN_STEP_DURATION.as_millis() as u64;
    
    PushDownUpdateResult {
        is_completed: false, // 具体的な完了判定は board state 必要
        should_move: elapsed_ms >= step_duration_ms,
    }
}

/// 【純粋関数】EraseLine状態の更新
pub fn update_erase_line_state(
    target_lines: Vec<usize>,
    current_step: usize,
    start_time_ms: u64,
    current_time_ms: u64,
) -> EraseLineUpdateResult {
    let elapsed_ms = current_time_ms.saturating_sub(start_time_ms);
    let erase_interval_ms = 120; // 120ミリ秒ごとに1ライン消去
    
    let expected_step = (elapsed_ms / erase_interval_ms) as usize;
    let should_process_step = expected_step > current_step;
    let is_completed = current_step >= target_lines.len();
    
    EraseLineUpdateResult {
        is_completed,
        lines_erased: current_step as u32,
        should_process_step,
    }
}

/// 【純粋関数】複数アニメーション状態の一括更新
/// CLI版とWASM版で共通使用
pub fn update_all_animation_states(
    animations: Vec<AnimationState>,
    current_time_ms: u64,
) -> AnimationUpdateResult {
    let mut result = AnimationUpdateResult {
        updated_animations: Vec::new(),
        completed_line_blinks: Vec::new(),
        completed_push_downs: Vec::new(),
        completed_erase_lines: Vec::new(),
    };
    
    for animation in animations {
        match animation.animation_type {
            AnimationType::LineBlink => {
                let update_result = update_line_blink_state(
                    animation.lines.clone(),
                    animation.start_time_ms,
                    current_time_ms,
                );
                
                if update_result.is_completed {
                    result.completed_line_blinks.push(animation.lines);
                } else {
                    let mut updated_animation = animation;
                    updated_animation.current_step = update_result.current_count;
                    result.updated_animations.push(updated_animation);
                }
            }
            
            AnimationType::PushDown => {
                let update_result = update_push_down_state(
                    animation.metadata.gray_line_y.unwrap_or(0),
                    animation.start_time_ms,
                    current_time_ms,
                );
                
                if update_result.should_move {
                    // PushDownの完了判定はboard stateが必要なので外部処理
                    result.completed_push_downs.push(
                        animation.metadata.gray_line_y.unwrap_or(0)
                    );
                } else {
                    result.updated_animations.push(animation);
                }
            }
            
            AnimationType::EraseLine => {
                let update_result = update_erase_line_state(
                    animation.lines.clone(),
                    animation.current_step,
                    animation.start_time_ms,
                    current_time_ms,
                );
                
                if update_result.is_completed {
                    result.completed_erase_lines.push((
                        animation.lines,
                        update_result.lines_erased,
                    ));
                } else if update_result.should_process_step {
                    let mut updated_animation = animation;
                    updated_animation.current_step += 1;
                    result.updated_animations.push(updated_animation);
                } else {
                    result.updated_animations.push(animation);
                }
            }
        }
    }
    
    result
}

/// 【純粋関数】アニメーション状態を作成
pub fn create_line_blink_animation(
    lines: Vec<usize>,
    start_time_ms: u64,
) -> AnimationState {
    AnimationState {
        animation_type: AnimationType::LineBlink,
        start_time_ms,
        lines,
        current_step: 0,
        is_completed: false,
        metadata: AnimationMetadata::default(),
    }
}

pub fn create_push_down_animation(
    gray_line_y: usize,
    start_time_ms: u64,
) -> AnimationState {
    AnimationState {
        animation_type: AnimationType::PushDown,
        start_time_ms,
        lines: vec![gray_line_y],
        current_step: 0,
        is_completed: false,
        metadata: AnimationMetadata {
            gray_line_y: Some(gray_line_y),
            chain_bonus_consumed: 0,
        },
    }
}

pub fn create_erase_line_animation(
    target_lines: Vec<usize>,
    start_time_ms: u64,
) -> AnimationState {
    AnimationState {
        animation_type: AnimationType::EraseLine,
        start_time_ms,
        lines: target_lines,
        current_step: 0,
        is_completed: false,
        metadata: AnimationMetadata::default(),
    }
}

/// 【純粋関数】アニメーション表示判定
/// LineBlink描画用のヘルパー関数
pub fn should_render_line_with_animation(
    line_y: usize,
    animations: &[AnimationState],
    current_time_ms: u64,
) -> bool {
    for animation in animations {
        if animation.animation_type == AnimationType::LineBlink 
            && animation.lines.contains(&line_y) 
        {
            let blink_result = update_line_blink_state(
                animation.lines.clone(),
                animation.start_time_ms,
                current_time_ms,
            );
            return blink_result.is_visible;
        }
    }
    true // アニメーション対象外は常に表示
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_blink_update() {
        let lines = vec![5, 10, 15];
        let start_time = 0;
        
        // 初期状態
        let result = update_line_blink_state(lines.clone(), start_time, 0);
        assert!(!result.is_completed);
        assert_eq!(result.current_count, 0);
        assert!(result.is_visible);
        
        // 1回点滅後
        let blink_duration = BLINK_ANIMATION_STEP.as_millis() as u64;
        let result = update_line_blink_state(lines.clone(), start_time, blink_duration);
        assert!(!result.is_completed);
        assert_eq!(result.current_count, 1);
        assert!(!result.is_visible);
        
        // 完了まで
        let total_duration = (BLINK_COUNT_MAX as u64) * blink_duration;
        let result = update_line_blink_state(lines, start_time, total_duration);
        assert!(result.is_completed);
    }
    
    #[test]
    fn test_animation_state_creation() {
        let lines = vec![1, 2, 3];
        let start_time = 1000;
        
        let animation = create_line_blink_animation(lines.clone(), start_time);
        assert_eq!(animation.animation_type, AnimationType::LineBlink);
        assert_eq!(animation.lines, lines);
        assert_eq!(animation.start_time_ms, start_time);
        assert!(!animation.is_completed);
    }
    
    #[test]
    fn test_multiple_animations_update() {
        let animations = vec![
            create_line_blink_animation(vec![5], 0),
            create_push_down_animation(10, 100),
        ];
        
        let result = update_all_animation_states(animations, 1000);
        
        // LineBlink は完了しているはず (1000ms >> 6 * 120ms)
        assert_eq!(result.completed_line_blinks.len(), 1);
        assert_eq!(result.completed_line_blinks[0], vec![5]);
        
        // PushDown は移動処理が必要
        assert_eq!(result.completed_push_downs.len(), 1);
        assert_eq!(result.completed_push_downs[0], 10);
    }
}