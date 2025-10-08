//! CLI専用入力処理
//! 
//! Layer 1の入力処理を使用したCLI特化入力ハンドリング
//! キーボード入力、リピート処理、ホットキー管理を含む

use crate::game_input::{GameInput, InputProvider, CrosstermInputProvider};
use crate::core::game_state::CoreGameEvent;
use crate::cli::cli_game_state::CliGameState;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::io;

/// CLI版入力ハンドラー
pub struct CliInputHandler {
    /// 入力プロバイダー
    input_provider: CrosstermInputProvider,
    
    /// キーリピート管理
    key_states: HashMap<GameInput, KeyState>,
    
    /// CLI特化設定
    repeat_delay: Duration,
    repeat_interval: Duration,
    
    /// 入力統計
    total_inputs: u64,
    last_input_time: Option<Instant>,
}

/// キー状態管理
#[derive(Debug, Clone)]
struct KeyState {
    pressed_at: Instant,
    last_repeat: Instant,
    is_repeating: bool,
}

impl CliInputHandler {
    /// 新しい入力ハンドラーを作成
    pub fn new() -> Self {
        Self {
            input_provider: CrosstermInputProvider::new(),
            key_states: HashMap::new(),
            repeat_delay: Duration::from_millis(500),   // 初回リピートまでの遅延
            repeat_interval: Duration::from_millis(50), // リピート間隔
            total_inputs: 0,
            last_input_time: None,
        }
    }
    
    /// CLI特化: ノンブロッキング入力処理
    pub fn poll_input(&mut self, cli_state: &mut CliGameState) -> io::Result<Vec<CoreGameEvent>> {
        let mut all_events = Vec::new();
        
        // 新しい入力をポーリング
        if let Ok(has_input) = self.input_provider.poll_input(0) {
            if has_input {
                if let Ok(input) = self.input_provider.read_input() {
                    self.process_new_input(input, cli_state, &mut all_events)?;
                }
            }
        }
        
        // キーリピート処理
        self.process_key_repeats(cli_state, &mut all_events)?;
        
        Ok(all_events)
    }
    
    /// CLI特化: ブロッキング入力処理（デバッグ用）
    pub fn wait_for_input(&mut self, cli_state: &mut CliGameState) -> io::Result<Vec<CoreGameEvent>> {
        let input = self.input_provider.read_input()?;
        let mut events = Vec::new();
        self.process_new_input(input, cli_state, &mut events)?;
        Ok(events)
    }
    
    /// 新しい入力の処理
    fn process_new_input(
        &mut self,
        input: GameInput,
        cli_state: &mut CliGameState,
        events: &mut Vec<CoreGameEvent>,
    ) -> io::Result<()> {
        self.total_inputs += 1;
        self.last_input_time = Some(Instant::now());
        
        // 特殊キー処理
        match input {
            GameInput::Quit => {
                // CLI特化: 即座に終了
                std::process::exit(0);
            }
            GameInput::Pause => {
                // CLI特化: ポーズ切り替え（実装は省略）
                return Ok(());
            }
            _ => {}
        }
        
        // Layer 1の入力処理を使用
        let input_events = cli_state.handle_input(input);
        events.extend(input_events);
        
        // リピート対象キーの状態更新
        if self.is_repeatable_key(&input) {
            let now = Instant::now();
            self.key_states.insert(input, KeyState {
                pressed_at: now,
                last_repeat: now,
                is_repeating: false,
            });
        }
        
        Ok(())
    }
    
    /// キーリピート処理
    fn process_key_repeats(
        &mut self,
        cli_state: &mut CliGameState,
        events: &mut Vec<CoreGameEvent>,
    ) -> io::Result<()> {
        let now = Instant::now();
        let mut keys_to_repeat = Vec::new();
        
        // リピート対象キーをチェック
        for (input, state) in &mut self.key_states {
            let elapsed = now.duration_since(state.pressed_at);
            
            if !state.is_repeating && elapsed >= self.repeat_delay {
                // 初回リピート開始
                state.is_repeating = true;
                state.last_repeat = now;
                keys_to_repeat.push(*input);
            } else if state.is_repeating {
                let since_last_repeat = now.duration_since(state.last_repeat);
                if since_last_repeat >= self.repeat_interval {
                    // リピート実行
                    state.last_repeat = now;
                    keys_to_repeat.push(*input);
                }
            }
        }
        
        // リピートキーの処理実行
        for input in keys_to_repeat {
            let input_events = cli_state.handle_input(input);
            events.extend(input_events);
        }
        
        Ok(())
    }
    
    /// リピート可能なキーかチェック
    fn is_repeatable_key(&self, input: &GameInput) -> bool {
        match input {
            GameInput::MoveLeft | 
            GameInput::MoveRight | 
            GameInput::MoveDown | 
            GameInput::SoftDrop => true,
            _ => false,
        }
    }
    
    /// CLI特化: 入力統計取得
    pub fn get_input_stats(&self) -> CliInputStats {
        CliInputStats {
            total_inputs: self.total_inputs,
            active_keys: self.key_states.len(),
            last_input_elapsed: self.last_input_time
                .map(|t| t.elapsed())
                .unwrap_or(Duration::from_secs(0)),
        }
    }
    
    /// CLI特化: キー状態リセット（ポーズ時など）
    pub fn reset_key_states(&mut self) {
        self.key_states.clear();
    }
}

/// CLI版入力統計情報
#[derive(Debug, Clone)]
pub struct CliInputStats {
    pub total_inputs: u64,
    pub active_keys: usize,
    pub last_input_elapsed: Duration,
}

impl Default for CliInputHandler {
    fn default() -> Self {
        Self::new()
    }
}