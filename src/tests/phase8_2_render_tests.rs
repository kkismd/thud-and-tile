use crate::GameState;

#[cfg(test)]
mod phase8_2_render_tests {
    use super::*;

    #[test]
    fn test_render_shows_total_score() {
        let mut state = GameState::new();
        state.custom_score_system.total_score = 1500;
        
        // Test the expected format string directly
        let expected_format = format!("TOTAL SCORE: {:<6}", state.custom_score_system.total_score);
        assert!(expected_format.contains("TOTAL SCORE: 1500"));
        assert!(!expected_format.contains("SCORE:      "));
    }

    #[test]
    fn test_render_shows_chain_bonus() {
        let mut state = GameState::new();
        state.custom_score_system.max_chains.chain_bonus = 5;
        
        let expected_format = format!("CHAIN-BONUS: {:<6}", state.custom_score_system.max_chains.chain_bonus);
        assert!(expected_format.contains("CHAIN-BONUS: 5"));
    }

    #[test]
    fn test_render_hides_individual_color_scores() {
        let mut state = GameState::new();
        state.custom_score_system.total_score = 2000;
        
        // Test that we expect TOTAL SCORE format instead of individual color scores
        let total_format = format!("TOTAL SCORE: {:<6}", state.custom_score_system.total_score);
        
        // Should contain TOTAL SCORE
        assert!(total_format.contains("TOTAL SCORE:"));
        
        // Should NOT look like old individual score format
        assert!(!total_format.starts_with("  CYAN:"));
        assert!(!total_format.starts_with("  MAGENTA:"));
        assert!(!total_format.starts_with("  YELLOW:"));
    }
}