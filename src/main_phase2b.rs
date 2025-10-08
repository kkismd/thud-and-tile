//! THUD&TILE メインプログラム (Phase 2B: CLI Input統合版)
//! 
//! Phase 2B: CLI Input処理を統合したメインプログラム
//! リアルタイム入力処理とゲームコマンド変換のテスト

use std::io;
use std::time::Duration;

// CLI Layer使用（正しいアーキテクチャ）
use thud_and_tile::cli::{CliGameState, CliRenderer, CliInputHandler, CoreGameEvent};

fn main() -> io::Result<()> {
    println!("🎮 Phase 2B: CLI Input統合版 THUD&TILE開始");
    println!("⌨️  テスト用キー操作:");
    println!("   A/←: 左移動    D/→: 右移動");
    println!("   S/↓: ソフトドロップ    W/↑: 回転");
    println!("   Space: ハードドロップ");
    println!("   R: リスタート    Q: 終了");
    println!("   E: EraseLineトグル");
    println!("-----------------------------------");
    
    // Raw mode有効化（キー入力検出用）
    crossterm::terminal::enable_raw_mode()?;
    
    // CLI Layer初期化
    let mut cli_state = CliGameState::new();
    let mut cli_renderer = CliRenderer::new();
    let mut input_handler = CliInputHandler::new();
    
    println!("🚀 リアルタイム入力テストを開始します（10秒間）...");
    println!("💡 入力統計とイベント検出をテストしています");
    
    let mut frame_count = 0;
    let max_frames = 200; // 約10秒 (20 FPS想定)
    let mut total_events = 0;
    
    // メインループ（10秒間のテスト）
    while frame_count < max_frames {
        // 入力処理（Phase 2B新機能）
        match input_handler.poll_input(&mut cli_state) {
            Ok(events) => {
                total_events += events.len();
                
                // イベント処理
                for event in events {
                    // 終了シグナル検出（イベントをコピーして使用）
                    let is_game_over = matches!(event, CoreGameEvent::GameOver);
                    
                    cli_state.process_game_event(event);
                    
                    if is_game_over {
                        println!("\\n🛑 終了シグナル検出 - テスト終了");
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ 入力エラー: {}", e);
                break;
            }
        }
        
        // フレーム更新
        let _frame_delta = cli_state.update_frame();
        
        // アニメーション更新
        let _animation_events = cli_state.update_animations();
        
        // 描画（2秒ごと）- 進捗表示
        if frame_count % 40 == 0 {
            println!("⏱️  フレーム: {}/{}, 入力統計: {}, イベント合計: {}", 
                     frame_count, max_frames, input_handler.input_count, total_events);
            
            // 描画テスト（CLI Layerを通じて）
            if cli_state.needs_redraw() {
                cli_renderer.render_full(&cli_state)?;
                cli_state.mark_rendered();
            }
        }
        
        frame_count += 1;
        
        // フレームレート制限（20 FPS）
        std::thread::sleep(Duration::from_millis(50));
    }
    
    // Raw mode無効化
    crossterm::terminal::disable_raw_mode()?;
    
    // テスト結果表示
    println!("\\n✅ Phase 2B: CLI Input統合テスト完了");
    println!("📊 テスト結果:");
    println!("   総フレーム数: {}", frame_count);
    println!("   入力検出回数: {}", input_handler.input_count);
    println!("   生成イベント数: {}", total_events);
    println!("   CLI入力イベント数: {}", cli_state.input_event_count);
    
    if input_handler.input_count > 0 {
        println!("✨ 入力検出機能: 正常動作");
    } else {
        println!("ℹ️  入力検出機能: 入力なし（正常）");
    }
    
    println!("🎯 Phase 2B実装完了!");
    Ok(())
}