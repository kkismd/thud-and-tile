//! Integration Phase II テスト (修正版)
//! TDD Cycle II-1: PushDown完了時の相殺判定統合
//! TDD Cycle II-2: EraseLineアニメーション統合

use crate::animation::Animation;
use crate::cell::Cell;
use crate::config::BOARD_WIDTH;
use crate::{GameState, MockTimeProvider, TimeProvider};
use crate::animation::{count_solid_lines_from_bottom, determine_erase_line_count};

/// テストヘルパー: 底辺にSolidライン配置
fn setup_solid_lines_at_bottom(state: &mut GameState, num_lines: usize) {
    let board_height = state.board.len();
    for i in 0..num_lines {
        let line_y = board_height - 1 - i;
        for x in 0..BOARD_WIDTH {
            state.board[line_y][x] = Cell::Solid;
        }
    }
}

/// テストヘルパー: PushDownアニメーション完了シミュレート
fn simulate_pushdown_completion(state: &mut GameState, time_provider: &MockTimeProvider) {
    // PushDown完了後の状態をシミュレート：
    // update_all_connected_block_countsが呼ばれ、
    // EraseLineアニメーション判定が実行される
    state.update_all_connected_block_counts();
    
    // PushDownStepResult::Completedケースの処理をシミュレート
    // handle_animationからの抜粋ロジック
    let solid_count = count_solid_lines_from_bottom(&state.board);
    let chain_bonus = state.custom_score_system.max_chains.chain_bonus;
    let erasable_lines = determine_erase_line_count(chain_bonus, solid_count);
    
    if erasable_lines > 0 {
        let board_height = state.board.len();
        let target_lines: Vec<usize> = (0..erasable_lines)
            .map(|i| board_height - 1 - i)
            .collect();
        
        state.animation.push(Animation::EraseLine {
            target_solid_lines: target_lines,
            current_step: 0,
            last_update: time_provider.now(),
            chain_bonus_consumed: 0,
        });
    }
}

/// テストヘルパー: EraseLineアニメーション検索
fn find_erase_line_animation(animations: &[Animation]) -> Option<&Animation> {
    animations.iter().find(|anim| matches!(anim, Animation::EraseLine { .. }))
}

/// TDD Cycle II-1: PushDown完了時にEraseLineアニメーションが作成されることを確認
#[test]
fn test_pushdown_triggers_erase_line_animation() {
    let mut state = GameState::new();
    
    // CHAIN-BONUSを3に設定
    state.custom_score_system.max_chains.chain_bonus = 3;
    
    // 底辺にSolidライン2本配置
    setup_solid_lines_at_bottom(&mut state, 2);
    
    // PushDownアニメーション完了をシミュレート
    let time_provider = MockTimeProvider::new();
    simulate_pushdown_completion(&mut state, &time_provider);
    
    // EraseLineアニメーションが作成されることを確認
    let erase_animation = find_erase_line_animation(&state.animation);
    assert!(
        erase_animation.is_some(),
        "PushDown完了後にEraseLineアニメーションが作成されるべき"
    );
    
    // EraseLineアニメーションの内容確認
    if let Some(Animation::EraseLine { target_solid_lines, .. }) = erase_animation {
        assert_eq!(
            target_solid_lines.len(), 2,
            "CHAIN-BONUS(3)とSolidライン(2)から min(3,2)=2本が対象になるべき"
        );
    }
}

/// TDD Cycle II-1: CHAIN-BONUSがSolidライン数より少ない場合のテスト
#[test]
fn test_pushdown_erase_line_limited_by_chain_bonus() {
    let mut state = GameState::new();
    
    // CHAIN-BONUSを2に設定
    state.custom_score_system.max_chains.chain_bonus = 2;
    
    // 底辺にSolidライン4本配置
    setup_solid_lines_at_bottom(&mut state, 4);
    
    // PushDownアニメーション完了をシミュレート
    let time_provider = MockTimeProvider::new();
    simulate_pushdown_completion(&mut state, &time_provider);
    
    // EraseLineアニメーション確認
    let erase_animation = find_erase_line_animation(&state.animation);
    assert!(erase_animation.is_some());
    
    // CHAIN-BONUSによって制限される
    if let Some(Animation::EraseLine { target_solid_lines, .. }) = erase_animation {
        assert_eq!(
            target_solid_lines.len(), 2,
            "CHAIN-BONUS(2)により2本に制限されるべき"
        );
    }
}

/// TDD Cycle II-1: CHAIN-BONUSが0の場合EraseLineアニメーションが作成されないテスト
#[test]
fn test_pushdown_no_erase_line_when_no_chain_bonus() {
    let mut state = GameState::new();
    
    // CHAIN-BONUSを0に設定
    state.custom_score_system.max_chains.chain_bonus = 0;
    
    // 底辺にSolidライン2本配置
    setup_solid_lines_at_bottom(&mut state, 2);
    
    // PushDownアニメーション完了をシミュレート
    let time_provider = MockTimeProvider::new();
    simulate_pushdown_completion(&mut state, &time_provider);
    
    // CHAIN-BONUSが0なのでEraseLineアニメーションは作成されない
    let erase_animation = find_erase_line_animation(&state.animation);
    assert!(
        erase_animation.is_none(),
        "CHAIN-BONUS=0の場合EraseLineアニメーションは作成されないべき"
    );
}