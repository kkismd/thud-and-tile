#[derive(Debug, Clone)]
pub struct Frame {
    pub lines: Vec<String>,
    pub requires_clear: bool,
}

impl Frame {
    pub fn new(lines: Vec<String>) -> Self {
        Self {
            lines,
            requires_clear: true,
        }
    }

    pub fn with_clear_flag(lines: Vec<String>, requires_clear: bool) -> Self {
        Self {
            lines,
            requires_clear,
        }
    }

    pub fn set_requires_clear(&mut self, requires_clear: bool) {
        self.requires_clear = requires_clear;
    }
}
