//! Core Logic Module
//! 
//! CLI版とWASM版で共有する純粋関数群
//! 借用チェッカー競合を完全に回避する設計

pub mod animation_logic;
pub mod board_logic;
pub mod game_state;

// 公開API
pub use animation_logic::*;
pub use board_logic::*;
pub use game_state::*;