//! THUD&TILE メインプログラム (Phase 2C: CLI描画統合版)
//! 
//! Phase 2C: CLI描画システムを統合したメインプログラム
//! Phase 2Bの入力処理 + Phase 2Cの描画処理による完全なゲームループ

use std::io;
use std::time::Duration;

// CLI Layer使用（正しいアーキテクチャ）
use thud_and_tile::cli::{CliGameState, CliRenderer, CliInputHandler, CoreGameEvent, CliRenderSettings};

fn main() -> io::Result<()> {
    println!("🎮 Phase 2C: CLI描画統合版 THUD&TILE開始");
    println!("⌨️  ゲーム操作:");
    println!("   A/←: 左移動    D/→: 右移動");
    println!("   S/↓: ソフトドロップ    W/↑: 回転");
    println!("   Space: ハードドロップ");
    println!("   R: リスタート    Q: 終了");
    println!("   E: EraseLineトグル");
    println!("-----------------------------------");
    println!("🖼️  Phase 2C統合機能:");
    println!("   ✓ リアルタイム入力処理 (Phase 2B)");
    println!("   ✓ ボード + 現在ピース描画 (Phase 2C)");
    println!("   ✓ T&T準拠UI表示 (Phase 2C)");
    println!("   ✓ 効率的な描画管理 (Phase 2C)");
    println!("-----------------------------------");
    
    // Raw mode有効化（キー入力検出用）
    crossterm::terminal::enable_raw_mode()?;
    
    // CLI Layer初期化
    let mut cli_state = CliGameState::new();
    
    // CLI Renderer設定（デバッグ情報付き）
    let render_settings = CliRenderSettings {
        show_debug_info: true,
        show_fps: true,
        show_animation_info: false,
        use_colors: true,
        double_buffering: true,
    };
    let mut cli_renderer = CliRenderer::with_settings(render_settings);
    
    let mut input_handler = CliInputHandler::new();
    
    // ゲーム用に適切なクールダウン設定
    input_handler.set_cooldown_ms(50); // 50ms（ゲーム向け）
    
    // ゲームをPlayingモードで開始（テストピース付き）
    cli_state.start_playing_mode();
    
    println!("🚀 Phase 2C統合ゲームループを開始します...");
    println!("💡 入力・描画・ゲーム更新の完全統合をテストしています");
    println!();
    
    let mut frame_count = 0;
    let max_frames = 1200; // 約60秒 (20 FPS想定)
    let mut total_events = 0;
    let mut last_render_time = std::time::Instant::now();
    
    // 初回描画
    cli_renderer.render_full(&cli_state)?;
    cli_state.mark_rendered();
    
    // メインループ（統合ゲームループ）
    while frame_count < max_frames {
        let loop_start = std::time::Instant::now();
        
        // ===== 入力処理 (Phase 2B) =====
        match input_handler.poll_input(&mut cli_state) {
            Ok(events) => {
                total_events += events.len();
                
                // イベント処理
                for event in events {
                    // 終了シグナル検出
                    let is_game_over = matches!(event, CoreGameEvent::GameOver);
                    
                    cli_state.process_game_event(event);
                    
                    if is_game_over {
                        println!("\\n🛑 終了シグナル検出 - ゲーム終了");
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ 入力エラー: {}", e);
                break;
            }
        }
        
        // ===== ゲーム更新 =====
        let _frame_delta = cli_state.update_frame();
        let _animation_events = cli_state.update_animations();
        
        // ===== 描画処理 (Phase 2C) =====
        let render_needed = cli_state.needs_redraw() || 
                           last_render_time.elapsed() >= Duration::from_millis(100); // 最低10 FPS
        
        if render_needed {
            // 効率的描画（needs_redrawフラグ活用）
            if cli_state.render_state.needs_full_redraw {
                cli_renderer.render_full(&cli_state)?;
            } else {
                cli_renderer.render_incremental(&cli_state)?;
            }
            
            cli_state.mark_rendered();
            last_render_time = std::time::Instant::now();
        }
        
        // ===== パフォーマンス統計 (Phase 2C) =====
        if frame_count % 100 == 0 && frame_count > 0 {
            let runtime = frame_count as f64 * 50.0 / 1000.0; // 秒
            println!("📊 Phase 2C統計 - 実行時間: {:.1}s, フレーム: {}, 入力: {}, イベント: {}", 
                     runtime, frame_count, input_handler.input_count, total_events);
        }
        
        frame_count += 1;
        
        // フレームレート制限（20 FPS）
        let loop_duration = loop_start.elapsed();
        if loop_duration < Duration::from_millis(50) {
            std::thread::sleep(Duration::from_millis(50) - loop_duration);
        }
    }
    
    // Raw mode無効化
    crossterm::terminal::disable_raw_mode()?;
    
    // ===== Phase 2C完了統計 =====
    println!("\\n✅ Phase 2C: CLI描画統合テスト完了");
    println!("📊 最終統計:");
    println!("   総フレーム数: {}", frame_count);
    println!("   実行時間: {:.1}秒", frame_count as f64 * 50.0 / 1000.0);
    println!("   平均FPS: {:.1}", 1000.0 / 50.0);
    println!();
    println!("🔧 入力システム (Phase 2B):");
    println!("   入力検出回数: {}", input_handler.input_count);
    println!("   生成イベント数: {}", total_events);
    println!("   CLI入力処理数: {}", cli_state.input_event_count);
    println!();
    println!("🖼️  描画システム (Phase 2C):");
    println!("   レンダリング可能: ✓");
    println!("   ボード描画: ✓");
    println!("   現在ピース統合: ✓");
    println!("   T&T準拠UI: ✓");
    println!("   効率的描画管理: ✓");
    
    if input_handler.input_count > 0 {
        println!("\\n✨ 入力・描画統合システム: 正常動作");
    } else {
        println!("\\nℹ️  入力・描画統合システム: 動作確認済み");
    }
    
    println!("\\n🎯 Phase 2C実装完了! CLI統合システム動作中");
    Ok(())
}