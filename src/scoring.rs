use crate::game_color::GameColor;
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
    #[allow(dead_code)]
    pub fn get(&self, color: GameColor) -> u32 {
        match color {
            GameColor::Cyan => self.cyan,
            GameColor::Magenta => self.magenta,
            GameColor::Yellow => self.yellow,
            _ => 0, // 他の色は対象外
        }
    }

    /// 指定された色にスコアを加算
    pub fn add(&mut self, color: GameColor, points: u32) {
        match color {
            GameColor::Cyan => self.cyan += points,
            GameColor::Magenta => self.magenta += points,
            GameColor::Yellow => self.yellow += points,
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
    pub chain_bonus: u32,
}

impl ColorMaxChains {
    pub fn new() -> Self {
        Self {
            cyan: 0,
            magenta: 0,
            yellow: 0,
            chain_bonus: 0,
        }
    }

    /// 指定された色の最大チェーン数を取得
    pub fn get(&self, color: GameColor) -> u32 {
        match color {
            GameColor::Cyan => self.cyan,
            GameColor::Magenta => self.magenta,
            GameColor::Yellow => self.yellow,
            _ => 0, // 他の色は対象外
        }
    }

    /// 指定された色の最大チェーン数を更新（現在の値より大きい場合のみ）
    pub fn update_max(&mut self, color: GameColor, chain_count: u32) {
        match color {
            GameColor::Cyan => {
                if chain_count > self.cyan {
                    self.cyan = chain_count;
                }
            }
            GameColor::Magenta => {
                if chain_count > self.magenta {
                    self.magenta = chain_count;
                }
            }
            GameColor::Yellow => {
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

    /// CHAIN-BONUSを加算（オーバーフロー防止）
    pub fn add_chain_bonus(&mut self, amount: u32) {
        self.chain_bonus = self.chain_bonus.saturating_add(amount);
    }

    /// CHAIN-BONUSを消費（指定された最大量まで消費し、実際に消費した量を返す）
    pub fn consume_chain_bonus(&mut self, max_amount: u32) -> u32 {
        let consumed = self.chain_bonus.min(max_amount);
        self.chain_bonus -= consumed;
        consumed
    }
}

/// カスタムスコアシステム全体を管理する構造体
#[derive(Clone, Debug, PartialEq)]
pub struct CustomScoreSystem {
    pub scores: ColorScores,
    pub max_chains: ColorMaxChains,
    pub total_score: u32,
}

impl CustomScoreSystem {
    pub fn new() -> Self {
        Self {
            scores: ColorScores::new(),
            max_chains: ColorMaxChains::new(),
            total_score: 0,
        }
    }

    /// 統合スコアを加算（オーバーフロー防止）
    pub fn add_total_score(&mut self, points: u32) {
        self.total_score = self.total_score.saturating_add(points);
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
        scores.add(GameColor::Cyan, 100);
        scores.add(GameColor::Magenta, 200);
        scores.add(GameColor::Yellow, 300);

        assert_eq!(scores.get(GameColor::Cyan), 100);
        assert_eq!(scores.get(GameColor::Magenta), 200);
        assert_eq!(scores.get(GameColor::Yellow), 300);
        assert_eq!(scores.total(), 600);
    }

    #[test]
    fn test_color_scores_ignore_invalid_colors() {
        let mut scores = ColorScores::new();
        scores.add(GameColor::Red, 100); // Should be ignored
        scores.add(GameColor::Blue, 200); // Should be ignored

        assert_eq!(scores.get(GameColor::Red), 0);
        assert_eq!(scores.get(GameColor::Blue), 0);
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
        max_chains.update_max(GameColor::Cyan, 3);
        max_chains.update_max(GameColor::Magenta, 5);
        max_chains.update_max(GameColor::Yellow, 2);

        assert_eq!(max_chains.get(GameColor::Cyan), 3);
        assert_eq!(max_chains.get(GameColor::Magenta), 5);
        assert_eq!(max_chains.get(GameColor::Yellow), 2);
        assert_eq!(max_chains.max(), 5);

        // より小さい値では更新されない
        max_chains.update_max(GameColor::Cyan, 2);
        max_chains.update_max(GameColor::Magenta, 4);
        assert_eq!(max_chains.get(GameColor::Cyan), 3);
        assert_eq!(max_chains.get(GameColor::Magenta), 5);

        // より大きい値では更新される
        max_chains.update_max(GameColor::Yellow, 8);
        assert_eq!(max_chains.get(GameColor::Yellow), 8);
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
        system.scores.add(GameColor::Cyan, 200);
        system.scores.add(GameColor::Magenta, 420);
        system.scores.add(GameColor::Yellow, 500);
        system.max_chains.update_max(GameColor::Cyan, 2);
        system.max_chains.update_max(GameColor::Magenta, 4);
        system.max_chains.update_max(GameColor::Yellow, 5);

        let expected = "SCORE:    1120\n  CYAN:    200\n  MAGENTA: 420\n  YELLOW:  500\n\nMAX-CHAIN: 5\n  CYAN:    2\n  MAGENTA: 4\n  YELLOW:  5";
        assert_eq!(format!("{}", system), expected);
    }

    #[test]
    fn test_color_max_chains_has_chain_bonus() {
        let max_chains = ColorMaxChains::new();
        assert_eq!(max_chains.chain_bonus, 0);
    }

    #[test]
    fn test_add_chain_bonus() {
        let mut max_chains = ColorMaxChains::new();

        // Initial state
        assert_eq!(max_chains.chain_bonus, 0);

        // Add some bonus
        max_chains.add_chain_bonus(10);
        assert_eq!(max_chains.chain_bonus, 10);

        // Add more bonus (accumulative)
        max_chains.add_chain_bonus(5);
        assert_eq!(max_chains.chain_bonus, 15);

        // Test overflow protection (saturating_add behavior)
        max_chains.chain_bonus = u32::MAX - 5;
        max_chains.add_chain_bonus(10);
        assert_eq!(max_chains.chain_bonus, u32::MAX);
    }

    #[test]
    fn test_consume_chain_bonus() {
        let mut max_chains = ColorMaxChains::new();

        // Test consuming from empty bonus
        assert_eq!(max_chains.consume_chain_bonus(10), 0);
        assert_eq!(max_chains.chain_bonus, 0);

        // Add some bonus first
        max_chains.add_chain_bonus(20);
        assert_eq!(max_chains.chain_bonus, 20);

        // Consume partial amount
        assert_eq!(max_chains.consume_chain_bonus(5), 5);
        assert_eq!(max_chains.chain_bonus, 15);

        // Consume more than available (should return available amount)
        assert_eq!(max_chains.consume_chain_bonus(25), 15);
        assert_eq!(max_chains.chain_bonus, 0);

        // Test consuming when bonus is 0
        assert_eq!(max_chains.consume_chain_bonus(10), 0);
        assert_eq!(max_chains.chain_bonus, 0);

        // Test consuming exact amount
        max_chains.add_chain_bonus(10);
        assert_eq!(max_chains.consume_chain_bonus(10), 10);
        assert_eq!(max_chains.chain_bonus, 0);
    }

    #[test]
    fn test_custom_score_system_has_total_score() {
        let system = CustomScoreSystem::new();
        assert_eq!(system.total_score, 0);
    }

    #[test]
    fn test_add_total_score() {
        let mut system = CustomScoreSystem::new();

        // Initial state
        assert_eq!(system.total_score, 0);

        // Add some score
        system.add_total_score(100);
        assert_eq!(system.total_score, 100);

        // Add more score (accumulative)
        system.add_total_score(50);
        assert_eq!(system.total_score, 150);

        // Test overflow protection (saturating_add behavior)
        system.total_score = u32::MAX - 25;
        system.add_total_score(50);
        assert_eq!(system.total_score, u32::MAX);
    }
}
