//! Integration Phase II テスト (修正版)
//! TDD Cycle II-1: PushDown完了時の相殺判定統合
//! TDD Cycle II-2: EraseLineアニメーション統合

use crate::animation::Animation;
use crate::animation::{
    consume_chain_bonus_for_erase_line, count_solid_lines_from_bottom, determine_erase_line_count,
};
use crate::cell::Cell;
use crate::config::BOARD_WIDTH;
use crate::{GameState, MockTimeProvider, TimeProvider};

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
        let target_lines: Vec<usize> = (0..erasable_lines).map(|i| board_height - 1 - i).collect();

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
    animations
        .iter()
        .find(|anim| matches!(anim, Animation::EraseLine { .. }))
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
    if let Some(Animation::EraseLine {
        target_solid_lines, ..
    }) = erase_animation
    {
        assert_eq!(
            target_solid_lines.len(),
            2,
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
    if let Some(Animation::EraseLine {
        target_solid_lines, ..
    }) = erase_animation
    {
        assert_eq!(
            target_solid_lines.len(),
            2,
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

// ==========================================
// TDD Cycle II-2: EraseLineアニメーション完了統合
// ==========================================

/// テストヘルパー: EraseLineアニメーション完了をシミュレート
fn simulate_erase_line_completion(state: &mut GameState, _time_provider: &MockTimeProvider) {
    // EraseLineアニメーション完了時の処理をシミュレート
    // 1. 対象Solidラインを除去
    // 2. CHAIN-BONUS消費
    // 3. アニメーション除去

    // 現在のEraseLineアニメーションを取得
    if let Some(Animation::EraseLine {
        target_solid_lines, ..
    }) = state
        .animation
        .iter()
        .find(|anim| matches!(anim, Animation::EraseLine { .. }))
    {
        let lines_to_erase = target_solid_lines.clone();
        let lines_erased = lines_to_erase.len() as u32;

        // Solidライン除去とCHAIN-BONUS消費
        for &line_y in &lines_to_erase {
            for x in 0..BOARD_WIDTH {
                state.board[line_y][x] = Cell::Empty;
            }
        }

        // CHAIN-BONUS消費
        let _consumed = consume_chain_bonus_for_erase_line(
            &mut state.custom_score_system.max_chains.chain_bonus,
            lines_erased,
        );

        // EraseLineアニメーション除去
        state
            .animation
            .retain(|anim| !matches!(anim, Animation::EraseLine { .. }));

        // 新ピース生成判定（アニメーションが空になった場合）
        if state.animation.is_empty() && state.current_piece.is_none() {
            state.spawn_piece();
        }
    }
}

/// TDD Cycle II-2: EraseLineアニメーション完了時に新ピースが生成されることを確認
#[test]
fn test_erase_line_completion_spawns_new_piece() {
    let mut state = GameState::new();

    // CHAIN-BONUSを3に設定
    state.custom_score_system.max_chains.chain_bonus = 3;

    // 底辺にSolidライン2本配置
    setup_solid_lines_at_bottom(&mut state, 2);

    // 既存ピースを削除（新ピース生成を確認するため）
    state.current_piece = None;

    // PushDownアニメーション完了をシミュレート（EraseLineアニメーション作成）
    let time_provider = MockTimeProvider::new();
    simulate_pushdown_completion(&mut state, &time_provider);

    // EraseLineアニメーションが作成されることを確認
    assert!(find_erase_line_animation(&state.animation).is_some());
    assert!(
        state.current_piece.is_none(),
        "EraseLineアニメーション中は新ピース未生成"
    );

    // EraseLineアニメーション完了をシミュレート
    simulate_erase_line_completion(&mut state, &time_provider);

    // 新ピースが生成されることを確認
    assert!(
        state.current_piece.is_some(),
        "EraseLineアニメーション完了後に新ピースが生成されるべき"
    );

    // アニメーションが空になることを確認
    assert!(
        state.animation.is_empty(),
        "EraseLineアニメーション完了後はアニメーションが空になるべき"
    );
}

/// TDD Cycle II-2: EraseLineアニメーション完了時のCHAIN-BONUS消費確認
#[test]
fn test_erase_line_completion_consumes_chain_bonus() {
    let mut state = GameState::new();

    // CHAIN-BONUSを5に設定
    state.custom_score_system.max_chains.chain_bonus = 5;

    // 底辺にSolidライン3本配置
    setup_solid_lines_at_bottom(&mut state, 3);

    // PushDownアニメーション完了をシミュレート
    let time_provider = MockTimeProvider::new();
    simulate_pushdown_completion(&mut state, &time_provider);

    // EraseLineアニメーション完了をシミュレート
    simulate_erase_line_completion(&mut state, &time_provider);

    // CHAIN-BONUSが3本分消費されることを確認
    assert_eq!(
        state.custom_score_system.max_chains.chain_bonus, 2,
        "EraseLineアニメーション完了時に消去ライン数分CHAIN-BONUSが消費されるべき"
    );
}

/// TDD Cycle II-2: 複数アニメーション実行中は新ピース生成されないテスト
#[test]
fn test_no_new_piece_with_other_animations_running() {
    let mut state = GameState::new();

    // CHAIN-BONUSを3に設定
    state.custom_score_system.max_chains.chain_bonus = 3;

    // 底辺にSolidライン2本配置
    setup_solid_lines_at_bottom(&mut state, 2);

    // 既存ピースを削除
    state.current_piece = None;

    // PushDownアニメーション完了をシミュレート
    let time_provider = MockTimeProvider::new();
    simulate_pushdown_completion(&mut state, &time_provider);

    // 別のアニメーションを追加（PushDownアニメーション）
    state.animation.push(Animation::PushDown {
        gray_line_y: 18,
        start_time: time_provider.now(),
    });

    // EraseLineアニメーション完了をシミュレート（ただし他のアニメーションが残る）
    if let Some(Animation::EraseLine {
        target_solid_lines, ..
    }) = state
        .animation
        .iter()
        .find(|anim| matches!(anim, Animation::EraseLine { .. }))
        .cloned()
    {
        // Solidライン除去のみ（アニメーション除去はしない）
        for &line_y in &target_solid_lines {
            for x in 0..BOARD_WIDTH {
                state.board[line_y][x] = Cell::Empty;
            }
        }

        // EraseLineアニメーションのみ除去
        state
            .animation
            .retain(|anim| !matches!(anim, Animation::EraseLine { .. }));

        // アニメーションが残っているので新ピース生成しない
        assert!(!state.animation.is_empty(), "他のアニメーションが実行中");
    }

    // 新ピースが生成されないことを確認
    assert!(
        state.current_piece.is_none(),
        "他のアニメーション実行中は新ピースが生成されないべき"
    );
}
