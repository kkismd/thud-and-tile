//! CLI版の新しいメインエントリーポイント
//! 
//! 統一アーキテクチャを使用し、sleep依存を削除したイベント駆動型ゲームループを実装

use std::io::{self};
use std::time::Duration;
use crossterm::{
    cursor::{Hide, Show},
    event::{
        KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    style::{ResetColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::unified_engine::{UnifiedGameController, cli_adapter::CliGameEngine};
use crate::unified_scheduler::create_time_provider;
use crate::game_input::{GameInput, InputProvider, CrosstermInputProvider};
use crate::render;

/// 新しいCLI版メイン関数（イベント駆動版）
pub fn main_unified() -> io::Result<()> {
    // 端末の初期化
    terminal::enable_raw_mode()?;
    
    let mut stdout = io::stdout();
    execute!(stdout, PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
    ))?;
    execute!(stdout, EnterAlternateScreen, Hide)?;
    
    // レンダラーと入力プロバイダーの初期化
    let mut renderer = render::TerminalRenderer::new(stdout);
    let mut input_provider = CrosstermInputProvider::new();
    
    // 統一ゲームコントローラーの作成
    let engine = Box::new(CliGameEngine::new());
    let time_provider = create_time_provider();
    let mut controller = UnifiedGameController::new(engine, time_provider);
    
    // 初期画面表示
    render::draw_title_screen(&mut renderer)?;
    
    // メインループ（イベント駆動、sleep非依存）
    loop {
        // 入力処理（ノンブロッキング）
        if input_provider.poll_input(1)? { // 1ms timeout
            if let Some(input) = input_provider.read_input()? {
                match input {
                    GameInput::Quit => {
                        if controller.get_game_state().get_game_mode() == 2 { // GameOver
                            break; // 終了
                        }
                        controller.handle_input(input);
                    }
                    _ => {
                        controller.handle_input(input);
                    }
                }
            }
        }
        
        // ゲーム状態更新
        let update_result = controller.update();
        
        // 描画処理
        if update_result.needs_render {
            match update_result.game_mode {
                0 => {
                    // Title mode - タイトル画面は初回のみ
                }
                1 | 2 => {
                    // Playing or GameOver mode
                    // TODO: 新しい描画システムとの連携
                    render_game_state(&mut renderer, &controller)?;
                    controller.render_complete();
                }
                _ => {}
            }
        }
        
        // CPU使用率を抑えるため最小限の待機
        // （実際のゲームタイミングはスケジューラーで管理）
        std::thread::sleep(Duration::from_millis(1));
    }
    
    // 端末の復元
    execute!(renderer.stdout, PopKeyboardEnhancementFlags)?;
    execute!(renderer.stdout, Show, LeaveAlternateScreen, ResetColor)?;
    terminal::disable_raw_mode()
}

/// ゲーム状態の描画（暫定実装）
fn render_game_state(
    renderer: &mut render::TerminalRenderer<std::io::Stdout>,
    controller: &UnifiedGameController,
) -> io::Result<()> {
    // TODO: 統一ゲーム状態から描画データを取得して描画
    // 現在は最小限の実装
    
    let game_state = controller.get_game_state();
    
    // ゲームモードに応じた描画
    match game_state.get_game_mode() {
        1 => {
            // Playing mode - ゲーム画面
            // TODO: draw関数の新しい実装
            renderer.move_to(1, 1)?;
            renderer.print("Playing... (Unified Architecture)")?;
        }
        2 => {
            // GameOver mode
            renderer.move_to(1, 1)?;
            renderer.print("GAME OVER (Press Q to quit)")?;
        }
        _ => {}
    }
    
    renderer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unified_main_setup() {
        // メイン関数の初期化部分をテスト
        let engine = Box::new(CliGameEngine::new());
        let time_provider = create_time_provider();
        let controller = UnifiedGameController::new(engine, time_provider);
        
        // 初期状態の確認
        assert_eq!(controller.get_game_state().get_game_mode(), 0); // Title
    }
}