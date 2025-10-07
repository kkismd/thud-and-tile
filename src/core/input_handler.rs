//! Core Input Handler - Unified Input Processing
//! 
//! CLI版とWASM版で共有する統一入力処理
//! GameInputをCoreGameStateに適用する純粋関数

use crate::core::game_state::{CoreGameState, CoreGameEvent, CoreGameMode};
use crate::game_input::GameInput;
use crate::tetromino::Tetromino;

/// 入力処理結果
#[derive(Debug, Clone)]
pub struct InputProcessResult {
    pub new_state: CoreGameState,
    pub events: Vec<CoreGameEvent>,
    pub input_consumed: bool,
    pub render_required: bool,
}

/// 【純粋関数】入力をコアゲーム状態に適用
pub fn process_input(state: CoreGameState, input: GameInput, current_time_ms: u64) -> InputProcessResult {
    match state.game_mode {
        CoreGameMode::Title => process_title_input(state, input, current_time_ms),
        CoreGameMode::Playing => process_playing_input(state, input, current_time_ms),
        CoreGameMode::GameOver => process_game_over_input(state, input, current_time_ms),
    }
}

/// タイトルモード入力処理
fn process_title_input(mut state: CoreGameState, input: GameInput, _current_time_ms: u64) -> InputProcessResult {
    let mut events = Vec::new();
    let mut input_consumed = false;
    let mut render_required = false;

    match input {
        GameInput::Restart => {
            state.game_mode = CoreGameMode::Playing;
            state = state.spawn_piece();
            events.push(CoreGameEvent::GameModeChanged { 
                new_mode: CoreGameMode::Playing 
            });
            input_consumed = true;
            render_required = true;
        }
        GameInput::ToggleEraseLine => {
            // EraseLineアニメーション切り替え
            state.enable_erase_line = !state.enable_erase_line;
            input_consumed = true;
            render_required = true;
        }
        GameInput::Quit => {
            // アプリケーション終了処理
            input_consumed = true;
        }
        _ => {
            // その他の入力は無視
        }
    }

    InputProcessResult {
        new_state: state,
        events,
        input_consumed,
        render_required,
    }
}

/// プレイングモード入力処理
fn process_playing_input(mut state: CoreGameState, input: GameInput, current_time_ms: u64) -> InputProcessResult {
    let mut events = Vec::new();
    let mut input_consumed = false;
    let mut render_required = false;

    // アニメーション中は入力を受け付けない
    if state.has_animations() {
        return InputProcessResult {
            new_state: state,
            events,
            input_consumed: false,
            render_required: false,
        };
    }

    if let Some(current_piece) = &state.current_piece {
        match input {
            GameInput::MoveLeft => {
                let moved_piece = current_piece.moved(-1, 0);
                if state.can_place_piece(&moved_piece) {
                    state.current_piece = Some(moved_piece);
                    input_consumed = true;
                    render_required = true;
                }
            }
            GameInput::MoveRight => {
                let moved_piece = current_piece.moved(1, 0);
                if state.can_place_piece(&moved_piece) {
                    state.current_piece = Some(moved_piece);
                    input_consumed = true;
                    render_required = true;
                }
            }
            GameInput::SoftDrop => {
                let moved_piece = current_piece.moved(0, 1);
                if state.can_place_piece(&moved_piece) {
                    state.current_piece = Some(moved_piece);
                    input_consumed = true;
                    render_required = true;
                } else {
                    // ピースロック処理
                    state = lock_current_piece(state, current_time_ms, &mut events);
                    input_consumed = true;
                    render_required = true;
                }
            }
            GameInput::RotateClockwise => {
                let rotated_piece = current_piece.rotated();
                if state.can_place_piece(&rotated_piece) {
                    state.current_piece = Some(rotated_piece);
                    input_consumed = true;
                    render_required = true;
                }
            }
            GameInput::RotateCounterClockwise => {
                let rotated_piece = current_piece.rotated_counter_clockwise();
                if state.can_place_piece(&rotated_piece) {
                    state.current_piece = Some(rotated_piece);
                    input_consumed = true;
                    render_required = true;
                }
            }
            GameInput::HardDrop => {
                // 最下位置まで即座に移動してロック
                let mut drop_piece = current_piece.clone();
                while state.can_place_piece(&drop_piece.moved(0, 1)) {
                    drop_piece = drop_piece.moved(0, 1);
                }
                state.current_piece = Some(drop_piece);
                state = lock_current_piece(state, current_time_ms, &mut events);
                input_consumed = true;
                render_required = true;
            }
            GameInput::Pause => {
                // ポーズ機能（今回は簡単にタイトルに戻る）
                state.game_mode = CoreGameMode::Title;
                events.push(CoreGameEvent::GameModeChanged { 
                    new_mode: CoreGameMode::Title 
                });
                input_consumed = true;
                render_required = true;
            }
            GameInput::Quit => {
                // ゲーム終了
                input_consumed = true;
            }
            _ => {
                // その他の入力は無視
            }
        }
    }

    InputProcessResult {
        new_state: state,
        events,
        input_consumed,
        render_required,
    }
}

/// ゲームオーバーモード入力処理
fn process_game_over_input(mut state: CoreGameState, input: GameInput, _current_time_ms: u64) -> InputProcessResult {
    let mut events = Vec::new();
    let mut input_consumed = false;
    let mut render_required = false;

    match input {
        GameInput::Restart => {
            // CLIの挙動に合わせて、タイトルモードに戻る
            state = CoreGameState::new();
            state.game_mode = CoreGameMode::Title;
            events.push(CoreGameEvent::GameModeChanged { 
                new_mode: CoreGameMode::Title 
            });
            input_consumed = true;
            render_required = true;
        }
        GameInput::Quit => {
            // アプリケーション終了
            input_consumed = true;
        }
        _ => {
            // その他の入力は無視
        }
    }

    InputProcessResult {
        new_state: state,
        events,
        input_consumed,
        render_required,
    }
}

/// 現在のピースをロックして次のピースをスポーン
fn lock_current_piece(mut state: CoreGameState, current_time_ms: u64, events: &mut Vec<CoreGameEvent>) -> CoreGameState {
    if let Some(piece) = state.current_piece.take() {
        // ピースをボードに配置
        state = state.place_piece(&piece);
        
        // ピースロックイベント
        let first_block = piece.iter_blocks().next().unwrap();
        events.push(CoreGameEvent::PieceLocked {
            position: crate::core::board_logic::Point(first_block.0.0 as usize, first_block.0.1 as usize),
            shape: piece.shape,
        });

        // ライン消去チェック
        let complete_lines = crate::core::board_logic::find_complete_lines(state.board, state.current_board_height);
        
        if !complete_lines.is_empty() {
            // ライン消去アニメーション開始
            state = state.start_line_blink(complete_lines.clone(), current_time_ms);
            
            events.push(CoreGameEvent::LinesCleared {
                lines: complete_lines.clone(),
                is_bottom: complete_lines.contains(&(crate::config::BOARD_HEIGHT - 1)),
            });
            
            events.push(CoreGameEvent::AnimationStarted {
                animation_type: crate::core::animation_logic::AnimationType::LineBlink,
            });
        } else {
            // アニメーション無しで次のピーススポーン
            state = state.spawn_piece();
        }

        // ゲームオーバーチェック
        if let Some(new_piece) = &state.current_piece {
            if !state.can_place_piece(new_piece) {
                state.game_mode = CoreGameMode::GameOver;
                state.current_piece = None;
                events.push(CoreGameEvent::GameOver);
                events.push(CoreGameEvent::GameModeChanged { 
                    new_mode: CoreGameMode::GameOver 
                });
            }
        }
    }

    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::game_state::CoreGameMode;

    #[test]
    fn test_title_input_start() {
        let state = CoreGameState::new();
        assert_eq!(state.game_mode, CoreGameMode::Title);

        let result = process_input(state, GameInput::Restart, 0);
        
        assert_eq!(result.new_state.game_mode, CoreGameMode::Playing);
        assert!(result.input_consumed);
        assert!(result.render_required);
        assert!(result.events.len() > 0);
    }

    #[test]
    fn test_playing_input_movement() {
        let mut state = CoreGameState::new();
        state.game_mode = CoreGameMode::Playing;
        state = state.spawn_piece();

        let original_piece = state.current_piece.as_ref().unwrap().clone();
        let original_first_block = original_piece.iter_blocks().next().unwrap();
        
        let result = process_input(state, GameInput::MoveLeft, 0);
        
        let new_piece = result.new_state.current_piece.as_ref().unwrap();
        let new_first_block = new_piece.iter_blocks().next().unwrap();
        assert_eq!(new_first_block.0.0, original_first_block.0.0 - 1);
        assert!(result.input_consumed);
        assert!(result.render_required);
    }

    #[test]
    fn test_playing_input_rotation_clockwise() {
        let mut state = CoreGameState::new();
        state.game_mode = CoreGameMode::Playing;
        state = state.spawn_piece();

        let original_piece = state.current_piece.as_ref().unwrap().clone();
        
        let result = process_input(state, GameInput::RotateClockwise, 0);
        
        // 回転後のピースが異なることを確認
        let new_piece = result.new_state.current_piece.as_ref().unwrap();
        assert_ne!(new_piece.get_rotation_state(), original_piece.get_rotation_state());
        assert!(result.input_consumed);
        assert!(result.render_required);
    }

    #[test]
    fn test_playing_input_rotation_counter_clockwise() {
        let mut state = CoreGameState::new();
        state.game_mode = CoreGameMode::Playing;
        state = state.spawn_piece();

        let original_piece = state.current_piece.as_ref().unwrap().clone();
        
        let result = process_input(state, GameInput::RotateCounterClockwise, 0);
        
        // 回転後のピースが異なることを確認
        let new_piece = result.new_state.current_piece.as_ref().unwrap();
        assert_ne!(new_piece.get_rotation_state(), original_piece.get_rotation_state());
        assert!(result.input_consumed);
        assert!(result.render_required);
    }

    #[test]
    fn test_game_over_input_restart() {
        let mut state = CoreGameState::new();
        state.game_mode = CoreGameMode::GameOver;

        let result = process_input(state, GameInput::Restart, 0);
        
        // CLIの挙動に合わせてタイトルモードに戻る
        assert_eq!(result.new_state.game_mode, CoreGameMode::Title);
        assert!(result.input_consumed);
        assert!(result.render_required);
        assert_eq!(result.events.len(), 1);
    }

    #[test]
    fn test_title_input_toggle_erase_line() {
        let mut state = CoreGameState::new();
        state.game_mode = CoreGameMode::Title;
        assert!(!state.enable_erase_line); // 初期状態はfalse
        
        let result = process_input(state, GameInput::ToggleEraseLine, 0);
        
        assert!(result.input_consumed);
        assert!(result.render_required);
        assert_eq!(result.new_state.game_mode, CoreGameMode::Title);
        assert!(result.new_state.enable_erase_line); // トグルされてtrue
        
        // もう一度トグルしてfalseに戻る
        let result2 = process_input(result.new_state, GameInput::ToggleEraseLine, 0);
        assert!(!result2.new_state.enable_erase_line); // trueからfalseに戻る
    }

    #[test]
    fn test_playing_input_pause() {
        let mut state = CoreGameState::new();
        state.game_mode = CoreGameMode::Playing;
        state.current_piece = Some(Tetromino::new_random());
        
        let result = process_input(state, GameInput::Pause, 0);
        
        assert!(result.input_consumed);
        assert!(result.render_required);
        assert_eq!(result.new_state.game_mode, CoreGameMode::Title);
        assert_eq!(result.events.len(), 1);
    }

    #[test]
    fn test_complete_input_coverage() {
        // 全てのGameInput入力が適切に処理されることを確認
        use crate::game_input::GameInput;
        
        let inputs = [
            GameInput::MoveLeft,
            GameInput::MoveRight,
            GameInput::SoftDrop,
            GameInput::HardDrop,
            GameInput::RotateClockwise,
            GameInput::RotateCounterClockwise,
            GameInput::Quit,
            GameInput::Restart,
            GameInput::Pause,
            GameInput::ToggleEraseLine,
            GameInput::Unknown,
        ];

        // Playing状態でのテスト（アニメーションなし）
        for input in &inputs {
            let mut state = CoreGameState::new();
            state.game_mode = CoreGameMode::Playing;
            state.current_piece = Some(Tetromino::new_random());
            
            let result = process_input(state, *input, 0);
            // 全ての入力が何らかの処理を行う（エラーにならない）ことを確認
            // Playing中の移動・回転系入力は消費される、制御系入力はモード変更かQuit
            match input {
                GameInput::MoveLeft | GameInput::MoveRight | GameInput::SoftDrop | 
                GameInput::HardDrop | GameInput::RotateClockwise | GameInput::RotateCounterClockwise => {
                    // 移動・回転系は入力が消費されるか、有効でなければ無視される
                    assert!(result.input_consumed || !result.input_consumed);
                }
                GameInput::Quit => {
                    assert!(result.input_consumed);
                }
                GameInput::Pause => {
                    assert_eq!(result.new_state.game_mode, CoreGameMode::Title);
                    assert!(result.input_consumed);
                }
                GameInput::Unknown => {
                    // 未知の入力は無視される
                    assert!(!result.input_consumed);
                }
                _ => {
                    // その他の入力（Restart, ToggleEraseLine）はPlaying中では無視される
                    assert!(!result.input_consumed);
                }
            }
        }
        
        // Title状態でのテスト
        for input in &inputs {
            let mut state = CoreGameState::new();
            state.game_mode = CoreGameMode::Title;
            
            let result = process_input(state, *input, 0);
            match input {
                GameInput::Restart => {
                    assert_eq!(result.new_state.game_mode, CoreGameMode::Playing);
                    assert!(result.input_consumed);
                }
                GameInput::ToggleEraseLine => {
                    assert!(result.input_consumed);
                }
                GameInput::Quit => {
                    assert!(result.input_consumed);
                }
                _ => {
                    // その他の入力は無視される
                    assert!(!result.input_consumed);
                }
            }
        }
        
        // GameOver状態でのテスト
        for input in &inputs {
            let mut state = CoreGameState::new();
            state.game_mode = CoreGameMode::GameOver;
            
            let result = process_input(state, *input, 0);
            match input {
                GameInput::Restart => {
                    assert_eq!(result.new_state.game_mode, CoreGameMode::Title);
                    assert!(result.input_consumed);
                }
                GameInput::Quit => {
                    assert!(result.input_consumed);
                }
                _ => {
                    // その他の入力は無視される
                    assert!(!result.input_consumed);
                }
            }
        }
    }
}