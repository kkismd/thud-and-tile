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
