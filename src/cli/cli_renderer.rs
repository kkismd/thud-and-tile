//! CLI専用描画処理（独立実装版）
//! 
//! Layer 1の描画ロジックを使用したCLI特化レンダリング
//! ターミナル出力、カラー管理、パフォーマンス最適化を含む

use crate::core::game_state::CoreGameState;
use crate::cli::cli_game_state::{CliGameState, CliRenderState};
use crate::cli::cli_animation::CliAnimationManager;
use crate::game_color::GameColor;
use std::io::{self, Write};

/// CLI版レンダラー（独立実装）
pub struct CliRenderer {
    /// CLI特化: アニメーション管理
    animation_manager: CliAnimationManager,
    
    /// CLI特化: 描画設定
    pub settings: CliRenderSettings,
    
    /// CLI特化: パフォーマンス管理
    frame_count: u64,
    render_buffer: Vec<String>,
}

/// CLI描画設定
#[derive(Debug, Clone)]
pub struct CliRenderSettings {
    pub show_debug_info: bool,
    pub show_fps: bool,
    pub show_animation_info: bool,
    pub use_colors: bool,
    pub double_buffering: bool,
}

impl Default for CliRenderSettings {
    fn default() -> Self {
        Self {
            show_debug_info: false,
            show_fps: true,
            show_animation_info: false,
            use_colors: true,
            double_buffering: true,
        }
    }
}

impl CliRenderer {
    /// 新しいCLIレンダラーを作成
    pub fn new() -> Self {
        Self {
            base_renderer: Renderer::new(),
            animation_manager: CliAnimationManager::new_default(),
            settings: CliRenderSettings::default(),
            frame_count: 0,
            render_buffer: Vec::new(),
        }
    }
    
    /// カスタム設定でCLIレンダラーを作成
    pub fn with_settings(settings: CliRenderSettings) -> Self {
        Self {
            base_renderer: Renderer::new(),
            animation_manager: CliAnimationManager::new_default(),
            settings,
            frame_count: 0,
            render_buffer: Vec::new(),
        }
    }
    
    /// CLI特化: 完全描画
    pub fn render_full(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        self.frame_count += 1;
        
        if self.settings.double_buffering {
            self.render_buffer.clear();
            self.render_to_buffer(cli_state)?;
            self.flush_buffer()?;
        } else {
            self.render_direct(cli_state)?;
        }
        
        Ok(())
    }
    
    /// CLI特化: 部分描画（最適化）
    pub fn render_incremental(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        if cli_state.render_state.needs_full_redraw {
            self.render_full(cli_state)?;
        } else if cli_state.render_state.needs_board_redraw {
            self.render_board_only(cli_state)?;
        } else if cli_state.render_state.needs_ui_redraw {
            self.render_ui_only(cli_state)?;
        }
        
        Ok(())
    }
    
    /// ボード部分のみ描画
    fn render_board_only(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        print!("\x1B[H"); // カーソルを左上に移動
        
        let current_time_ms = cli_state.time_provider.now_ms();
        
        // Layer 1のボード状態を使用
        self.render_board_with_animations(&cli_state.core, current_time_ms)?;
        
        io::stdout().flush()?;
        Ok(())
    }
    
    /// UI部分のみ描画
    fn render_ui_only(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        // スコア、FPS、統計情報の描画
        self.render_stats(cli_state)?;
        
        if self.settings.show_animation_info {
            self.render_animation_info(cli_state)?;
        }
        
        io::stdout().flush()?;
        Ok(())
    }
    
    /// バッファに描画
    fn render_to_buffer(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        // ヘッダー情報
        if self.settings.show_debug_info {
            self.render_buffer.push(format!("Frame: {}", self.frame_count));
        }
        
        // メインゲーム画面
        let current_time_ms = cli_state.time_provider.now_ms();
        self.render_game_screen_to_buffer(&cli_state.core, current_time_ms)?;
        
        // フッター情報
        if self.settings.show_fps {
            self.render_buffer.push(format!("FPS: {:.1}", cli_state.last_fps));
        }
        
        Ok(())
    }
    
    /// バッファの内容を出力
    fn flush_buffer(&mut self) -> io::Result<()> {
        print!("\x1B[2J\x1B[H"); // 画面クリア＋カーソルリセット
        
        for line in &self.render_buffer {
            println!("{}", line);
        }
        
        io::stdout().flush()?;
        Ok(())
    }
    
    /// 直接描画
    fn render_direct(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        print!("\x1B[2J\x1B[H"); // 画面クリア＋カーソルリセット
        
        let current_time_ms = cli_state.time_provider.now_ms();
        
        // 既存レンダラーを活用
        self.base_renderer.render(&cli_state.core.board, &cli_state.core.current_piece);
        
        // CLI特化情報の追加描画
        self.render_cli_specific_info(cli_state)?;
        
        io::stdout().flush()?;
        Ok(())
    }
    
    /// アニメーション付きボード描画
    fn render_board_with_animations(&mut self, core_state: &CoreGameState, current_time_ms: u64) -> io::Result<()> {
        for y in 0..20 { // BOARD_HEIGHT
            for x in 0..10 { // BOARD_WIDTH
                let should_render = self.animation_manager.should_render_line(y, core_state, current_time_ms);
                
                if should_render {
                    self.render_cell(core_state.board[y][x], x, y)?;
                } else {
                    self.render_animated_cell(x, y, current_time_ms)?;
                }
            }
            println!(); // 行終了
        }
        
        Ok(())
    }
    
    /// セル描画
    fn render_cell(&self, cell: crate::cell::Cell, _x: usize, _y: usize) -> io::Result<()> {
        use crate::cell::Cell;
        
        let char_repr = match cell {
            Cell::Empty => "  ",
            Cell::Occupied(color) => self.get_color_char(color),
            Cell::Solid => "██",
            Cell::Connected { color, .. } => self.get_color_char(color),
        };
        
        print!("{}", char_repr);
        Ok(())
    }
    
    /// アニメーション中のセル描画
    fn render_animated_cell(&self, _x: usize, _y: usize, current_time_ms: u64) -> io::Result<()> {
        // 点滅効果
        let blink_phase = (current_time_ms / 120) % 2;
        let char_repr = if blink_phase == 0 { "██" } else { "  " };
        
        print!("{}", char_repr);
        Ok(())
    }
    
    /// 色文字取得
    fn get_color_char(&self, color: GameColor) -> &'static str {
        if !self.settings.use_colors {
            return "██";
        }
        
        match color {
            GameColor::Red => "\x1B[41m  \x1B[0m",
            GameColor::Green => "\x1B[42m  \x1B[0m",
            GameColor::Blue => "\x1B[44m  \x1B[0m",
            GameColor::Cyan => "\x1B[46m  \x1B[0m",
            GameColor::Magenta => "\x1B[45m  \x1B[0m",
            GameColor::Yellow => "\x1B[43m  \x1B[0m",
            GameColor::Grey => "\x1B[47m  \x1B[0m",
            GameColor::DarkGrey => "\x1B[100m  \x1B[0m",
        }
    }
    
    /// ゲーム画面をバッファに描画
    fn render_game_screen_to_buffer(&mut self, core_state: &CoreGameState, current_time_ms: u64) -> io::Result<()> {
        // 実装簡略化: 既存レンダラーの出力をキャプチャして追加
        self.render_buffer.push("=== GAME BOARD ===".to_string());
        
        // ボード状態の文字列表現を追加
        for y in 0..20 {
            let mut line = String::new();
            for x in 0..10 {
                let should_render = self.animation_manager.should_render_line(y, core_state, current_time_ms);
                line.push_str(if should_render { "██" } else { "  " });
            }
            self.render_buffer.push(line);
        }
        
        Ok(())
    }
    
    /// CLI特化情報描画
    fn render_cli_specific_info(&self, cli_state: &CliGameState) -> io::Result<()> {
        println!("\n--- CLI Info ---");
        
        if self.settings.show_fps {
            println!("FPS: {:.1}", cli_state.last_fps);
        }
        
        if self.settings.show_debug_info {
            println!("Frame: {}", self.frame_count);
            println!("Animations: {}", cli_state.core.animations_count);
        }
        
        Ok(())
    }
    
    /// 統計情報描画
    fn render_stats(&self, cli_state: &CliGameState) -> io::Result<()> {
        println!("Score: {}", cli_state.core.score);
        println!("Lines: {}", cli_state.core.lines_cleared);
        println!("Chain: {}", cli_state.core.chain_bonus);
        Ok(())
    }
    
    /// アニメーション情報描画
    fn render_animation_info(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        let active_animations = self.animation_manager.get_active_animations(&cli_state.core);
        
        if !active_animations.is_empty() {
            println!("\nActive Animations:");
            for anim_type in active_animations {
                println!("  {:?}", anim_type);
            }
        }
        
        Ok(())
    }
    
    /// 設定更新
    pub fn update_settings(&mut self, new_settings: CliRenderSettings) {
        self.settings = new_settings;
    }
    
    /// パフォーマンス統計取得
    pub fn get_render_stats(&self) -> CliRenderStats {
        CliRenderStats {
            frame_count: self.frame_count,
            buffer_size: self.render_buffer.len(),
            double_buffering: self.settings.double_buffering,
        }
    }
}

/// CLI描画統計情報
#[derive(Debug, Clone)]
pub struct CliRenderStats {
    pub frame_count: u64,
    pub buffer_size: usize,
    pub double_buffering: bool,
}

impl Default for CliRenderer {
    fn default() -> Self {
        Self::new()
    }
}