//! Phase 9のEraseLineアニメーション統合テスト
//! update_animations関数でのEraseLine処理統合を検証

use crate::animation::{update_animations, Animation, AnimationResult};
use std::time::Duration;

#[test]
fn test_erase_line_animation_updates() {
    let mut animations = vec![Animation::EraseLine { 
        lines_remaining: 2,
        last_update: Duration::from_millis(0)
    }];
    let current_time = Duration::from_millis(120);
    
    let result = update_animations(&mut animations, current_time);
    
    // 1つのアニメーションが継続されることを確認
    assert_eq!(result.continuing_animations.len(), 1);
    
    // 1つのラインが削除されることを確認
    if let Animation::EraseLine { lines_remaining, .. } = &result.continuing_animations[0] {
        assert_eq!(*lines_remaining, 1);
    } else {
        panic!("Expected EraseLine animation");
    }
}

#[test]
fn test_erase_line_animation_completion_through_update() {
    let mut animations = vec![Animation::EraseLine { 
        lines_remaining: 1,
        last_update: Duration::from_millis(0)
    }];
    let current_time = Duration::from_millis(120);
    
    let result = update_animations(&mut animations, current_time);
    
    // アニメーションが完了し、継続アニメーションがないことを確認
    assert_eq!(result.continuing_animations.len(), 0);
}

#[test]
fn test_erase_line_animation_no_update_before_interval() {
    let mut animations = vec![Animation::EraseLine { 
        lines_remaining: 3,
        last_update: Duration::from_millis(50)
    }];
    let current_time = Duration::from_millis(120); // 70ms後（120ms未満）
    
    let result = update_animations(&mut animations, current_time);
    
    // アニメーションが継続されることを確認
    assert_eq!(result.continuing_animations.len(), 1);
    
    // ライン数が変更されていないことを確認
    if let Animation::EraseLine { lines_remaining, .. } = &result.continuing_animations[0] {
        assert_eq!(*lines_remaining, 3);
    } else {
        panic!("Expected EraseLine animation");
    }
}