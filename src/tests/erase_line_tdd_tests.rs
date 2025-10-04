//! EraseLineアニメーション関連のテスト
//! Phase 9-1: EraseLineアニメーション基盤構築のTDDテスト

use crate::animation::{process_erase_line_step, Animation, EraseLineStepResult};
use std::time::Duration;

/// TDD Cycle 9-1-1: EraseLineアニメーション構造体設計のテスト
#[test]
fn test_erase_line_animation_structure() {
    // RED: 新しい構造体設計をテスト
    let animation = Animation::EraseLine {
        target_solid_lines: vec![19, 18, 17],
        current_step: 0,
        last_update: Duration::from_millis(0),
        chain_bonus_consumed: 0,
    };
    
    // 構造体が正しく作成されることを確認
    if let Animation::EraseLine { 
        target_solid_lines, 
        current_step, 
        last_update, 
        chain_bonus_consumed 
    } = animation {
        assert_eq!(target_solid_lines, vec![19, 18, 17]);
        assert_eq!(current_step, 0);
        assert_eq!(last_update, Duration::from_millis(0));
        assert_eq!(chain_bonus_consumed, 0);
    } else {
        panic!("EraseLine構造体の作成に失敗");
    }
}

/// TDD Cycle 9-1-2: EraseLineアニメーションステップ処理のテスト
#[test]
fn test_erase_line_animation_step_processing() {
    let mut animation = Animation::EraseLine {
        target_solid_lines: vec![19, 18, 17],
        current_step: 0,
        last_update: Duration::from_millis(0),
        chain_bonus_consumed: 0,
    };
    
    // 120ms経過後にステップ処理
    let result = process_erase_line_step(&mut animation, Duration::from_millis(120));
    
    // 1ステップ進行することを確認
    if let Animation::EraseLine { current_step, chain_bonus_consumed, .. } = animation {
        assert_eq!(current_step, 1);
        assert_eq!(chain_bonus_consumed, 1);
        assert!(matches!(result, EraseLineStepResult::Continue));
    } else {
        panic!("EraseLine構造体の処理に失敗");
    }
}

/// TDD Cycle 9-1-2: EraseLineアニメーション完了のテスト
#[test]
fn test_erase_line_animation_completion() {
    let mut animation = Animation::EraseLine {
        target_solid_lines: vec![19],
        current_step: 0,
        last_update: Duration::from_millis(0),
        chain_bonus_consumed: 0,
    };
    
    // 120ms経過後にステップ処理
    let result = process_erase_line_step(&mut animation, Duration::from_millis(120));
    
    // アニメーション完了を確認
    assert!(matches!(result, EraseLineStepResult::Complete { lines_erased: 1 }));
}

/// TDD Cycle 9-1-2: 時間未経過でのステップ処理テスト
#[test]
fn test_erase_line_animation_time_not_elapsed() {
    let mut animation = Animation::EraseLine {
        target_solid_lines: vec![19, 18],
        current_step: 0,
        last_update: Duration::from_millis(0),
        chain_bonus_consumed: 0,
    };
    
    // 60ms経過（120ms未満）
    let result = process_erase_line_step(&mut animation, Duration::from_millis(60));
    
    // ステップが進行しないことを確認
    if let Animation::EraseLine { current_step, chain_bonus_consumed, .. } = animation {
        assert_eq!(current_step, 0);
        assert_eq!(chain_bonus_consumed, 0);
        assert!(matches!(result, EraseLineStepResult::Continue));
    } else {
        panic!("EraseLine構造体の処理に失敗");
    }
}