//! CLI専用描画処理（独立実装版）
//! 
//! Layer 1の描画ロジックを使用したCLI特化レンダリング
//! ターミナル出力、カラー管理、パフォーマンス最適化を含む

use crate::cli::cli_game_state::CliGameState;
use crate::cli::cli_animation::CliAnimationManager;
use crate::game_color::GameColor;
use crate::cell::Cell;
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

/// 描画統計情報
#[derive(Debug, Clone)]
pub struct RenderStats {
    pub total_frames: u64,
    pub buffer_capacity: usize,
    pub settings: CliRenderSettings,
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
            render_buffer: Vec::with_capacity(30), // 事前容量確保でパフォーマンス向上
        }
    }
    
    /// カスタム設定でCLIレンダラーを作成
    pub fn with_settings(settings: CliRenderSettings) -> Self {
        Self {
            animation_manager: CliAnimationManager::new_default(),
            settings,
            frame_count: 0,
            render_buffer: Vec::with_capacity(30), // 事前容量確保
        }
    }
    
    /// 描画統計を取得
    pub fn get_render_stats(&self) -> RenderStats {
        RenderStats {
            total_frames: self.frame_count,
            buffer_capacity: self.render_buffer.capacity(),
            settings: self.settings.clone(),
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
        // 何も描画が必要でない場合は何もしない（最適化）
        
        Ok(())
    }
    
    /// ボード部分のみ描画
    fn render_board_only(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        print!("\\x1B[H"); // カーソルを左上に移動
        self.render_game_board(cli_state)?;
        Ok(())
    }
    
    /// UI部分のみ描画
    fn render_ui_only(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        // カーソルをUI部分に移動（ボード下部）
        use crate::config::BOARD_HEIGHT;
        print!("\\x1B[{};1H", BOARD_HEIGHT + 3); // ボード下部 + 2行下
        
        // UI情報のみ描画
        self.render_ui_info(cli_state)?;
        Ok(())
    }
    
    /// バッファへ描画
    fn render_to_buffer(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        // ボード内容をバッファに格納
        use crate::config::{BOARD_HEIGHT, BOARD_WIDTH};
        
        self.render_buffer.clear();
        
        // 上端枠
        let mut top_border = "┌".to_string();
        for _ in 0..BOARD_WIDTH {
            top_border.push_str("──");
        }
        top_border.push('┐');
        self.render_buffer.push(top_border);
        
        // ボード内容
        for y in 0..BOARD_HEIGHT {
            let mut line = "│".to_string();
            for x in 0..BOARD_WIDTH {
                let cell_display = self.get_display_for_position(cli_state, x, y);
                line.push_str(&cell_display);
            }
            line.push('│');
            self.render_buffer.push(line);
        }
        
        // 下端枠
        let mut bottom_border = "└".to_string();
        for _ in 0..BOARD_WIDTH {
            bottom_border.push_str("──");
        }
        bottom_border.push('┘');
        self.render_buffer.push(bottom_border);
        
        // UI情報をバッファに追加
        self.render_buffer.push(String::new());
        self.render_buffer.push(format!("TOTAL SCORE: {}", cli_state.core.score));
        self.render_buffer.push(format!("CHAIN-BONUS: {}", cli_state.core.chain_bonus));
        self.render_buffer.push(String::new());
        self.render_buffer.push("MAX-CHAIN:".to_string());
        self.render_buffer.push(format!("  CYAN:    {}", cli_state.core.max_chains.cyan));
        self.render_buffer.push(format!("  MAGENTA: {}", cli_state.core.max_chains.magenta));
        self.render_buffer.push(format!("  YELLOW:  {}", cli_state.core.max_chains.yellow));
        self.render_buffer.push(String::new());
        self.render_buffer.push(format!("Lines: {}", cli_state.core.lines_cleared));
        self.render_buffer.push(format!("Mode: {:?}", cli_state.core.game_mode));
        
        if self.settings.show_fps {
            self.render_buffer.push(format!("FPS: {:.1}", cli_state.last_fps));
        }
        
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
    fn render_direct(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        self.render_game_board(cli_state)?;
        self.render_ui_info(cli_state)?;
        Ok(())
    }
    
    /// ゲームボード全体を描画
    fn render_game_board(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        use crate::config::{BOARD_HEIGHT, BOARD_WIDTH};
        
        // 画面クリア（必要に応じて）
        if cli_state.render_state.needs_full_redraw {
            print!("\\x1B[2J"); // 画面クリア
        }
        print!("\\x1B[H"); // カーソルを左上に移動
        
        // ボードの上端描画
        print!("┌");
        for _ in 0..BOARD_WIDTH {
            print!("──");
        }
        println!("┐");
        
        // ボード内容描画（上から下へ）
        for y in 0..BOARD_HEIGHT {
            print!("│");
            for x in 0..BOARD_WIDTH {
                let cell_display = self.get_display_for_position(cli_state, x, y);
                print!("{}", cell_display);
            }
            println!("│");
        }
        
        // ボードの下端描画
        print!("└");
        for _ in 0..BOARD_WIDTH {
            print!("──");
        }
        println!("┘");
        
        io::stdout().flush()
    }
    
    /// 指定位置の表示内容を決定（ボード + 現在ピース）
    fn get_display_for_position(&self, cli_state: &CliGameState, x: usize, y: usize) -> String {
        // 現在ピースが存在し、この位置にピースのブロックがあるかチェック
        if let Some(ref current_piece) = cli_state.core.current_piece {
            for ((piece_x, piece_y), piece_color) in current_piece.iter_blocks() {
                if piece_x >= 0 && piece_y >= 0 
                   && piece_x as usize == x && piece_y as usize == y {
                    // 現在ピースのブロックを描画
                    return self.color_to_display_string(&piece_color);
                }
            }
        }
        
        // 現在ピースがない場合は、ボードのセルを描画
        let board_cell = cli_state.core.board[y][x];
        self.cell_to_display(&board_cell)
    }
    
    /// ゲーム色を表示文字列に変換
    fn color_to_display_string(&self, color: &GameColor) -> String {
        if !self.settings.use_colors {
            "██".to_string()
        } else {
            self.color_to_ansi(color).to_string()
        }
    }
    
    /// セルを表示用文字列に変換
    fn cell_to_display(&self, cell: &Cell) -> String {
        if !self.settings.use_colors {
            // 色なしモード
            match cell {
                Cell::Empty => "  ".to_string(),
                Cell::Occupied(_) => "██".to_string(),
                Cell::Solid => "▓▓".to_string(),
                Cell::Connected { .. } => "▓▓".to_string(),
            }
        } else {
            // 色ありモード
            match cell {
                Cell::Empty => "  ".to_string(),
                Cell::Occupied(color) => {
                    let ansi_color = self.color_to_ansi(color);
                    ansi_color.to_string()
                }
                Cell::Solid => "\\x1B[47m  \\x1B[0m".to_string(), // 白背景
                Cell::Connected { color, .. } => {
                    let ansi_color = self.color_to_ansi(color);
                    ansi_color.to_string()
                }
            }
        }
    }
    
    /// UI情報を描画
    fn render_ui_info(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        println!();
        
        // T&T仕様準拠のスコア表示
        println!("TOTAL SCORE: {}", cli_state.core.score);
        println!("CHAIN-BONUS: {}", cli_state.core.chain_bonus);
        println!();
        
        // 色別MAX-CHAIN表示
        println!("MAX-CHAIN:");
        println!("  CYAN:    {}", cli_state.core.max_chains.cyan);
        println!("  MAGENTA: {}", cli_state.core.max_chains.magenta);
        println!("  YELLOW:  {}", cli_state.core.max_chains.yellow);
        println!();
        
        // 基本ゲーム情報
        println!("Lines: {}", cli_state.core.lines_cleared);
        println!("Mode: {:?}", cli_state.core.game_mode);
        
        // デバッグ情報（オプション）
        if self.settings.show_fps {
            println!("FPS: {:.1}", cli_state.last_fps);
        }
        
        if self.settings.show_debug_info {
            println!();
            println!("=== DEBUG INFO ===");
            println!("Frame: {}", cli_state.frame_count);
            println!("Events: {}", cli_state.input_event_count);
            println!("Animations: {}", cli_state.render_state.active_animations.len());
            println!("Current piece: {}", 
                if cli_state.core.current_piece.is_some() { "Yes" } else { "No" });
        }
        
        io::stdout().flush()
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