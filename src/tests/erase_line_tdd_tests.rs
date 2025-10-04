//! EraseLineアニメーション関連のテスト
//! Phase 9-1: EraseLineアニメーション基盤構築のTDDテスト
//! Phase 9-2: CHAIN-BONUS統合システムのTDDテスト

use crate::animation::{
    process_erase_line_step, Animation, EraseLineStepResult,
    determine_erase_line_count, consume_chain_bonus_for_erase_line
};
use std::time::Duration;

/// ============================================================================
/// Phase 9-1: EraseLineアニメーション基盤構築のテスト
/// ============================================================================

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

/// ============================================================================
/// Phase 9-2: CHAIN-BONUS統合システムのテスト
/// ============================================================================

/// TDD Cycle 9-2-1: PushDown完了時のCHAIN-BONUS制限テスト
#[test]
fn test_chain_bonus_limits_erase_line_creation() {
    // RED: CHAIN-BONUSがSolidライン数を制限することをテスト
    let chain_bonus = 2;
    let solid_lines = vec![19, 18, 17]; // 3行のSolidライン
    
    // CHAIN-BONUSによる制限を適用
    let erase_line_count = determine_erase_line_count(chain_bonus, solid_lines.len());
    
    // min(chain_bonus, solid_lines.len()) = min(2, 3) = 2
    assert_eq!(erase_line_count, 2);
}

/// TDD Cycle 9-2-1: CHAIN-BONUS不足時の制限テスト
#[test]
fn test_insufficient_chain_bonus_limits_erase_lines() {
    // RED: CHAIN-BONUSが不足している場合の制限テスト
    let chain_bonus = 1;
    let solid_lines = vec![19, 18, 17, 16, 15]; // 5行のSolidライン
    
    // CHAIN-BONUSによる制限を適用
    let erase_line_count = determine_erase_line_count(chain_bonus, solid_lines.len());
    
    // min(chain_bonus, solid_lines.len()) = min(1, 5) = 1
    assert_eq!(erase_line_count, 1);
}

/// TDD Cycle 9-2-2: EraseLineアニメーション完了時のCHAIN-BONUS消費テスト
#[test]
fn test_chain_bonus_consumption_on_erase_line_completion() {
    // RED: EraseLineアニメーション完了時のCHAIN-BONUS消費をテスト
    let mut initial_chain_bonus = 5;
    let lines_erased = 3;
    
    // EraseLineアニメーション完了時の処理をシミュレート
    let consumed = consume_chain_bonus_for_erase_line(&mut initial_chain_bonus, lines_erased);
    
    // 3ライン消去で3のCHAIN-BONUSが消費されることを確認
    assert_eq!(consumed, 3);
    assert_eq!(initial_chain_bonus, 2); // 5 - 3 = 2
}

/// TDD Cycle 9-2-2: CHAIN-BONUS枯渇時の消費制限テスト
#[test]
fn test_chain_bonus_exhaustion_limits_consumption() {
    // RED: CHAIN-BONUS枯渇時の消費制限をテスト
    let mut initial_chain_bonus = 1;
    let lines_erased = 3;
    
    // EraseLineアニメーション完了時の処理をシミュレート
    let consumed = consume_chain_bonus_for_erase_line(&mut initial_chain_bonus, lines_erased);
    
    // 1しかCHAIN-BONUSがない場合、1のみ消費
    assert_eq!(consumed, 1);
    assert_eq!(initial_chain_bonus, 0); // 1 - 1 = 0
}