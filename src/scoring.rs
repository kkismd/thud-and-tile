use crossterm::style::Color;
use std::fmt;

/// 色別のスコアとMAX-CHAINを管理する構造体
#[derive(Clone, Debug, PartialEq)]
pub struct ColorScores {
    pub cyan: u32,
    pub magenta: u32,
    pub yellow: u32,
}

impl ColorScores {
    pub fn new() -> Self {
        Self {
            cyan: 0,
            magenta: 0,
            yellow: 0,
        }
    }

    /// 指定された色のスコアを取得
    pub fn get(&self, color: Color) -> u32 {
        match color {
            Color::Cyan => self.cyan,
            Color::Magenta => self.magenta,
            Color::Yellow => self.yellow,
            _ => 0, // 他の色は対象外
        }
    }

    /// 指定された色にスコアを加算
    pub fn add(&mut self, color: Color, points: u32) {
        match color {
            Color::Cyan => self.cyan += points,
            Color::Magenta => self.magenta += points,
            Color::Yellow => self.yellow += points,
            _ => {} // 他の色は何もしない
        }
    }

    /// 合計スコアを計算
    pub fn total(&self) -> u32 {
        self.cyan + self.magenta + self.yellow
    }
}

/// 色別の最大チェーン数を管理する構造体
#[derive(Clone, Debug, PartialEq)]
pub struct ColorMaxChains {
    pub cyan: u32,
    pub magenta: u32,
    pub yellow: u32,
}

impl ColorMaxChains {
    pub fn new() -> Self {
        Self {
            cyan: 0,
            magenta: 0,
            yellow: 0,
        }
    }

    /// 指定された色の最大チェーン数を取得
    pub fn get(&self, color: Color) -> u32 {
        match color {
            Color::Cyan => self.cyan,
            Color::Magenta => self.magenta,
            Color::Yellow => self.yellow,
            _ => 0, // 他の色は対象外
        }
    }

    /// 指定された色の最大チェーン数を更新（現在の値より大きい場合のみ）
    pub fn update_max(&mut self, color: Color, chain_count: u32) {
        match color {
            Color::Cyan => {
                if chain_count > self.cyan {
                    self.cyan = chain_count;
                }
            }
            Color::Magenta => {
                if chain_count > self.magenta {
                    self.magenta = chain_count;
                }
            }
            Color::Yellow => {
                if chain_count > self.yellow {
                    self.yellow = chain_count;
                }
            }
            _ => {} // 他の色は何もしない
        }
    }

    /// 最大チェーン数を取得
    pub fn max(&self) -> u32 {
        self.cyan.max(self.magenta).max(self.yellow)
    }
}

/// カスタムスコアシステム全体を管理する構造体
#[derive(Clone, Debug, PartialEq)]
pub struct CustomScoreSystem {
    pub scores: ColorScores,
    pub max_chains: ColorMaxChains,
}

impl CustomScoreSystem {
    pub fn new() -> Self {
        Self {
            scores: ColorScores::new(),
            max_chains: ColorMaxChains::new(),
        }
    }
}

/// スコア表示用のフォーマット実装
impl fmt::Display for CustomScoreSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "SCORE:    {}", self.scores.total())?;
        writeln!(f, "  CYAN:    {}", self.scores.cyan)?;
        writeln!(f, "  MAGENTA: {}", self.scores.magenta)?;
        writeln!(f, "  YELLOW:  {}", self.scores.yellow)?;
        writeln!(f)?;
        writeln!(f, "MAX-CHAIN: {}", self.max_chains.max())?;
        writeln!(f, "  CYAN:    {}", self.max_chains.cyan)?;
        writeln!(f, "  MAGENTA: {}", self.max_chains.magenta)?;
        write!(f, "  YELLOW:  {}", self.max_chains.yellow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scores_initialization() {
        let scores = ColorScores::new();
        assert_eq!(scores.cyan, 0);
        assert_eq!(scores.magenta, 0);
        assert_eq!(scores.yellow, 0);
        assert_eq!(scores.total(), 0);
    }

    #[test]
    fn test_color_scores_add() {
        let mut scores = ColorScores::new();
        scores.add(Color::Cyan, 100);
        scores.add(Color::Magenta, 200);
        scores.add(Color::Yellow, 300);

        assert_eq!(scores.get(Color::Cyan), 100);
        assert_eq!(scores.get(Color::Magenta), 200);
        assert_eq!(scores.get(Color::Yellow), 300);
        assert_eq!(scores.total(), 600);
    }

    #[test]
    fn test_color_scores_ignore_invalid_colors() {
        let mut scores = ColorScores::new();
        scores.add(Color::Red, 100); // Should be ignored
        scores.add(Color::Blue, 200); // Should be ignored

        assert_eq!(scores.get(Color::Red), 0);
        assert_eq!(scores.get(Color::Blue), 0);
        assert_eq!(scores.total(), 0);
    }

    #[test]
    fn test_color_max_chains_initialization() {
        let max_chains = ColorMaxChains::new();
        assert_eq!(max_chains.cyan, 0);
        assert_eq!(max_chains.magenta, 0);
        assert_eq!(max_chains.yellow, 0);
        assert_eq!(max_chains.max(), 0);
    }

    #[test]
    fn test_color_max_chains_update() {
        let mut max_chains = ColorMaxChains::new();

        // 初回設定
        max_chains.update_max(Color::Cyan, 3);
        max_chains.update_max(Color::Magenta, 5);
        max_chains.update_max(Color::Yellow, 2);

        assert_eq!(max_chains.get(Color::Cyan), 3);
        assert_eq!(max_chains.get(Color::Magenta), 5);
        assert_eq!(max_chains.get(Color::Yellow), 2);
        assert_eq!(max_chains.max(), 5);

        // より小さい値では更新されない
        max_chains.update_max(Color::Cyan, 2);
        max_chains.update_max(Color::Magenta, 4);
        assert_eq!(max_chains.get(Color::Cyan), 3);
        assert_eq!(max_chains.get(Color::Magenta), 5);

        // より大きい値では更新される
        max_chains.update_max(Color::Yellow, 8);
        assert_eq!(max_chains.get(Color::Yellow), 8);
        assert_eq!(max_chains.max(), 8);
    }

    #[test]
    fn test_custom_score_system_initialization() {
        let system = CustomScoreSystem::new();
        assert_eq!(system.scores.total(), 0);
        assert_eq!(system.max_chains.max(), 0);
    }

    #[test]
    fn test_custom_score_system_display() {
        let mut system = CustomScoreSystem::new();
        system.scores.add(Color::Cyan, 200);
        system.scores.add(Color::Magenta, 420);
        system.scores.add(Color::Yellow, 500);
        system.max_chains.update_max(Color::Cyan, 2);
        system.max_chains.update_max(Color::Magenta, 4);
        system.max_chains.update_max(Color::Yellow, 5);

        let expected = "SCORE:    1120\n  CYAN:    200\n  MAGENTA: 420\n  YELLOW:  500\n\nMAX-CHAIN: 5\n  CYAN:    2\n  MAGENTA: 4\n  YELLOW:  5";
        assert_eq!(format!("{}", system), expected);
    }
}
