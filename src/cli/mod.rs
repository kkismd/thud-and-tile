//! CLI Layer - Command Line Interface専用モジュール
//!
//! Phase 2A: CLI Layer基盤作成
//! Layer 1 (core) の純粋関数を活用したCLI特化実装

pub mod cli_animation;
pub mod cli_game_state;
pub mod cli_input_handler_simple;
pub mod cli_renderer;
pub mod renderer;

// 公開API
pub use cli_animation::CliAnimationManager;
pub use cli_game_state::CliGameState;
pub use cli_input_handler_simple::CliInputHandler;
pub use cli_renderer::CliRenderer;
pub use renderer::CliRenderSettings;

// Core layerからの必要な型の再公開（CLI Layerでのみ必要なもの）
pub use crate::core::game_state::CoreGameEvent;
