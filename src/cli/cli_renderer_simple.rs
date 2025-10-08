//! CLI専用描画処理（独立実装版）
//! 
//! Layer 1の描画ロジックを使用したCLI特化レンダリング
//! ターミナル出力、カラー管理、パフォーマンス最適化を含む

// use crate::core::game_state::CoreG
use crate::cli::cli_game_state::CliGameState;
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
    /// 標準設定でCLIレンダラーを作成
    pub fn new() -> Self {
        Self {
            animation_manager: CliAnimationManager::new_default(),
            settings: CliRenderSettings::default(),
            frame_count: 0,
            render_buffer: Vec::new(),
        }
    }
    
    /// カスタム設定でCLIレンダラーを作成
    pub fn with_settings(settings: CliRenderSettings) -> Self {
        Self {
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
    fn render_board_only(&mut self, _cli_state: &CliGameState) -> io::Result<()> {
        print!("\\x1B[H"); // カーソルを左上に移動
        println!("Board rendering...");
        Ok(())
    }
    
    /// UI部分のみ描画
    fn render_ui_only(&mut self, _cli_state: &CliGameState) -> io::Result<()> {
        println!("UI rendering...");
        Ok(())
    }
    
    /// バッファへ描画
    fn render_to_buffer(&mut self, _cli_state: &CliGameState) -> io::Result<()> {
        self.render_buffer.push("Buffer rendering...".to_string());
        Ok(())
    }
    
    /// バッファをフラッシュ
    fn flush_buffer(&mut self) -> io::Result<()> {
        for line in &self.render_buffer {
            println!("{}", line);
        }
        io::stdout().flush()
    }
    
    /// 直接描画
    fn render_direct(&mut self, _cli_state: &CliGameState) -> io::Result<()> {
        println!("Direct rendering...");
        Ok(())
    }
    
    /// 色をANSIエスケープコードに変換
    fn color_to_ansi(&self, color: &GameColor) -> &'static str {
        match color {
            GameColor::Red => "\\x1B[41m  \\x1B[0m",
            GameColor::Green => "\\x1B[42m  \\x1B[0m",
            GameColor::Blue => "\\x1B[44m  \\x1B[0m",
            GameColor::Yellow => "\\x1B[43m  \\x1B[0m",
            GameColor::Cyan => "\\x1B[46m  \\x1B[0m",
            GameColor::Magenta => "\\x1B[45m  \\x1B[0m",
            GameColor::Grey => "\\x1B[47m  \\x1B[0m",
            _ => "\\x1B[40m  \\x1B[0m", // デフォルト（黒背景）
        }
    }
}

impl Default for CliRenderer {
    fn default() -> Self {
        Self::new()
    }
}