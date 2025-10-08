//! CLI Layer: 入力処理（Phase 2B実装）
//! ゲーム用キーボード入力をリアルタイム検出・変換

use crate::core::CoreGameEvent;
use crate::cli::CliGameState;
use crate::game_input::{CrosstermInputProvider, InputProvider};
use std::time::{Instant};
use std::io;

/// CLI特化入力ハンドラー（Phase 2B拡張版）
#[derive(Debug)]
pub struct CliInputHandler {
    /// 最後の入力時刻
    last_input_time: Option<Instant>,
    
    /// 入力統計
    pub input_count: u64,
    
    /// 入力プロバイダー
    input_provider: CrosstermInputProvider,
    
    /// 連続入力防止のためのクールダウン時間（ms）
    input_cooldown_ms: u64,
}

impl CliInputHandler {
    pub fn new() -> Self {
        Self {
            last_input_time: None,
            input_count: 0,
            input_provider: CrosstermInputProvider::new(),
            input_cooldown_ms: 100, // 100ms クールダウン
        }
    }
    
    /// Phase 2B: リアルタイム入力処理とゲームコマンド変換
    pub fn poll_input(&mut self, cli_state: &mut CliGameState) -> io::Result<Vec<CoreGameEvent>> {
        let mut all_events = Vec::new();
        
        // 入力が利用可能かチェック（ノンブロッキング）
        if self.input_provider.poll_input(1)? {
            // 利用可能な入力をすべて読み取り
            let inputs = self.input_provider.read_all_pending()?;
            
            for game_input in inputs {
                // クールダウンチェック
                let current_time = Instant::now();
                if let Some(last_time) = self.last_input_time {
                    if current_time.duration_since(last_time).as_millis() < self.input_cooldown_ms as u128 {
                        continue; // クールダウン中はスキップ
                    }
                }
                
                self.input_count += 1;
                self.last_input_time = Some(current_time);
                
                // CliGameState経由でLayer 1のcore input handlerを使用
                let events = cli_state.handle_input(game_input);
                all_events.extend(events);
            }
        }
        
        Ok(all_events)
    }
    
    /// 入力統計リセット
    pub fn reset_stats(&mut self) {
        self.input_count = 0;
        self.last_input_time = None;
    }
    
    /// クールダウン時間設定
    pub fn set_cooldown_ms(&mut self, cooldown_ms: u64) {
        self.input_cooldown_ms = cooldown_ms;
    }
}

impl Default for CliInputHandler {
    fn default() -> Self {
        Self::new()
    }
}