// ThumperBlocks用のデバイス独立入力システム
// WASM移植のためにcrossterm::eventからの独立を実現

use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameInput {
    // 移動操作
    MoveLeft,
    MoveRight,
    SoftDrop,        // スペースキー: ゆっくり落下
    HardDrop,        // Shift+Down: 即座に着地
    
    // 回転操作
    RotateClockwise,        // Down: 時計回り
    RotateCounterClockwise, // Up: 反時計回り
    
    // ゲーム制御
    Quit,            // 'q': ゲーム終了
    Restart,         // Enter: ゲーム開始/再開
    Pause,           // 'p': 一時停止（将来用）
    
    // その他
    Unknown,         // 未対応キー
}

/// プラットフォーム独立な入力プロバイダー
pub trait InputProvider {
    /// 入力をポーリングして、利用可能な入力があるかチェック
    fn poll_input(&mut self, timeout_ms: u64) -> io::Result<bool>;
    
    /// 次の入力を読み取り
    fn read_input(&mut self) -> io::Result<Option<GameInput>>;
    
    /// 複数の入力をまとめて読み取り（入力バッファリング用）
    fn read_all_pending(&mut self) -> io::Result<Vec<GameInput>>;
}

/// crossterm用の入力プロバイダー実装（ターミナル環境用）
#[cfg(not(target_arch = "wasm32"))]
pub struct CrosstermInputProvider;

#[cfg(not(target_arch = "wasm32"))]
impl CrosstermInputProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl InputProvider for CrosstermInputProvider {
    fn poll_input(&mut self, timeout_ms: u64) -> io::Result<bool> {
        use crossterm::event;
        use std::time::Duration;
        
        event::poll(Duration::from_millis(timeout_ms))
    }
    
    fn read_input(&mut self) -> io::Result<Option<GameInput>> {
        use crossterm::event::{self, Event, KeyCode, KeyModifiers};
        
        match event::read()? {
            Event::Key(key_event) => {
                // KeyEventKindがPressでない場合は無視（リリースイベント等）
                if key_event.kind != crossterm::event::KeyEventKind::Press {
                    return Ok(Some(GameInput::Unknown));
                }
                
                let input = match key_event.code {
                    KeyCode::Left => GameInput::MoveLeft,
                    KeyCode::Right => GameInput::MoveRight,
                    KeyCode::Down => {
                        if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                            GameInput::HardDrop
                        } else {
                            GameInput::RotateClockwise
                        }
                    }
                    KeyCode::Up => GameInput::RotateCounterClockwise,
                    KeyCode::Char(' ') => GameInput::SoftDrop,
                    KeyCode::Char('q') | KeyCode::Char('Q') => GameInput::Quit,
                    KeyCode::Enter => GameInput::Restart,
                    KeyCode::Char('p') | KeyCode::Char('P') => GameInput::Pause,
                    _ => GameInput::Unknown,
                };
                
                Ok(Some(input))
            }
            _ => Ok(Some(GameInput::Unknown)), // マウスイベント等は無視
        }
    }
    
    fn read_all_pending(&mut self) -> io::Result<Vec<GameInput>> {
        use std::time::Duration;
        
        let mut inputs = Vec::new();
        
        // 待機中の全入力を読み取り
        while self.poll_input(0)? {
            if let Some(input) = self.read_input()? {
                if input != GameInput::Unknown {
                    inputs.push(input);
                }
            }
        }
        
        Ok(inputs)
    }
}

/// Web/WASM用の入力プロバイダー（将来実装用）
#[cfg(target_arch = "wasm32")]
pub struct WebInputProvider {
    // Web環境での入力状態管理
    pending_inputs: std::collections::VecDeque<GameInput>,
}

#[cfg(target_arch = "wasm32")]
impl WebInputProvider {
    pub fn new() -> Self {
        Self {
            pending_inputs: std::collections::VecDeque::new(),
        }
    }
    
    /// JavaScript側からの入力をキューに追加
    pub fn push_input(&mut self, input: GameInput) {
        self.pending_inputs.push_back(input);
    }
}

#[cfg(target_arch = "wasm32")]
impl InputProvider for WebInputProvider {
    fn poll_input(&mut self, _timeout_ms: u64) -> io::Result<bool> {
        Ok(!self.pending_inputs.is_empty())
    }
    
    fn read_input(&mut self) -> io::Result<Option<GameInput>> {
        Ok(self.pending_inputs.pop_front())
    }
    
    fn read_all_pending(&mut self) -> io::Result<Vec<GameInput>> {
        let inputs: Vec<GameInput> = self.pending_inputs.drain(..).collect();
        Ok(inputs)
    }
}

/// 入力のヘルパー関数
impl GameInput {
    /// 移動系の入力かどうか
    pub fn is_movement(&self) -> bool {
        matches!(self, 
            GameInput::MoveLeft | 
            GameInput::MoveRight | 
            GameInput::SoftDrop | 
            GameInput::HardDrop
        )
    }
    
    /// 回転系の入力かどうか
    pub fn is_rotation(&self) -> bool {
        matches!(self, 
            GameInput::RotateClockwise | 
            GameInput::RotateCounterClockwise
        )
    }
    
    /// ゲーム制御系の入力かどうか
    pub fn is_control(&self) -> bool {
        matches!(self, 
            GameInput::Quit | 
            GameInput::Restart | 
            GameInput::Pause
        )
    }
    
    /// 文字列表現を取得（デバッグ用）
    pub fn description(&self) -> &'static str {
        match self {
            GameInput::MoveLeft => "Move Left",
            GameInput::MoveRight => "Move Right", 
            GameInput::SoftDrop => "Soft Drop",
            GameInput::HardDrop => "Hard Drop",
            GameInput::RotateClockwise => "Rotate Clockwise",
            GameInput::RotateCounterClockwise => "Rotate Counter-Clockwise",
            GameInput::Quit => "Quit Game",
            GameInput::Restart => "Restart/Start Game",
            GameInput::Pause => "Pause Game",
            GameInput::Unknown => "Unknown Input",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_input_classification() {
        assert!(GameInput::MoveLeft.is_movement());
        assert!(GameInput::SoftDrop.is_movement());
        assert!(!GameInput::RotateClockwise.is_movement());
        
        assert!(GameInput::RotateClockwise.is_rotation());
        assert!(GameInput::RotateCounterClockwise.is_rotation());
        assert!(!GameInput::MoveLeft.is_rotation());
        
        assert!(GameInput::Quit.is_control());
        assert!(GameInput::Restart.is_control());
        assert!(!GameInput::MoveLeft.is_control());
    }

    #[test]
    fn test_game_input_descriptions() {
        assert_eq!(GameInput::MoveLeft.description(), "Move Left");
        assert_eq!(GameInput::Quit.description(), "Quit Game");
        assert_eq!(GameInput::Unknown.description(), "Unknown Input");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_crossterm_input_provider_creation() {
        let _provider = CrosstermInputProvider::new();
        // 基本的な作成テスト
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_web_input_provider_creation() {
        let mut provider = WebInputProvider::new();
        
        // 初期状態はempty
        assert!(!provider.poll_input(0).unwrap());
        assert_eq!(provider.read_input().unwrap(), None);
        
        // 入力を追加
        provider.push_input(GameInput::MoveLeft);
        assert!(provider.poll_input(0).unwrap());
        assert_eq!(provider.read_input().unwrap(), Some(GameInput::MoveLeft));
    }
}