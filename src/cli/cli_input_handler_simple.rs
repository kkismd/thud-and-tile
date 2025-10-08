//! CLI Layer: 入力処理（簡略版）
//! Phase 2A: 基本的な入力機能のみ実装

use crate::core::CoreGameEvent;
use crate::cli::CliGameState;
use std::time::Duration;
use std::io;

/// CLI特化入力ハンドラー（簡略版）
#[derive(Debug)]
pub struct CliInputHandler {
    /// 最後の入力時刻
    last_input_time: Option<Duration>,
    
    /// 入力統計
    pub input_count: u64,
}

impl CliInputHandler {
    pub fn new() -> Self {
        Self {
            last_input_time: None,
            input_count: 0,
        }
    }
    
    /// 基本的な入力ポーリング（簡略版）
    pub fn poll_input(&mut self, _cli_state: &mut CliGameState) -> io::Result<Vec<CoreGameEvent>> {
        use crossterm::event::{poll, read, Event, KeyCode};
        
        let mut events = Vec::new();
        
        // 入力が利用可能かチェック（ノンブロッキング）
        if poll(Duration::from_millis(1))? {
            match read()? {
                Event::Key(key_event) => {
                    self.input_count += 1;
                    
                    // 基本的なキー処理
                    match key_event.code {
                        KeyCode::Esc => {
                            // 終了シグナル用のイベント（簡略版 - ゲームオーバー）
                            events.push(CoreGameEvent::GameOver);
                        }
                        KeyCode::Char(_c) => {
                            // キー入力検出 - ダミーイベント
                            events.push(CoreGameEvent::ScoreUpdated { new_score: 0, added_points: 1 });
                        }
                        _ => {
                            // その他のキー - ダミーイベント
                            events.push(CoreGameEvent::ScoreUpdated { new_score: 0, added_points: 0 });
                        }
                    }
                }
                Event::Resize(_width, _height) => {
                    // リサイズイベント - ダミーイベント
                    events.push(CoreGameEvent::ScoreUpdated { new_score: 0, added_points: 0 });
                }
                _ => {}
            }
        }
        
        Ok(events)
    }
}

impl Default for CliInputHandler {
    fn default() -> Self {
        Self::new()
    }
}