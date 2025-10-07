# WASM API Core Module統合 技術詳細書

## Phase 1: 統合テストフレームワーク構築

### Step 1.1: WASM統合テスト環境構築

#### テスト対象
- WasmGameState → CoreGameState状態同期
- input_code → GameInput → InputProcessResult変換チェーン
- Core ModuleイベントのWASM境界通過

#### 実装詳細
```rust
// src/lib.rs に追加
#[cfg(all(target_arch = "wasm32", test))]
mod wasm_integration_tests {
    use super::*;
    use wasm_bindgen_test::*;
    use crate::core::input_handler::process_input;
    
    #[wasm_bindgen_test]
    fn test_core_module_state_sync() {
        let mut wasm_state = WasmGameState::new();
        wasm_state.start_game();
        
        // Core Module状態確認
        assert_eq!(wasm_state.core_state.game_mode, CoreGameMode::Playing);
        assert!(wasm_state.core_state.current_piece.is_some());
    }
    
    #[wasm_bindgen_test]
    fn test_input_processing_chain() {
        let mut wasm_state = WasmGameState::new();
        wasm_state.start_game();
        
        // input_code → GameInput変換テスト
        let result = wasm_state.handle_input(0); // MoveLeft
        assert!(result);
    }
}
```

#### 設定ファイル更新
```toml
# Cargo.toml に追加
[dependencies]
wasm-bindgen-test = "0.3"

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

### Step 1.2: 基本統合テストケース作成

#### 重要テストケース
1. **初期化同期テスト**: WasmGameState.new() → CoreGameState初期化確認
2. **ゲーム開始同期テスト**: start_game() → CoreGameMode::Playing確認
3. **基本入力テスト**: handle_input(input_code) → Core Module処理確認

## Phase 2: Core Module入力処理統合

### Step 2.1: 入力処理統合テスト作成

#### 実装すべきテストケース
```rust
#[wasm_bindgen_test]
fn test_all_input_codes_mapping() {
    let test_cases = [
        (0, GameInput::MoveLeft),
        (1, GameInput::MoveRight),
        (2, GameInput::SoftDrop),
        (3, GameInput::RotateClockwise),
        (4, GameInput::RotateCounterClockwise),
        (5, GameInput::HardDrop),
        (6, GameInput::Restart),
        (7, GameInput::Quit),
        (8, GameInput::ToggleEraseLine), // 新規追加
    ];
    
    for (input_code, expected_input) in test_cases {
        let actual_input = convert_input_code_to_game_input(input_code);
        assert_eq!(actual_input, expected_input);
    }
}

#[wasm_bindgen_test]
fn test_toggle_erase_line_integration() {
    let mut wasm_state = WasmGameState::new();
    
    // 初期状態: enable_erase_line = false
    assert!(!wasm_state.get_enable_erase_line());
    
    // ToggleEraseLine実行
    let result = wasm_state.handle_input(8);
    assert!(result);
    assert!(wasm_state.get_enable_erase_line());
    
    // 再度実行でfalseに戻る
    let result2 = wasm_state.handle_input(8);
    assert!(result2);
    assert!(!wasm_state.get_enable_erase_line());
}
```

### Step 2.2: handle_input関数のCore Module統合実装

#### 現状の問題
```rust
// 現在の実装（問題あり）
pub fn handle_input(&mut self, input_code: u8) -> bool {
    let game_input = match input_code {
        0 => GameInput::MoveLeft,
        // ... ToggleEraseLine (8) が未対応
        _ => GameInput::Unknown,
    };
    
    // Core Moduleのprocess_inputを使わず独自処理
    match game_input {
        GameInput::MoveLeft => {
            if self.game_mode == 1 {
                self.move_current_piece(-1, 0)
            } else {
                false
            }
        }
        // ...
    }
}
```

#### 修正実装
```rust
// 新しい実装
pub fn handle_input(&mut self, input_code: u8) -> bool {
    let game_input = convert_input_code_to_game_input(input_code);
    
    // Core Moduleのprocess_inputを使用
    let current_time_ms = 0; // WASM環境では簡易実装
    let result = process_input(
        self.core_state.clone(), 
        game_input, 
        current_time_ms
    );
    
    // 結果をWasmGameStateに反映
    self.core_state = result.new_state;
    
    // イベント処理
    self.process_core_events(result.events);
    
    result.input_consumed
}

fn convert_input_code_to_game_input(input_code: u8) -> GameInput {
    match input_code {
        0 => GameInput::MoveLeft,
        1 => GameInput::MoveRight,
        2 => GameInput::SoftDrop,
        3 => GameInput::RotateClockwise,
        4 => GameInput::RotateCounterClockwise,
        5 => GameInput::HardDrop,
        6 => GameInput::Restart,
        7 => GameInput::Quit,
        8 => GameInput::ToggleEraseLine, // 新規追加
        _ => GameInput::Unknown,
    }
}
```

### Step 2.3: ToggleEraseLine機能実装

#### 必要な追加API
```rust
/// enable_erase_line状態を取得
#[wasm_bindgen]
pub fn get_enable_erase_line(&self) -> bool {
    self.core_state.enable_erase_line
}

/// enable_erase_line状態を設定
#[wasm_bindgen]
pub fn set_enable_erase_line(&mut self, enabled: bool) {
    self.core_state.enable_erase_line = enabled;
}

/// chain_bonus状態を取得
#[wasm_bindgen]
pub fn get_chain_bonus(&self) -> u32 {
    self.core_state.chain_bonus
}
```

## Phase 3: アニメーション状態API統合

### Step 3.1: アニメーション状態テスト作成

#### テスト仕様
```rust
#[wasm_bindgen_test]
fn test_erase_line_animation_status_api() {
    let mut wasm_state = WasmGameState::new();
    wasm_state.start_game();
    wasm_state.set_enable_erase_line(true);
    
    // 初期状態: アニメーションなし
    assert!(!wasm_state.has_active_erase_line_animation());
    assert_eq!(wasm_state.get_erase_line_animation_progress(), 0);
    
    // アニメーション開始条件設定
    // (chain_bonusを増加させてからエラーライン発動)
    setup_erase_line_animation_condition(&mut wasm_state);
    
    // アニメーション開始確認
    assert!(wasm_state.has_active_erase_line_animation());
    assert!(wasm_state.get_erase_line_animation_progress() > 0);
}
```

### Step 3.2: アニメーション状態API実装

#### 実装すべきAPI
```rust
/// EraseLineアニメーションが実行中かチェック
#[wasm_bindgen]
pub fn has_active_erase_line_animation(&self) -> bool {
    self.core_state.animations.iter()
        .any(|anim| matches!(anim.animation_type, AnimationType::EraseLine { .. }))
}

/// EraseLineアニメーションの進行状況取得 (0-100)
#[wasm_bindgen]
pub fn get_erase_line_animation_progress(&self) -> u8 {
    if let Some(erase_anim) = self.core_state.animations.iter()
        .find(|anim| matches!(anim.animation_type, AnimationType::EraseLine { .. })) {
        
        let elapsed = erase_anim.elapsed_ms;
        let duration = match &erase_anim.animation_type {
            AnimationType::EraseLine { duration_ms, .. } => *duration_ms,
            _ => return 0,
        };
        
        ((elapsed * 100) / duration).min(100) as u8
    } else {
        0
    }
}

/// EraseLineアニメーションの詳細情報取得
#[wasm_bindgen]
pub fn get_erase_line_animation_status(&self) -> String {
    // JSON形式でアニメーション状態を返す
    if let Some(erase_anim) = self.core_state.animations.iter()
        .find(|anim| matches!(anim.animation_type, AnimationType::EraseLine { .. })) {
        
        format!(
            "{{\"active\":true,\"progress\":{},\"step\":{}}}",
            self.get_erase_line_animation_progress(),
            erase_anim.step_count
        )
    } else {
        "{\"active\":false,\"progress\":0,\"step\":0}".to_string()
    }
}
```

## Phase 4: イベント処理統合

### Step 4.1: イベント処理テスト作成

#### 重要テストケース
```rust
#[wasm_bindgen_test]
fn test_core_event_processing() {
    let mut wasm_state = WasmGameState::new();
    
    // Restart入力でGameModeChangedイベント発生確認
    let result = wasm_state.handle_input(6); // Restart
    assert!(result);
    assert_eq!(wasm_state.get_game_mode(), 1); // Playing
    
    // イベントログ確認
    let events = wasm_state.get_recent_events();
    assert!(events.contains("GameModeChanged"));
}
```

### Step 4.2: イベント処理統合実装

#### イベント処理機構
```rust
// WasmGameStateにイベント処理機能追加
pub struct WasmGameState {
    pub core_state: CoreGameState,
    recent_events: Vec<String>, // JavaScript側通知用
    // ...
}

impl WasmGameState {
    fn process_core_events(&mut self, events: Vec<CoreGameEvent>) {
        for event in events {
            match event {
                CoreGameEvent::GameModeChanged { new_mode } => {
                    let mode_str = match new_mode {
                        CoreGameMode::Title => "Title",
                        CoreGameMode::Playing => "Playing", 
                        CoreGameMode::GameOver => "GameOver",
                    };
                    self.recent_events.push(format!("GameModeChanged:{}", mode_str));
                }
                CoreGameEvent::EraseLineAnimationStarted => {
                    self.recent_events.push("EraseLineAnimationStarted".to_string());
                }
                CoreGameEvent::EraseLineAnimationCompleted => {
                    self.recent_events.push("EraseLineAnimationCompleted".to_string());
                }
                // その他のイベント処理
                _ => {}
            }
        }
    }
    
    /// 最近のイベントを取得（JavaScript側へ）
    #[wasm_bindgen]
    pub fn get_recent_events(&mut self) -> String {
        let events_json = format!("[{}]", 
            self.recent_events.iter()
                .map(|e| format!("\"{}\"", e))
                .collect::<Vec<_>>()
                .join(",")
        );
        self.recent_events.clear(); // 取得後クリア
        events_json
    }
}
```

## Phase 5: 型安全性・エラーハンドリング強化

### Step 5.1: 型安全性テスト作成

#### エラーケーステスト
```rust
#[wasm_bindgen_test]
fn test_invalid_input_code_handling() {
    let mut wasm_state = WasmGameState::new();
    
    // 無効なinput_code
    let result = wasm_state.handle_input(255);
    assert!(!result); // 処理されないことを確認
    
    // 境界値テスト
    let result = wasm_state.handle_input(9);
    assert!(!result); // 8より大きい値は無効
}
```

### Step 5.2: 型安全性強化実装

#### エラーハンドリング強化
```rust
pub fn handle_input(&mut self, input_code: u8) -> bool {
    // 入力値検証
    if input_code > 8 {
        console_log!("Warning: Invalid input_code: {}", input_code);
        return false;
    }
    
    let game_input = convert_input_code_to_game_input(input_code);
    
    // Unknown入力の明示的処理
    if matches!(game_input, GameInput::Unknown) {
        console_log!("Warning: Unknown input received: {}", input_code);
        return false;
    }
    
    // Core Module処理（エラーハンドリング付き）
    match std::panic::catch_unwind(|| {
        process_input(self.core_state.clone(), game_input, 0)
    }) {
        Ok(result) => {
            self.core_state = result.new_state;
            self.process_core_events(result.events);
            result.input_consumed
        }
        Err(_) => {
            console_log!("Error: Core module input processing failed");
            false
        }
    }
}
```

## Phase 6: 総合統合テスト・性能最適化

### Step 6.1: 総合統合テスト作成

#### 完全フローテスト
```rust
#[wasm_bindgen_test]
fn test_complete_erase_line_flow_wasm_api() {
    let mut wasm_state = WasmGameState::new();
    wasm_state.start_game();
    
    // 1. EraseLineアニメーション有効化
    wasm_state.handle_input(8); // ToggleEraseLine
    assert!(wasm_state.get_enable_erase_line());
    
    // 2. chain_bonus条件設定
    setup_chain_bonus_condition(&mut wasm_state);
    assert!(wasm_state.get_chain_bonus() >= 6);
    
    // 3. アニメーション開始確認
    trigger_erase_line_animation(&mut wasm_state);
    assert!(wasm_state.has_active_erase_line_animation());
    
    // 4. アニメーション進行確認
    simulate_animation_steps(&mut wasm_state);
    
    // 5. アニメーション完了・chain_bonus消費確認
    assert!(!wasm_state.has_active_erase_line_animation());
    assert!(wasm_state.get_chain_bonus() < 6);
}
```

### 実装進行チェックリスト

#### Phase 1
- [ ] wasm-bindgen-test環境構築
- [ ] 基本統合テストケース作成
- [ ] Core Module状態同期確認

#### Phase 2  
- [ ] 全input_code統合テスト作成
- [ ] handle_input Core Module統合実装
- [ ] ToggleEraseLine (input_code: 8) 実装

#### Phase 3
- [ ] アニメーション状態API実装
- [ ] has_active_erase_line_animation()
- [ ] get_erase_line_animation_progress()

#### Phase 4
- [ ] Core Moduleイベント処理統合
- [ ] process_core_events実装
- [ ] get_recent_events() API

#### Phase 5
- [ ] 型安全性強化
- [ ] エラーハンドリング統一
- [ ] 不正入力の適切な処理

#### Phase 6
- [ ] 総合統合テスト
- [ ] CLI版との等価性確認
- [ ] 性能最適化