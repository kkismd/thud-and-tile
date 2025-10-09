//! CLI専用描画処理（Phase 2D再設計版）
//!
//! 旧CLI版の表示仕様を忠実に維持しながら、FrameBuilderとTerminalDriverで描画責務を分割する。

use std::io;

use crate::cli::cli_animation::CliAnimationManager;
use crate::cli::cli_game_state::CliGameState;
use crate::cli::renderer::{CliRenderSettings, FrameBuilder, TerminalDriver};

/// 描画統計情報
#[derive(Debug, Clone)]
pub struct RenderStats {
    pub total_frames: u64,
    pub last_frame_lines: Option<usize>,
    pub settings: CliRenderSettings,
}

/// CLI版レンダラー（責務分離後）
pub struct CliRenderer {
    _animation_manager: CliAnimationManager,
    pub settings: CliRenderSettings,
    frame_count: u64,
    frame_builder: FrameBuilder,
    terminal_driver: TerminalDriver,
    last_frame_lines: Option<usize>,
}

impl CliRenderer {
    /// 標準設定でCLIレンダラーを作成
    pub fn new() -> Self {
        Self {
            _animation_manager: CliAnimationManager::new_default(),
            settings: CliRenderSettings::default(),
            frame_count: 0,
            frame_builder: FrameBuilder::new(),
            terminal_driver: TerminalDriver::new(),
            last_frame_lines: None,
        }
    }

    /// カスタム設定でCLIレンダラーを作成
    pub fn with_settings(settings: CliRenderSettings) -> Self {
        Self {
            _animation_manager: CliAnimationManager::new_default(),
            settings,
            frame_count: 0,
            frame_builder: FrameBuilder::new(),
            terminal_driver: TerminalDriver::new(),
            last_frame_lines: None,
        }
    }

    /// 描画統計を取得
    pub fn get_render_stats(&self) -> RenderStats {
        RenderStats {
            total_frames: self.frame_count,
            last_frame_lines: self.last_frame_lines,
            settings: self.settings.clone(),
        }
    }

    /// CLI特化: 完全描画
    pub fn render_full(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        self.frame_count += 1;

        // TODO: animation_managerによる描画拡張を統合する（Phase 2D後半）
        let mut frame = self
            .frame_builder
            .build_full_frame(cli_state, &self.settings);
        frame.set_requires_clear(true);

        self.last_frame_lines = Some(frame.lines.len());
        self.terminal_driver
            .present_full_frame(frame, &self.settings)
    }

    /// CLI特化: 部分描画（Phase 2D初期はフル描画フォールバック）
    pub fn render_incremental(&mut self, cli_state: &CliGameState) -> io::Result<()> {
        self.frame_count += 1;

        let mut frame = self
            .frame_builder
            .build_full_frame(cli_state, &self.settings);
        frame.set_requires_clear(false);

        self.last_frame_lines = Some(frame.lines.len());
        self.terminal_driver
            .present_incremental_frame(frame, &self.settings)
    }
}

impl Default for CliRenderer {
    fn default() -> Self {
        Self::new()
    }
}
