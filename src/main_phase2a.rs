//! THUD&TILE メインプログラム (Phase 2A: CLI Layer使用版)
//! 
//! Phase 2A: CLI Layerを使用したメインプログラムの実装
//! 3-layer architectureによる分離: Core → CLI → Main

use std::io;

// CLI Layer使用（正しいアーキテクチャ）
use thud_and_tile::cli::{CliGameState, CliRenderer};

fn main() -> io::Result<()> {
    println!("Phase 2A: CLI Layer版 THUD&TILE開始");
    
    // CLI Layer初期化（正しいアーキテクチャ）
    let mut cli_state = CliGameState::new();
    let mut cli_renderer = CliRenderer::new();
    
    println!("5秒間のテスト実行を開始します...");
    
    // 簡単なテストループ（5秒間）
    for frame in 0..100 { // 約5秒 (20 FPS想定)
        // フレーム更新
        let _frame_delta = cli_state.update_frame();
        
        // アニメーション更新
        let _animation_events = cli_state.update_animations();
        
        // 描画（1秒ごと）- CLI Layerを通じて
        if frame % 20 == 0 {
            cli_renderer.render_full(&cli_state)?;
        }
        
        // フレームレート制限（20 FPS）
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    
    println!("Phase 2A: CLI Layer テスト完了");
    Ok(())
}