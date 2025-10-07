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

/// 【純粋関数】底辺からSolidライン（完全Solid行）の数をカウント
/// 
/// Solidラインの定義：
/// - ボードの幅（10セル）全てがCell::Solidで埋まっている行
/// - 空セル、Occupied、Connected等が混在する行は非Solid
/// 
/// # Arguments
/// * `board` - ゲームボード（固定サイズ配列）
/// * `current_height` - 現在のボード高
/// 
/// # Returns
/// 底辺から連続するSolidライン数
pub fn count_solid_lines_from_bottom(board: FixedBoard, current_height: usize) -> usize {
    let mut count = 0;
    
    // 底辺から上に向かってチェック（連続性が重要）
    for y in (0..current_height).rev() {
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
    if count_solid_lines_from_bottom(board, current_height) == 0 {
        return (board, current_height, false);
    }
    
    // CLI版準拠：底辺ライン除去
    // 1. 上の行を1つずつ下にシフト（底辺ライン削除効果）
    //    底辺から上に向かって、一つ上の行をコピーする
    for y in (1..current_height).rev() {
        board[y] = board[y - 1];  // 上の行を下の行にコピー
    }
    
    // 2. 最上部（index 0）を空行にクリア
    board[0] = [Cell::Empty; BOARD_WIDTH];
    
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
    
    let solid_count = count_solid_lines_from_bottom(board, current_height);
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
        let count = count_solid_lines_from_bottom(board, BOARD_HEIGHT);
        assert_eq!(count, 3);
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
        let initial_height = BOARD_HEIGHT - 5; // 拡張余地を作る
        
        // 底辺から連続する2つのSolidラインを作成
        let mut board = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        
        // 底辺（current_height-1）とその上（current_height-2）にSolidライン作成
        for y in (initial_height-2)..initial_height {
            for x in 0..BOARD_WIDTH {
                board[y][x] = Cell::Solid;
            }
        }
        
        // デバッグ：Solidライン数を確認
        let solid_count_before = count_solid_lines_from_bottom(board, initial_height);
        println!("Solid lines before removal: {}, initial_height: {}", solid_count_before, initial_height);
        
        let (new_board, new_height, success) = remove_solid_line_from_bottom(board, initial_height);
        
        assert!(success, "Should successfully remove solid line when {} solid lines exist", solid_count_before);
        assert_eq!(new_height, initial_height + 1); // height拡張確認
        
        // 底辺の固体ラインが除去されたことを確認
        let remaining_solid = count_solid_lines_from_bottom(new_board, new_height);
        assert_eq!(remaining_solid, 1);
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