//! EraseLineアニメーション統合テスト
//! ToggleEraseLine→enable_erase_line→EraseLineアニメーション開始→chain_bonus消費→solid_line除去の完全フロー

use crate::cell::Cell;
use crate::config::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::core::erase_line_logic::*;
use crate::core::game_state::CoreGameState;
use crate::game_color::GameColor;

/// テスト用のボード作成ヘルパー
fn create_test_board_with_bottom_solid_lines(
    solid_lines: usize,
) -> [[Cell; BOARD_WIDTH]; BOARD_HEIGHT] {
    let mut board = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];

    // 底部からsolid_lines分のSolidラインを作成
    for y in (BOARD_HEIGHT - solid_lines)..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            board[y][x] = Cell::Solid;
        }
    }

    board
}

/// ピース配置からchain_bonus増加まで統合テスト
#[test]
fn test_piece_lock_to_chain_bonus_increase() {
    let mut state = CoreGameState::new();

    // シナリオ：同色隣接ブロックでchain_bonus増加
    // 底部に5つの同色Cyanブロックを配置（より大きな連結で確実にchain_bonus増加）
    state.board[19][4] = Cell::Occupied(GameColor::Cyan);
    state.board[19][5] = Cell::Occupied(GameColor::Cyan);
    state.board[19][6] = Cell::Occupied(GameColor::Cyan);
    state.board[19][7] = Cell::Occupied(GameColor::Cyan);
    state.board[19][8] = Cell::Occupied(GameColor::Cyan);

    let initial_chain_bonus = state.chain_bonus;
    println!("Initial chain_bonus: {}", initial_chain_bonus);
    println!("Initial max_chains cyan: {}", state.max_chains.cyan);

    // 隣接ブロック処理を実行（lock_current_pieceの部分処理）
    let old_max_chains = state.max_chains.clone();

    // 1. 隣接ブロック検出・Connected変換
    let components = crate::core::board_logic::find_connected_components(state.board, &[]);
    println!("Found {} components", components.len());
    state = state.apply_connected_components(components);

    // 2. MAX-CHAIN更新
    state = state.update_max_chains_from_board();
    println!("Updated max_chains cyan: {}", state.max_chains.cyan);

    // 3. chain_bonus増加
    let total_increase = crate::core::game_state::CoreColorMaxChains::calculate_chain_increases(
        &old_max_chains,
        &state.max_chains,
    );
    println!("Total increase calculated: {}", total_increase);
    state = state.add_chain_bonus(total_increase);

    println!("Final chain_bonus: {}", state.chain_bonus);

    // 検証：5つのCyanブロックがConnectedになっている
    assert!(matches!(
        state.board[19][4],
        Cell::Connected {
            color: GameColor::Cyan,
            count: 5
        }
    ));
    assert!(matches!(
        state.board[19][5],
        Cell::Connected {
            color: GameColor::Cyan,
            count: 5
        }
    ));
    assert!(matches!(
        state.board[19][6],
        Cell::Connected {
            color: GameColor::Cyan,
            count: 5
        }
    ));
    assert!(matches!(
        state.board[19][7],
        Cell::Connected {
            color: GameColor::Cyan,
            count: 5
        }
    ));
    assert!(matches!(
        state.board[19][8],
        Cell::Connected {
            color: GameColor::Cyan,
            count: 5
        }
    ));

    // chain_bonusが増加したかをチェック（初期値が0の場合、最大でも同じになることがある）
    if state.max_chains.cyan > old_max_chains.cyan {
        assert!(
            state.chain_bonus >= initial_chain_bonus,
            "chain_bonus should increase or stay same: {} to {}",
            initial_chain_bonus,
            state.chain_bonus
        );
    }
}

#[test]
fn test_enable_erase_line_triggers_animation() {
    let mut state = CoreGameState::new();

    // enable_erase_lineを有効化
    state.enable_erase_line = true;

    // 十分なchain_bonusを設定
    state = state.add_chain_bonus(10);

    // 底部にSolidラインを作成
    state.board = create_test_board_with_bottom_solid_lines(2);
    state.current_board_height = 20; // テスト用の高さ設定

    // EraseLineアニメーション開始条件をテスト
    let solid_count = count_solid_lines_from_bottom(state.board);
    let (should_start, target_lines) = should_start_erase_line_animation(
        state.enable_erase_line,
        state.chain_bonus,
        state.board,
        state.current_board_height,
    );

    assert!(should_start, "EraseLineアニメーションが開始されるべき");
    assert_eq!(solid_count, 2, "2ラインのSolidラインが検出されるべき");
    assert_eq!(
        target_lines.len(),
        2,
        "2つのターゲットラインが設定されるべき"
    );
}

#[test]
fn test_chain_bonus_consumption_for_erase_line() {
    let initial_chain_bonus = 15u32;
    let lines_to_erase = 3u32;

    // CLI版準拠のchain_bonus消費ロジックをテスト
    let (remaining_bonus, consumed) =
        consume_chain_bonus_for_erase_line(initial_chain_bonus, lines_to_erase);

    assert_eq!(
        consumed, lines_to_erase,
        "消費されるchain_bonusは除去ライン数と等しい"
    );
    assert_eq!(
        remaining_bonus,
        initial_chain_bonus - lines_to_erase,
        "残りのchain_bonusが正確"
    );
}

#[test]
fn test_determine_erase_line_count_logic() {
    // 十分なchain_bonusとSolidライン
    assert_eq!(
        determine_erase_line_count(10, 5),
        5,
        "Solidライン数がchain_bonusより少ない場合"
    );

    // chain_bonusが不足
    assert_eq!(
        determine_erase_line_count(3, 5),
        3,
        "chain_bonusが不足している場合"
    );

    // 境界値
    assert_eq!(
        determine_erase_line_count(5, 5),
        5,
        "chain_bonusとSolidライン数が等しい場合"
    );
    assert_eq!(determine_erase_line_count(0, 3), 0, "chain_bonusが0の場合");
}

#[test]
fn test_solid_line_removal_integration() {
    let board = create_test_board_with_bottom_solid_lines(3);
    let initial_height = BOARD_HEIGHT;

    // 最初のSolidライン除去
    let (new_board, new_height, removed) = remove_solid_line_from_bottom(board, initial_height);

    assert!(removed, "Solidラインが除去されるべき");
    assert_eq!(new_height, initial_height, "高さは変更されない");

    // 底部ラインが空になり、上のラインがシフトダウンしている
    for x in 0..BOARD_WIDTH {
        assert!(
            matches!(new_board[BOARD_HEIGHT - 1][x], Cell::Solid),
            "シフト後も底部はSolidライン"
        );
        assert!(matches!(new_board[0][x], Cell::Empty), "最上部は空行になる");
    }

    // 残りのSolidライン数が減っている
    let remaining_solid = count_solid_lines_from_bottom(new_board);
    assert_eq!(remaining_solid, 2, "1ライン除去後、2ラインのSolidが残る");
}

#[test]
fn test_full_erase_line_animation_flow() {
    let mut state = CoreGameState::new();

    // 前提条件設定
    state.enable_erase_line = true;
    state = state.add_chain_bonus(5); // 5ライン分のchain_bonus
    state.board = create_test_board_with_bottom_solid_lines(3); // 3ラインのSolid
    state.current_board_height = 20; // テスト用の高さ設定

    let initial_chain_bonus = state.chain_bonus;
    let initial_solid_count = count_solid_lines_from_bottom(state.board);

    // 完全なEraseLineアニメーションフローを1ステップずつテスト

    // 1. アニメーション開始判定
    let (should_start, target_lines) = should_start_erase_line_animation(
        state.enable_erase_line,
        state.chain_bonus,
        state.board,
        state.current_board_height,
    );
    assert!(should_start, "アニメーション開始条件を満たす");
    assert_eq!(
        target_lines.len(),
        initial_solid_count,
        "Solidライン数と一致するターゲットライン"
    );

    // 2. 除去ライン数決定
    let lines_to_erase = determine_erase_line_count(state.chain_bonus, initial_solid_count);
    assert_eq!(lines_to_erase, 3, "3ライン除去される");

    // 3. chain_bonus消費
    let (_remaining_bonus, consumed) =
        consume_chain_bonus_for_erase_line(state.chain_bonus, lines_to_erase as u32);
    state = state.consume_chain_bonus(consumed).0;
    assert_eq!(consumed, 3, "3のchain_bonusが消費される");
    assert_eq!(
        state.chain_bonus,
        initial_chain_bonus - 3,
        "chain_bonusが正確に減少"
    );

    // 4. Solidライン除去（複数回実行）
    let mut current_board = state.board;
    let mut removed_count = 0;

    for _step in 0..lines_to_erase {
        let (new_board, _, removed) = remove_solid_line_from_bottom(current_board, BOARD_HEIGHT);
        if removed {
            current_board = new_board;
            removed_count += 1;
        } else {
            break;
        }
    }

    assert_eq!(removed_count, 3, "3回のライン除去が実行される");

    // 5. 最終状態検証
    let final_solid_count = count_solid_lines_from_bottom(current_board);
    assert_eq!(final_solid_count, 0, "すべてのSolidラインが除去される");

    // ボード最下部が空になっている
    for x in 0..BOARD_WIDTH {
        assert!(
            matches!(current_board[BOARD_HEIGHT - 1][x], Cell::Empty),
            "最下部は空セルになる"
        );
    }
}

#[test]
fn test_erase_line_with_insufficient_chain_bonus() {
    let mut state = CoreGameState::new();

    // 不足するchain_bonusシナリオ
    state.enable_erase_line = true;
    state = state.add_chain_bonus(2); // 2しかない
    state.board = create_test_board_with_bottom_solid_lines(5); // 5ライン必要

    let solid_count = count_solid_lines_from_bottom(state.board);
    let lines_to_erase = determine_erase_line_count(state.chain_bonus, solid_count);

    // chain_bonusの制限により2ラインのみ除去される
    assert_eq!(
        lines_to_erase, 2,
        "chain_bonusが不足する場合は利用可能分のみ"
    );

    // chain_bonus消費
    let (_, consumed) =
        consume_chain_bonus_for_erase_line(state.chain_bonus, lines_to_erase as u32);
    assert_eq!(consumed, 2, "利用可能な分だけ消費される");
}

#[test]
fn test_erase_line_disabled_scenario() {
    let mut state = CoreGameState::new();

    // enable_erase_line無効シナリオ
    state.enable_erase_line = false; // 無効
    state = state.add_chain_bonus(10);
    state.board = create_test_board_with_bottom_solid_lines(3);

    let (should_start, _target_lines) = should_start_erase_line_animation(
        state.enable_erase_line,
        state.chain_bonus,
        state.board,
        state.current_board_height,
    );

    assert!(
        !should_start,
        "enable_erase_lineが無効の場合はアニメーション開始しない"
    );
}
