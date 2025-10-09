//! CoreGameStateのchain_bonus操作テスト

use crate::core::game_state::CoreGameState;

#[test]
fn test_add_chain_bonus() {
    let initial_state = CoreGameState::new();
    assert_eq!(initial_state.chain_bonus, 0);

    let state_with_bonus = initial_state.add_chain_bonus(50);
    assert_eq!(state_with_bonus.chain_bonus, 50);

    let state_with_more_bonus = state_with_bonus.add_chain_bonus(30);
    assert_eq!(state_with_more_bonus.chain_bonus, 80);
}

#[test]
fn test_add_chain_bonus_overflow_protection() {
    let initial_state = CoreGameState::new();
    let state_with_max = initial_state.add_chain_bonus(u32::MAX);
    assert_eq!(state_with_max.chain_bonus, u32::MAX);

    // 追加でオーバーフローしない
    let state_after_add = state_with_max.add_chain_bonus(100);
    assert_eq!(state_after_add.chain_bonus, u32::MAX);
}

#[test]
fn test_consume_chain_bonus() {
    let initial_state = CoreGameState::new().add_chain_bonus(100);
    assert_eq!(initial_state.chain_bonus, 100);

    let (state_after_consume, consumed) = initial_state.consume_chain_bonus(30);
    assert_eq!(state_after_consume.chain_bonus, 70);
    assert_eq!(consumed, 30);

    let (state_after_more_consume, more_consumed) = state_after_consume.consume_chain_bonus(50);
    assert_eq!(state_after_more_consume.chain_bonus, 20);
    assert_eq!(more_consumed, 50);
}

#[test]
fn test_consume_chain_bonus_underflow_protection() {
    let initial_state = CoreGameState::new().add_chain_bonus(30);
    assert_eq!(initial_state.chain_bonus, 30);

    // 利用可能な分だけ消費される
    let (state_after_consume, consumed) = initial_state.consume_chain_bonus(50);
    assert_eq!(state_after_consume.chain_bonus, 0);
    assert_eq!(consumed, 30); // 実際に消費できた分だけ

    // 既に0の場合
    let (state_final, final_consumed) = state_after_consume.consume_chain_bonus(10);
    assert_eq!(state_final.chain_bonus, 0);
    assert_eq!(final_consumed, 0);
}

#[test]
fn test_erase_line_integration_pattern() {
    // EraseLineアニメーションでの使用パターンをテスト
    let initial_state = CoreGameState::new().add_chain_bonus(100);

    // 3ライン除去のためにchain_bonusを消費
    let lines_to_erase = 3;
    let (state_after_erase, consumed) = initial_state.consume_chain_bonus(lines_to_erase);

    assert_eq!(state_after_erase.chain_bonus, 97); // 100 - 3
    assert_eq!(consumed, 3);

    // 追加のボーナス獲得
    let state_with_new_bonus = state_after_erase.add_chain_bonus(20);
    assert_eq!(state_with_new_bonus.chain_bonus, 117); // 97 + 20
}
