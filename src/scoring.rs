use crate::game_color::GameColor;
use std::fmt;

/// 総合スコアのみを管理する構造体
#[derive(Clone, Debug, PartialEq)]
pub struct TotalScore {
    total: u32,
}

impl TotalScore {
    pub fn new() -> Self {
        Self { total: 0 }
    }

    /// スコアを加算
    pub fn add(&mut self, points: u32) {
        self.total += points;
    }

    /// 合計スコアを取得
    pub fn total(&self) -> u32 {
        self.total
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
}

/// カスタムスコアシステム全体を管理する構造体
#[derive(Clone, Debug, PartialEq)]
pub struct CustomScoreSystem {
    pub score: TotalScore,
    pub max_chains: ColorMaxChains,
    pub chain_bonus: u32,
}

impl CustomScoreSystem {
    pub fn new() -> Self {
        Self {
            score: TotalScore::new(),
            max_chains: ColorMaxChains::new(),
            chain_bonus: 0,
        }
    }

    pub fn add_score(&mut self, points: u32) {
        self.score.add(points);
    }

    /// chain_bonusに段数を加算する。上限は10段。
    pub fn add_chain_bonus(&mut self, lines: u32) {
        self.chain_bonus = (self.chain_bonus + lines).min(10);
    }

    /// chain_bonusから指定段数を消費する。足りない場合は0になる。
    pub fn consume_chain_bonus(&mut self, lines: u32) -> u32 {
        let consumed = self.chain_bonus.min(lines);
        self.chain_bonus -= consumed;
        consumed
    }

    /// 現在の盤面から算出されたボーナス段数でCHAIN-BONUSを上書きする（上限10）。
    pub fn set_chain_bonus_from_total(&mut self, total_bonus: u32) {
        self.chain_bonus = total_bonus.min(10);
    }
}

/// スコア表示用のフォーマット実装
impl fmt::Display for CustomScoreSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "SCORE:    {}", self.score.total())?;
        writeln!(f)?;
        writeln!(f, "MAX-CHAIN:")?;
        writeln!(f, "  CYAN:    {}", self.max_chains.cyan)?;
        writeln!(f, "  MAGENTA: {}", self.max_chains.magenta)?;
        write!(f, "  YELLOW:  {}", self.max_chains.yellow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_score_initialization() {
        let score = TotalScore::new();
        assert_eq!(score.total(), 0);
    }

    #[test]
    fn test_total_score_add() {
        let mut score = TotalScore::new();
        score.add(150);
        score.add(275);

        assert_eq!(score.total(), 425);
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
        assert_eq!(system.score.total(), 0);
        assert_eq!(system.max_chains.max(), 0);
        assert_eq!(system.chain_bonus, 0);
    }

    #[test]
    fn test_set_chain_bonus_from_total() {
        let mut system = CustomScoreSystem::new();

        system.set_chain_bonus_from_total(3);
        assert_eq!(system.chain_bonus, 3);

        system.set_chain_bonus_from_total(9);
        assert_eq!(system.chain_bonus, 9);

        // 上限10に丸め込まれる
        system.set_chain_bonus_from_total(12);
        assert_eq!(system.chain_bonus, 10);
    }

    #[test]
    fn test_chain_bonus_add() {
        let mut system = CustomScoreSystem::new();

        // 通常の加算
        system.add_chain_bonus(3);
        assert_eq!(system.chain_bonus, 3);

        // 累積
        system.add_chain_bonus(5);
        assert_eq!(system.chain_bonus, 8);

        // 上限テスト：10段を超えない
        system.add_chain_bonus(5);
        assert_eq!(system.chain_bonus, 10);

        // さらに加算しても10のまま
        system.add_chain_bonus(3);
        assert_eq!(system.chain_bonus, 10);
    }

    #[test]
    fn test_chain_bonus_consume() {
        let mut system = CustomScoreSystem::new();
        system.add_chain_bonus(7);

        // 一部消費
        let consumed = system.consume_chain_bonus(3);
        assert_eq!(consumed, 3);
        assert_eq!(system.chain_bonus, 4);

        // さらに消費
        let consumed = system.consume_chain_bonus(2);
        assert_eq!(consumed, 2);
        assert_eq!(system.chain_bonus, 2);

        // 残り以上を消費しようとした場合
        let consumed = system.consume_chain_bonus(5);
        assert_eq!(consumed, 2); // 実際には2しかない
        assert_eq!(system.chain_bonus, 0);

        // 0の状態で消費
        let consumed = system.consume_chain_bonus(3);
        assert_eq!(consumed, 0);
        assert_eq!(system.chain_bonus, 0);
    }

    #[test]
    fn test_custom_score_system_display() {
        let mut system = CustomScoreSystem::new();
        system.add_score(1120);
        system.max_chains.update_max(GameColor::Cyan, 2);
        system.max_chains.update_max(GameColor::Magenta, 4);
        system.max_chains.update_max(GameColor::Yellow, 5);

        let expected = "SCORE:    1120\n\nMAX-CHAIN:\n  CYAN:    2\n  MAGENTA: 4\n  YELLOW:  5";
        assert_eq!(format!("{}", system), expected);
    }
}
