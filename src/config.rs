use crate::game_color::GameColor;

// --- 定数 ---
pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;
pub const FALL_SPEED_START: std::time::Duration = std::time::Duration::from_millis(800);

pub const COLOR_PALETTE: [GameColor; 3] = [GameColor::Cyan, GameColor::Magenta, GameColor::Yellow];
pub const BLINK_ANIMATION_STEP: std::time::Duration = std::time::Duration::from_millis(120);
pub const BLINK_COUNT_MAX: usize = 6; // 3 blinks: on-off-on-off-on-off
pub const PUSH_DOWN_STEP_DURATION: std::time::Duration = std::time::Duration::from_millis(100);
