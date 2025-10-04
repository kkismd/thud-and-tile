//! Phase 8のUI/表示系更新テスト
//! total_score中心の表示システムとCHAIN-BONUS連携を検証

use crate::scoring::CustomScoreSystem;

#[test]
fn test_score_display_shows_total_score() {
    let mut system = CustomScoreSystem::new();
    system.total_score = 1250;
    system.max_chains.cyan = 3;
    system.max_chains.magenta = 4;
    system.max_chains.yellow = 5;
    system.max_chains.chain_bonus = 2;
    
    let display_text = format!("{}", system);
    assert!(display_text.contains("TOTAL SCORE: 1250"));
    assert!(display_text.contains("CHAIN-BONUS: 2"));
    assert!(!display_text.lines().any(|line| line.trim().starts_with("SCORE:")), "旧スコア行は非表示");
}

#[test]
fn test_total_score_display_format() {
    let mut system = CustomScoreSystem::new();
    system.total_score = 42000;
    system.max_chains.chain_bonus = 15;
    
    let display_text = format!("{}", system);
    assert!(display_text.contains("TOTAL SCORE: 42000"));
    assert!(display_text.contains("CHAIN-BONUS: 15"));
}

#[test]
fn test_chain_bonus_zero_display() {
    let mut system = CustomScoreSystem::new();
    system.total_score = 500;
    system.max_chains.chain_bonus = 0;
    
    let display_text = format!("{}", system);
    assert!(display_text.contains("TOTAL SCORE: 500"));
    assert!(display_text.contains("CHAIN-BONUS: 0"));
}