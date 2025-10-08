//! CLI Layer統合実行例
//! 
//! Phase 2A: CLI Layerを使用したメインプログラムの実装例
//! 既存main.rsをCLI Layer構造に移行

use std::io;
use std::time::Duration;

// CLI Layer使用
use thud_and_tile::cli::{CliGameState, CliInputHandler, CliRenderer, CliRenderSettings};

fn main() -> io::Result<()> {
    // CLI Layer初期化
    let mut cli_state = CliGameState::new();
    let mut input_handler = CliInputHandler::new();
    let mut renderer = CliRenderer::with_settings(CliRenderSettings {
        show_debug_info: true,
        show_fps: true,
        show_animation_info: true,
        use_colors: true,
        double_buffering: true,
    });
    
    println!("Phase 2A: CLI Layer テスト開始");
    println!("Ctrl+C で終了");
    
    // メインゲームループ（簡略版）
    loop {
        // フレーム更新
        let frame_delta = cli_state.update_frame();
        
        // 入力処理
        if let Ok(events) = input_handler.poll_input(&mut cli_state) {
            if !events.is_empty() {
                println!("イベント発生: {:?}", events);
            }
        }
        
        // アニメーション更新
        let animation_events = cli_state.update_animations();
        if !animation_events.is_empty() {
            println!("アニメーションイベント: {:?}", animation_events);
        }
        
        // 描画（簡略版）
        if cli_state.needs_redraw() {
            println!("--- CLI Layer Status ---");
            println!("Frame: {}", cli_state.frame_count);
            println!("FPS: {:.1}", cli_state.last_fps);
            println!("Core Score: {}", cli_state.core.score);
            println!("Animations: {}", cli_state.core.animations_count);
            
            cli_state.mark_rendered();
        }
        
        // フレームレート制限
        if frame_delta < Duration::from_millis(16) { // 60 FPS制限
            std::thread::sleep(Duration::from_millis(16) - frame_delta);
        }
        
        // 簡易終了条件（実際の実装では入力ハンドリング）
        if cli_state.frame_count > 300 { // 5秒程度でテスト終了
            break;
        }
    }
    
    println!("Phase 2A: CLI Layer テスト完了");
    Ok(())
}