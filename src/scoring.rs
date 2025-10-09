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
        writeln!(f, "TOTAL SCORE: {}", self.total_score)?;
        writeln!(f)?;
        writeln!(f, "MAX-CHAIN: {}", self.max_chains.max())?;
        writeln!(f, "  CYAN:    {}", self.max_chains.cyan)?;
        writeln!(f, "  MAGENTA: {}", self.max_chains.magenta)?;
        writeln!(f, "  YELLOW:  {}", self.max_chains.yellow)?;
        writeln!(f)?;
        write!(f, "CHAIN-BONUS: {}", self.max_chains.chain_bonus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Phase 5A-1: REFACTOR段階 - total_scoreベーステストの可読性向上
    #[test]
    fn test_total_score_initialization() {
        let system = CustomScoreSystem::new();
        assert_eq!(
            system.total_score, 0,
            "新規CustomScoreSystemのtotal_scoreは0で初期化されるべき"
        );
    }

    #[test]
    fn test_total_score_addition() {
        let mut system = CustomScoreSystem::new();

        // 段階的にスコアを加算
        system.add_total_score(100);
        assert_eq!(system.total_score, 100, "最初の加算後");

        system.add_total_score(200);
        assert_eq!(system.total_score, 300, "2回目の加算後");

        system.add_total_score(300);
        assert_eq!(system.total_score, 600, "3回目の加算後、累積スコア確認");
    }

    #[test]
    fn test_total_score_overflow_protection() {
        let mut system = CustomScoreSystem::new();
        system.total_score = u32::MAX - 100;

        // オーバーフローを引き起こすような大きな値を加算
        system.add_total_score(200);

        // saturating_addによりu32::MAXでカンプ
        assert_eq!(
            system.total_score,
            u32::MAX,
            "オーバーフロー保護でu32::MAXに制限される"
        );
    }

    // Phase 5A-1: RED段階 - ColorScoresが新システムとの整合性チェック
    #[test]
    fn test_score_system_migration_consistency() {
        // このテストは新旧システムの一貫性をチェックする
        let mut system = CustomScoreSystem::new();

        // 旧システム（ColorScores）でスコア追加
        system.scores.add(GameColor::Cyan, 100);
        system.scores.add(GameColor::Magenta, 200);

        // 新システム（total_score）でも同じスコアが反映されているべき
        let old_total = system.scores.total();
        let new_total = system.total_score;

        // この段階では一致しないため、RED状態であることを確認
        // 新システムに移行すれば一致するはず
        assert_eq!(old_total, 300);
        assert_eq!(new_total, 0); // 新システムはまだ未使用のため0

        // 将来的には total_score が主要スコアシステムになる
        // この不一致が修正対象であることを示すテスト
        assert_ne!(
            old_total, new_total,
            "新旧システムの不一致を確認（修正対象）"
        );
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
        // 新統合スコアシステムではtotal_scoreフィールドを使用
        system.add_total_score(1120);
        system.max_chains.update_max(GameColor::Cyan, 2);
        system.max_chains.update_max(GameColor::Magenta, 4);
        system.max_chains.update_max(GameColor::Yellow, 5);

        let expected = "TOTAL SCORE: 1120\n\nMAX-CHAIN: 5\n  CYAN:    2\n  MAGENTA: 4\n  YELLOW:  5\n\nCHAIN-BONUS: 0";
        assert_eq!(format!("{}", system), expected);
    }

    // Phase 5A-2: 修正済み - CustomScoreSystemの新統合スコアシステム確認テスト
    #[test]
    fn test_custom_score_system_consistency_issue() {
        // 新統合スコアシステムではtotal_scoreフィールドを使用
        let mut system = CustomScoreSystem::new();

        // 新しいadd_total_score()を使用
        system.add_total_score(100);

        // 新システムでは整合性が保たれる
        assert_eq!(system.total_score, 100);

        // Display機能も新統合スコアシステムに対応
        let display_text = format!("{}", system);
        assert!(
            display_text.contains("TOTAL SCORE: 100"),
            "Display should show new integrated scores system"
        );

        // 新統合スコアシステムの整合性確認（修正完了）
        assert_eq!(system.total_score, 100, "新統合スコアシステムの整合性確認");
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

/// Phase 3-1: ライン消去の統合スコア計算関数
pub fn calculate_line_clear_total_score(
    board: &Vec<Vec<crate::cell::Cell>>,
    cleared_line_y: usize,
    max_chains: &ColorMaxChains,
) -> u32 {
    use crate::cell::Cell;
    use crate::game_color::GameColor;

    if cleared_line_y >= board.len() {
        return 0;
    }

    // 各色のブロック数をカウント
    let mut color_counts = [0u32; 3]; // [cyan, magenta, yellow]

    for cell in &board[cleared_line_y] {
        match cell {
            Cell::Occupied(color) => match color {
                GameColor::Cyan => color_counts[0] += 1,
                GameColor::Magenta => color_counts[1] += 1,
                GameColor::Yellow => color_counts[2] += 1,
                _ => {} // 他の色は無視
            },
            Cell::Connected { color, count } => match color {
                GameColor::Cyan => color_counts[0] += *count as u32,
                GameColor::Magenta => color_counts[1] += *count as u32,
                GameColor::Yellow => color_counts[2] += *count as u32,
                _ => {} // 他の色は無視
            },
            _ => {} // Empty, Solidなどは無視
        }
    }

    // 各色のスコアを計算: ブロック数 × MAX-CHAIN × 10
    let cyan_score = color_counts[0] * max_chains.cyan * 10;
    let magenta_score = color_counts[1] * max_chains.magenta * 10;
    let yellow_score = color_counts[2] * max_chains.yellow * 10;

    // オーバーフロー防止で合計
    cyan_score
        .saturating_add(magenta_score)
        .saturating_add(yellow_score)
}

/// Phase 4A-1: MAX-CHAIN増加量計算関数
pub fn calculate_chain_increases(old_chains: &ColorMaxChains, new_chains: &ColorMaxChains) -> u32 {
    let mut total_increases = 0u32;

    // 各色のMAX-CHAIN増加量を計算（減少時は0）
    if new_chains.cyan > old_chains.cyan {
        total_increases = total_increases.saturating_add(new_chains.cyan - old_chains.cyan);
    }

    if new_chains.magenta > old_chains.magenta {
        total_increases = total_increases.saturating_add(new_chains.magenta - old_chains.magenta);
    }

    if new_chains.yellow > old_chains.yellow {
        total_increases = total_increases.saturating_add(new_chains.yellow - old_chains.yellow);
    }

    total_increases
}

/// Phase 4A-2: CHAIN-BONUS自動更新関数
pub fn update_chain_bonus_from_increases(
    target_chains: &mut ColorMaxChains,
    new_chains: &ColorMaxChains,
) {
    let increases = calculate_chain_increases(target_chains, new_chains);
    target_chains.chain_bonus = target_chains.chain_bonus.saturating_add(increases);

    // Update the MAX-CHAIN values to the new values
    target_chains.cyan = new_chains.cyan;
    target_chains.magenta = new_chains.magenta;
    target_chains.yellow = new_chains.yellow;
}

/// Phase 4B-1: lock_piece()での新スコア計算使用関数
/// この関数は既存のlock_piece()とは分離された新しいスコア計算パスです
pub fn lock_piece_with_total_score(
    custom_score: &mut CustomScoreSystem,
    board: &Vec<Vec<crate::cell::Cell>>,
    cleared_line_indices: &[usize],
) {
    // 消去されたラインのスコアを新しい計算で算出
    for &line_y in cleared_line_indices {
        let line_score = calculate_line_clear_total_score(board, line_y, &custom_score.max_chains);
        custom_score.total_score = custom_score.total_score.saturating_add(line_score);
    }
}

/// Phase 4A-2: ピース着地時のCHAIN-BONUS更新関数
pub fn update_chain_bonus_on_piece_lock(
    custom_score: &mut CustomScoreSystem,
    new_chains: &ColorMaxChains,
) {
    let old_chains = custom_score.max_chains.clone();
    let increases = calculate_chain_increases(&old_chains, new_chains);

    // MAX-CHAINを更新
    custom_score.max_chains.cyan = new_chains.cyan;
    custom_score.max_chains.magenta = new_chains.magenta;
    custom_score.max_chains.yellow = new_chains.yellow;

    // CHAIN-BONUSに増加分を加算
    custom_score.max_chains.chain_bonus = custom_score
        .max_chains
        .chain_bonus
        .saturating_add(increases);
}

/// Phase 4B-1: lock_piece()での新スコア計算統合関数
pub fn lock_piece_with_integrated_scoring(
    custom_score: &mut CustomScoreSystem,
    board: &Vec<Vec<crate::cell::Cell>>,
    cleared_line_indices: &[usize],
) {
    // 既存のlock_piece_with_total_score関数を呼び出して統合
    lock_piece_with_total_score(custom_score, board, cleared_line_indices);
}

#[cfg(test)]
mod phase4_tests {
    use super::*;

    #[test]
    fn test_phase4a1_detect_max_chain_increases() {
        let old_chains = ColorMaxChains {
            cyan: 2,
            magenta: 3,
            yellow: 4,
            chain_bonus: 0,
        };
        let new_chains = ColorMaxChains {
            cyan: 4,
            magenta: 3,
            yellow: 6,
            chain_bonus: 0,
        };

        let increases = calculate_chain_increases(&old_chains, &new_chains);
        assert_eq!(increases, 4); // (4-2) + (6-4) = 4
    }

    // Phase 4A-2: ピース着地時のCHAIN-BONUS更新テスト（RED段階）
    #[test]
    fn test_phase4a2_chain_bonus_update_on_piece_lock() {
        let mut custom_score = CustomScoreSystem::new();
        custom_score.max_chains.cyan = 2;
        custom_score.max_chains.chain_bonus = 1;

        // 新しいMAX-CHAINの状態（Cyan: 2→4に増加）
        let new_chains = ColorMaxChains {
            cyan: 4,
            magenta: 0,
            yellow: 0,
            chain_bonus: 0,
        };

        // この関数は未実装のため失敗するはず
        update_chain_bonus_on_piece_lock(&mut custom_score, &new_chains);

        assert_eq!(custom_score.max_chains.cyan, 4);
        assert_eq!(custom_score.max_chains.chain_bonus, 3); // 1 + (4-2)
    }

    // Phase 4A-2: エッジケーステスト（REFACTOR段階）
    #[test]
    fn test_phase4a2_chain_bonus_no_decrease() {
        let mut custom_score = CustomScoreSystem::new();
        custom_score.max_chains.cyan = 5;
        custom_score.max_chains.chain_bonus = 2;

        // MAX-CHAINが減少する場合（5→3）
        let new_chains = ColorMaxChains {
            cyan: 3,
            magenta: 0,
            yellow: 0,
            chain_bonus: 0,
        };

        update_chain_bonus_on_piece_lock(&mut custom_score, &new_chains);

        // 減少時はCHAIN-BONUSは増加しない
        assert_eq!(custom_score.max_chains.cyan, 3);
        assert_eq!(custom_score.max_chains.chain_bonus, 2); // 変化なし
    }

    // Phase 4B-1: lock_piece()での新スコア計算使用テスト（RED段階）
    #[test]
    fn test_phase4b1_lock_piece_uses_total_score() {
        use crate::cell::Cell;
        use crate::game_color::GameColor;

        let mut custom_score = CustomScoreSystem::new();
        let initial_total = custom_score.total_score;

        // MAX-CHAINを設定してスコア計算が可能にする
        custom_score.max_chains.cyan = 2;
        custom_score.max_chains.magenta = 3;
        custom_score.max_chains.yellow = 1;

        // テスト用ボードを作成（1ライン分のブロック）
        let mut board = vec![vec![Cell::Empty; 10]; 20];
        for x in 0..10 {
            board[19][x] = Cell::Occupied(GameColor::Cyan);
        }

        // この関数は未実装のため失敗するはず
        lock_piece_with_integrated_scoring(&mut custom_score, &board, &[19]);

        assert!(custom_score.total_score > initial_total);
        // 既存のcolor_scoresは更新されないことを確認（並行期間中）
        assert_eq!(custom_score.scores.total(), 0);
    }
}
