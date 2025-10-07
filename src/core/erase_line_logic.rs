//! Core EraseLine Logic - Pure Functions
//! 
//! CLI版のEraseLineアニメーション機能をCore Moduleに移植
//! chain_bonus管理、solid_line操作、animation_step処理の純粋関数群

use crate::config::{BOARD_WIDTH, BOARD_HEIGHT};
use crate::core::board_logic::FixedBoard;
use crate::cell::Cell;

/// EraseLineステップの処理結果
#[derive(Debug, Clone, PartialEq)]
pub enum EraseLineStepResult {
    Continue,
    Complete { lines_erased: u32 },
}

/// EraseLineアニメーション状態
#[derive(Debug, Clone, PartialEq)]
pub struct EraseLineAnimationState {
    pub target_lines: Vec<usize>,
    pub current_step: usize,
    pub last_update_ms: u64,
    pub chain_bonus_consumed: u32,
    pub is_completed: bool,
}

impl EraseLineAnimationState {
    pub fn new(target_lines: Vec<usize>, start_time_ms: u64) -> Self {
        Self {
            target_lines,
            current_step: 0,
            last_update_ms: start_time_ms,
            chain_bonus_consumed: 0,
            is_completed: false,
        }
    }
}

/// 【純粋関数】PushDown完了時のEraseLineアニメーション開始判定
/// chain_bonusの量とSolidライン数から消去可能ライン数を計算
/// 
/// # Arguments
/// * `chain_bonus` - 現在のCHAIN-BONUS量
/// * `solid_lines_count` - 対象のSolidライン数
/// 
/// # Returns
/// EraseLineアニメーションで処理すべきライン数
pub fn determine_erase_line_count(chain_bonus: u32, solid_lines_count: usize) -> usize {
    std::cmp::min(chain_bonus as usize, solid_lines_count)
}

/// 【純粋関数】EraseLineアニメーション完了時のCHAIN-BONUS消費処理
/// 
/// # Arguments
/// * `chain_bonus` - 現在のCHAIN-BONUS量
/// * `lines_erased` - 消去されたライン数
/// 
/// # Returns
/// (消費後のchain_bonus, 実際に消費された量)
pub fn consume_chain_bonus_for_erase_line(chain_bonus: u32, lines_erased: u32) -> (u32, u32) {
    let consumed = std::cmp::min(chain_bonus, lines_erased);
    let new_chain_bonus = chain_bonus - consumed;
    (new_chain_bonus, consumed)
}

/// 【純粋関数】底辺から連続するSolidライン数をカウント
/// 
/// CLI版と同等のロジック：
/// - 物理的な底辺（BOARD_HEIGHT-1）から上に向かってチェック
/// - 各行が完全にSolidセルで埋まっているかチェック  
/// - 底辺からの連続性重視：非Solidライン発見時点でカウント終了
/// - 空セル、Occupied、Connected等が混在する行は非Solid
/// 
/// # Arguments
/// * `board` - ゲームボード（固定サイズ配列）
/// 
/// # Returns
/// 底辺から連続するSolidライン数
pub fn count_solid_lines_from_bottom(board: FixedBoard) -> usize {
    let mut count = 0;
    
    // 物理的な底辺から上に向かってチェック（CLI版準拠）
    for y in (0..BOARD_HEIGHT).rev() {
        let mut is_solid_line = true;
        
        // 行が完全にSolidブロックで埋まっているかチェック
        for x in 0..BOARD_WIDTH {
            match board[y][x] {
                Cell::Solid => {
                    // Solidセルは継続
                }
                _ => {
                    // Solid以外（空、Occupied、Connected等）があれば非Solid
                    is_solid_line = false;
                    break;
                }
            }
        }
        
        if is_solid_line {
            count += 1;
        } else {
            // 連続が途切れたらカウント終了（底辺からの連続性チェック）
            break;
        }
    }
    
    count
}

/// 【純粋関数】底辺のSolidライン1行を除去し、上部に空行を追加
/// 
/// EraseLineアニメーションの物理的な処理：
/// 1. 底辺のSolidライン1行を削除
/// 2. 上部（index 0）に新しい空行を挿入
/// 3. ボード高を1行拡張（相殺効果）
/// 
/// # Arguments
/// * `board` - ゲームボード（固定サイズ配列）
/// * `current_height` - 現在のボード高
/// 
/// # Returns
/// (更新されたboard, 新しいcurrent_height, 除去成功フラグ)
pub fn remove_solid_line_from_bottom(
    mut board: FixedBoard,
    current_height: usize,
) -> (FixedBoard, usize, bool) {
    // 底辺にSolidラインがあるかチェック
    if count_solid_lines_from_bottom(board) == 0 {
        return (board, current_height, false);
    }
    
    // CLI版準拠：物理的底辺ライン除去
    // 物理的底辺から最も下のSolidラインを特定
    let mut bottom_solid_line: Option<usize> = None;
    for y in (0..BOARD_HEIGHT).rev() {
        if board[y].iter().all(|cell| matches!(cell, Cell::Solid)) {
            bottom_solid_line = Some(y);
            break;
        }
    }
    
    if let Some(solid_y) = bottom_solid_line {
        // CLI版準拠：底辺ライン除去 + 上から下へシフト
        // 底辺ライン除去をシミュレート：上の行を1つずつ下にシフト
        for y in (1..=solid_y).rev() {
            board[y] = board[y - 1];  // 上の行を下の行にコピー
        }
        
        // 最上部（index 0）に新しい空行を挿入
        board[0] = [Cell::Empty; BOARD_WIDTH];
    }
    
    // 3. ボード高を1行拡張（相殺効果でプレイ領域が拡大）
    let new_height = std::cmp::min(current_height + 1, BOARD_HEIGHT);
    
    (board, new_height, true)
}

/// 【純粋関数】EraseLineアニメーション1ステップの処理
/// 120ミリ秒間隔でのライン消去処理
/// 
/// # Arguments
/// * `animation_state` - 現在のアニメーション状態
/// * `current_time_ms` - 現在時刻（ミリ秒）
/// * `board` - ゲームボード
/// * `current_height` - 現在のボード高
/// 
/// # Returns
/// (更新されたanimation_state, 更新されたboard, 新しいcurrent_height, ステップ結果)
pub fn process_erase_line_step(
    mut animation_state: EraseLineAnimationState,
    current_time_ms: u64,
    board: FixedBoard,
    current_height: usize,
) -> (EraseLineAnimationState, FixedBoard, usize, EraseLineStepResult) {
    // 120ミリ秒ごとに1ライン消去
    let erase_interval_ms = 120;
    let elapsed_ms = current_time_ms.saturating_sub(animation_state.last_update_ms);

    if elapsed_ms >= erase_interval_ms {
        if animation_state.current_step < animation_state.target_lines.len() {
            // 実際にSolidライン除去を実行
            let (new_board, new_height, removed) = remove_solid_line_from_bottom(board, current_height);
            
            if removed {
                animation_state.current_step += 1;
                animation_state.chain_bonus_consumed += 1;
                animation_state.last_update_ms = current_time_ms;

                let target_lines_len = animation_state.target_lines.len();
                let current_step = animation_state.current_step;

                if current_step >= target_lines_len {
                    animation_state.is_completed = true;
                    (animation_state, new_board, new_height, EraseLineStepResult::Complete { 
                        lines_erased: target_lines_len as u32 
                    })
                } else {
                    (animation_state, new_board, new_height, EraseLineStepResult::Continue)
                }
            } else {
                // Solidライン不足による完了
                let current_step = animation_state.current_step;
                animation_state.is_completed = true;
                (animation_state, board, current_height, EraseLineStepResult::Complete { 
                    lines_erased: current_step as u32 
                })
            }
        } else {
            let target_lines_len = animation_state.target_lines.len();
            animation_state.is_completed = true;
            (animation_state, board, current_height, EraseLineStepResult::Complete { 
                lines_erased: target_lines_len as u32 
            })
        }
    } else {
        (animation_state, board, current_height, EraseLineStepResult::Continue)
    }
}

/// 【純粋関数】EraseLineアニメーション開始条件の判定
/// enable_erase_lineフラグとchain_bonus、solid_line数を総合判定
/// 
/// # Arguments
/// * `enable_erase_line` - ToggleEraseLineで設定されるフラグ
/// * `chain_bonus` - 現在のCHAIN-BONUS量
/// * `board` - ゲームボード
/// * `current_height` - 現在のボード高
/// 
/// # Returns
/// EraseLineアニメーション開始可否とtarget_lines
pub fn should_start_erase_line_animation(
    enable_erase_line: bool,
    chain_bonus: u32,
    board: FixedBoard,
    current_height: usize,
) -> (bool, Vec<usize>) {
    if !enable_erase_line {
        return (false, Vec::new());
    }
    
    let solid_count = count_solid_lines_from_bottom(board);
    let erasable_lines = determine_erase_line_count(chain_bonus, solid_count);
    
    if erasable_lines > 0 {
        let target_lines: Vec<usize> = (0..erasable_lines)
            .map(|i| current_height - 1 - i)
            .collect();
        (true, target_lines)
    } else {
        (false, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_board_with_solid_lines(solid_lines: usize) -> FixedBoard {
        let mut board = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        
        // 底辺からsolid_lines分のSolidラインを作成
        for y in (BOARD_HEIGHT - solid_lines)..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                board[y][x] = Cell::Solid;
            }
        }
        
        board
    }

    fn create_test_board_with_solid_lines_at_height(solid_lines: usize, current_height: usize) -> FixedBoard {
        let mut board = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        
        // current_height範囲内で底辺からsolid_lines分のSolidラインを作成
        for y in (current_height.saturating_sub(solid_lines))..current_height {
            for x in 0..BOARD_WIDTH {
                board[y][x] = Cell::Solid;
            }
        }
        
        board
    }

    #[test]
    fn test_count_solid_lines_from_bottom() {
        let board = create_test_board_with_solid_lines(3);
        let count = count_solid_lines_from_bottom(board);
        assert_eq!(count, 3);
        
        // テストケース1: 空のボード（Solidライン無し）
        let empty_board = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        let count = count_solid_lines_from_bottom(empty_board);
        assert_eq!(count, 0, "空のボードでは0であるべき");

        // テストケース2: 底辺に1行のSolidライン
        let mut board_one_solid = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        // 物理的な底辺（BOARD_HEIGHT-1）にSolidライン作成
        for x in 0..BOARD_WIDTH {
            board_one_solid[BOARD_HEIGHT - 1][x] = Cell::Solid;
        }
        let count = count_solid_lines_from_bottom(board_one_solid);
        assert_eq!(count, 1, "底辺に1行のSolidラインがある場合は1であるべき");

        // テストケース3: 底辺から連続する2行のSolidライン
        let mut board_two_solid = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        // 底辺（BOARD_HEIGHT-1）とその上（BOARD_HEIGHT-2）にSolidライン作成
        for y in (BOARD_HEIGHT - 2)..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                board_two_solid[y][x] = Cell::Solid;
            }
        }
        let count = count_solid_lines_from_bottom(board_two_solid);
        assert_eq!(count, 2, "底辺から連続する2行のSolidラインがある場合は2であるべき");

        // テストケース4: 非連続のSolidライン（底辺Solid、1つ上Empty、2つ上Solid）
        let mut board_non_continuous = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        // 底辺（BOARD_HEIGHT-1）にSolidライン
        for x in 0..BOARD_WIDTH {
            board_non_continuous[BOARD_HEIGHT - 1][x] = Cell::Solid;
        }
        // 2つ上（BOARD_HEIGHT-3）にSolidライン（1つ上（BOARD_HEIGHT-2）はEmpty）
        for x in 0..BOARD_WIDTH {
            board_non_continuous[BOARD_HEIGHT - 3][x] = Cell::Solid;
        }
        let count = count_solid_lines_from_bottom(board_non_continuous);
        assert_eq!(count, 1, "非連続の場合は底辺からの連続分のみカウントすべき");

        // テストケース5: 底辺に不完全なライン（一部のセルがEmpty）
        let mut board_incomplete = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        // 底辺の最初のセル以外をSolidに
        for x in 1..BOARD_WIDTH {
            board_incomplete[BOARD_HEIGHT - 1][x] = Cell::Solid;
        }
        // board_incomplete[BOARD_HEIGHT-1][0] は Empty のまま
        let count = count_solid_lines_from_bottom(board_incomplete);
        assert_eq!(count, 0, "不完全なラインはSolidラインとしてカウントしないべき");
    }

    #[test]
    fn test_determine_erase_line_count() {
        assert_eq!(determine_erase_line_count(5, 3), 3); // solid_lines制限
        assert_eq!(determine_erase_line_count(2, 5), 2); // chain_bonus制限
        assert_eq!(determine_erase_line_count(0, 3), 0); // chain_bonus不足
    }

    #[test]
    fn test_consume_chain_bonus_for_erase_line() {
        let (new_bonus, consumed) = consume_chain_bonus_for_erase_line(10, 3);
        assert_eq!(new_bonus, 7);
        assert_eq!(consumed, 3);
        
        let (new_bonus, consumed) = consume_chain_bonus_for_erase_line(2, 5);
        assert_eq!(new_bonus, 0);
        assert_eq!(consumed, 2);
    }

    #[test]
    fn test_remove_solid_line_from_bottom() {
        // テストケース1: Solidラインが無い場合（除去失敗）
        let empty_board = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        let initial_height = 5;
        let (result_board, result_height, success) = remove_solid_line_from_bottom(empty_board, initial_height);
        
        assert!(!success, "Solidラインが無い場合は除去失敗すべき");
        assert_eq!(result_height, initial_height, "除去失敗時は高さ変更なし");
        // ボードも変更されていないことを確認
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                assert_eq!(result_board[y][x], Cell::Empty, "除去失敗時はボード変更なし");
            }
        }
        
        // テストケース2: 物理的底辺に1行のSolidライン（除去成功）
        let mut board_with_solid = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        // 物理的底辺（BOARD_HEIGHT-1）にSolidライン作成
        for x in 0..BOARD_WIDTH {
            board_with_solid[BOARD_HEIGHT - 1][x] = Cell::Solid;
        }
        
        let initial_height = 10;
        let solid_count_before = count_solid_lines_from_bottom(board_with_solid);
        println!("除去前のSolidライン数: {}", solid_count_before);
        
        let (result_board, result_height, success) = remove_solid_line_from_bottom(board_with_solid, initial_height);
        
        assert!(success, "Solidラインがある場合は除去成功すべき");
        assert_eq!(result_height, initial_height + 1, "除去成功時は高さ+1");
        
        let solid_count_after = count_solid_lines_from_bottom(result_board);
        println!("除去後のSolidライン数: {}", solid_count_after);
        
        assert_eq!(solid_count_after, 0, "除去後はSolidライン数が0になるべき");
    }

    #[test]
    fn test_should_start_erase_line_animation() {
        let board = create_test_board_with_solid_lines(3);
        
        // enable_erase_line=false → 開始しない
        let (should_start, _) = should_start_erase_line_animation(false, 5, board, BOARD_HEIGHT);
        assert!(!should_start);
        
        // enable_erase_line=true, chain_bonus=2, solid_lines=3 → 2ライン消去
        let (should_start, target_lines) = should_start_erase_line_animation(true, 2, board, BOARD_HEIGHT);
        assert!(should_start);
        assert_eq!(target_lines.len(), 2);
    }

    #[test]
    fn test_erase_line_animation_state() {
        let target_lines = vec![19, 18, 17];
        let state = EraseLineAnimationState::new(target_lines.clone(), 1000);
        
        assert_eq!(state.target_lines, target_lines);
        assert_eq!(state.current_step, 0);
        assert_eq!(state.last_update_ms, 1000);
        assert!(!state.is_completed);
    }
}