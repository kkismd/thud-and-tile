use crossterm::style::Color;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct CustomScore {
    pub scores: HashMap<Color, u32>,
    pub max_chains: HashMap<Color, u32>,
}

impl CustomScore {
    pub fn new() -> Self {
        let mut scores = HashMap::new();
        let mut max_chains = HashMap::new();
        // シアン、マゼンタ、イエローの3色を初期化
        scores.insert(Color::Cyan, 0);
        scores.insert(Color::Magenta, 0);
        scores.insert(Color::Yellow, 0);
        max_chains.insert(Color::Cyan, 0);
        max_chains.insert(Color::Magenta, 0);
        max_chains.insert(Color::Yellow, 0);
        Self { scores, max_chains }
    }
}
