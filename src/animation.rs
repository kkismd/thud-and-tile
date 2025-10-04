//! アニメーション処理の共通ロジック
//! CLI版とWASM版で共有されるアニメーション処理を統一

use crate::cell::Cell;
use crate::config::{
    BLINK_ANIMATION_STEP, BLINK_COUNT_MAX, BOARD_HEIGHT, BOARD_WIDTH, PUSH_DOWN_STEP_DURATION,
};
use crate::game_color::GameColor;
use crate::scoring::ColorMaxChains; // MaxChainsの正しい型名
use std::time::Duration;

/// アニメーションの種類
#[derive(Debug, Clone, PartialEq)]
pub enum Animation {
    LineBlink {
        lines: Vec<usize>,
        count: usize,
        start_time: Duration,
    },
    PushDown {
        gray_line_y: usize,
        start_time: Duration,
    },
    EraseLine {
        lines_remaining: u32,
        last_update: Duration,
    },
}

/// アニメーション処理結果
#[derive(Debug)]
pub struct AnimationResult {
    pub continuing_animations: Vec<Animation>,
    pub completed_line_blinks: Vec<Vec<usize>>, // 完了したLineBlink のラインリスト
    pub completed_push_downs: Vec<usize>,       // 完了したPush Down のgray_line_y
}

impl AnimationResult {
    pub fn new() -> Self {
        Self {
            continuing_animations: Vec::new(),
            completed_line_blinks: Vec::new(),
            completed_push_downs: Vec::new(),
        }
    }
}

/// アニメーション更新処理（CLI版とWASM版共通）
/// 注意: この関数はLineBlink完了時にcompleted_line_blinksのみを返します。
/// PushDownアニメーションの生成は呼び出し元で底辺ライン判定を行った後に実行してください。
pub fn update_animations(
    animations: &mut Vec<Animation>,
    current_time: Duration,
) -> AnimationResult {
    let mut result = AnimationResult::new();

    for animation in animations.drain(..) {
        match animation {
            Animation::LineBlink {
                lines,
                count,
                start_time,
            } => {
                let elapsed = current_time - start_time;
                let blink_step = BLINK_ANIMATION_STEP;
                let steps_elapsed = elapsed.as_millis() / blink_step.as_millis();

                if steps_elapsed >= BLINK_COUNT_MAX as u128 {
                    // LineBlink完了 → 呼び出し元で底辺ライン判定を実行
                    result.completed_line_blinks.push(lines.clone());

                    // PushDownアニメーションの生成は呼び出し元に委託
                    // （CLI版: 底辺ライン判定後に非底辺ラインのみPushDown作成）
                } else {
                    // LineBlink継続
                    result.continuing_animations.push(Animation::LineBlink {
                        lines,
                        count: steps_elapsed as usize,
                        start_time,
                    });
                }
            }
            Animation::PushDown {
                gray_line_y,
                start_time,
            } => {
                let elapsed = current_time - start_time;

                if elapsed >= PUSH_DOWN_STEP_DURATION {
                    // Push Down 1ステップ実行またはアニメーション完了
                    result.completed_push_downs.push(gray_line_y);
                } else {
                    // Push Down継続
                    result.continuing_animations.push(Animation::PushDown {
                        gray_line_y,
                        start_time,
                    });
                }
            }
            Animation::EraseLine {
                lines_remaining,
                last_update,
            } => {
                // 最小実装：そのまま継続
                result.continuing_animations.push(Animation::EraseLine {
                    lines_remaining,
                    last_update,
                });
            }
        }
    }

    result
}

/// Push Downアニメーション1ステップの処理（CLI版とWASM版共通）
pub fn process_push_down_step(
    board: &mut Vec<Vec<Cell>>,
    current_board_height: &mut usize,
    gray_line_y: usize,
) -> PushDownStepResult {
    let target_y = gray_line_y + 1;

    // Push Down完了条件をチェック
    if target_y >= *current_board_height
        || (target_y < BOARD_HEIGHT && board[target_y][0] == Cell::Solid)
    {
        // Push Down完了: グレーラインをSolidに変換
        for x in 0..BOARD_WIDTH {
            board[gray_line_y][x] = Cell::Solid;
        }
        *current_board_height = current_board_height.saturating_sub(1);

        PushDownStepResult::Completed
    } else {
        // ブロックを1ライン下に移動
        if target_y < BOARD_HEIGHT {
            board.remove(target_y);
            board.insert(0, vec![Cell::Empty; BOARD_WIDTH]);

            PushDownStepResult::Moved {
                new_gray_line_y: target_y,
            }
        } else {
            PushDownStepResult::Completed
        }
    }
}

/// Push Downステップの結果
#[derive(Debug)]
pub enum PushDownStepResult {
    Completed,
    Moved { new_gray_line_y: usize },
}

/// EraseLineステップの結果
#[derive(Debug)]
pub enum EraseLineStepResult {
    Continue,
    Complete,
}

/// EraseLineアニメーション処理（1ステップ実行）
pub fn process_erase_line_step(
    animation: &mut Animation,
    current_time: Duration,
) -> EraseLineStepResult {
    if let Animation::EraseLine {
        lines_remaining,
        last_update,
    } = animation
    {
        // 100ミリ秒ごとに1ライン消去
        let erase_interval = Duration::from_millis(100);

        if current_time - *last_update >= erase_interval {
            if *lines_remaining > 0 {
                *lines_remaining -= 1;
                *last_update = current_time;

                if *lines_remaining == 0 {
                    EraseLineStepResult::Complete
                } else {
                    EraseLineStepResult::Continue
                }
            } else {
                EraseLineStepResult::Complete
            }
        } else {
            EraseLineStepResult::Continue
        }
    } else {
        // EraseLine以外のアニメーションが渡された場合はComplete扱い
        EraseLineStepResult::Complete
    }
}

/// ライン消去時のスコア計算（CLI版とWASM版共通）
pub fn calculate_line_clear_score(
    board: &Vec<Vec<Cell>>,
    line_y: usize,
    max_chains: &ColorMaxChains,
) -> Vec<(GameColor, u32)> {
    let mut scores = Vec::new();

    for x in 0..BOARD_WIDTH {
        match board[line_y][x] {
            Cell::Occupied(color) => {
                // Occupied blocks have count=1
                let points = max_chains.get(color) * 10;
                scores.push((color, points));
            }
            Cell::Connected { color, count } => {
                // Connected blocks use their actual count value
                let points = (count as u32) * max_chains.get(color) * 10;
                scores.push((color, points));
            }
            _ => {} // Empty cells and other types are ignored
        }
    }

    scores
}

/// ライン消去処理の共通ロジック（CLI版とWASM版共通）
pub fn process_line_clear(
    board: &mut Vec<Vec<Cell>>,
    current_board_height: usize,
    lines: &[usize],
) -> (Vec<usize>, Vec<usize>) {
    let mut bottom_lines_cleared = Vec::new();
    let mut non_bottom_lines_cleared = Vec::new();

    // Bottom line と Non-bottom line を分離
    for &y in lines {
        if y == current_board_height - 1 {
            bottom_lines_cleared.push(y);
        } else {
            non_bottom_lines_cleared.push(y);
        }
    }

    // Bottom lines の標準テトリスクリア処理
    if !bottom_lines_cleared.is_empty() {
        let num_cleared = bottom_lines_cleared.len();
        let mut sorted_lines = bottom_lines_cleared.to_vec();
        sorted_lines.sort_by(|a, b| b.cmp(a));

        // ライン削除と上からの補充
        for &line_y in &sorted_lines {
            board.remove(line_y);
        }
        for _ in 0..num_cleared {
            board.insert(0, vec![Cell::Empty; BOARD_WIDTH]);
        }
    }

    // Non-bottom lines をグレー化（アニメーション準備）
    for &y in &non_bottom_lines_cleared {
        for x in 0..BOARD_WIDTH {
            board[y][x] = Cell::Occupied(GameColor::Grey);
        }
    }

    (bottom_lines_cleared, non_bottom_lines_cleared)
}
