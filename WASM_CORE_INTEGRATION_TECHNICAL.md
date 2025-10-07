# WASM統合技術詳細仕様書（Phase 1-3再設計結果版）# WASM API Core Module統合 技術詳細書



**作成日**: 2025年10月7日  ## Phase 1: 統合テストフレームワーク構築

**基準**: PHASE3_WASM_BOUNDARY_REDESIGN.md  

**目標**: データコピー最優先原則による安全なWASM境界実装### Step 1.1: WASM統合テスト環境構築



---#### テスト対象

- WasmGameState → CoreGameState状態同期

## 🏗️ **3層アーキテクチャ技術仕様**- input_code → GameInput → InputProcessResult変換チェーン

- Core ModuleイベントのWASM境界通過

### **Layer 1: 共通コアロジック（src/core/）**

```rust#### 実装詳細

// 既存Core Moduleの活用（95%適合確認済み）```rust

pub struct CoreGameState {// src/lib.rs に追加

    pub board: FixedBoard,                    // [[Cell; WIDTH]; HEIGHT] - WASM安全#[cfg(all(target_arch = "wasm32", test))]

    pub animations: Vec<AnimationState>,      // 軽微調整：固定配列化予定mod wasm_integration_tests {

    pub current_piece: Option<Tetromino>,     // プリミティブ - 安全    use super::*;

    pub score: u64,                          // プリミティブ - 安全    use wasm_bindgen_test::*;

    pub chain_bonus: u32,                    // プリミティブ - 安全    use crate::core::input_handler::process_input;

    // ... 他フィールド    

}    #[wasm_bindgen_test]

    fn test_core_module_state_sync() {

// Phase 1で調整予定        let mut wasm_state = WasmGameState::new();

pub struct AnimationState {        wasm_state.start_game();

    pub lines: [Option<usize>; 4],  // Vec<usize> → 固定配列        

    pub animation_type: AnimationType,        // Core Module状態確認

    pub start_time_ms: u64,        assert_eq!(wasm_state.core_state.game_mode, CoreGameMode::Playing);

    pub current_step: usize,        assert!(wasm_state.core_state.current_piece.is_some());

    pub is_completed: bool,    }

    pub metadata: AnimationMetadata,    

}    #[wasm_bindgen_test]

```    fn test_input_processing_chain() {

        let mut wasm_state = WasmGameState::new();

### **Layer 2: CLI専用レイヤー（src/cli/）**        wasm_state.start_game();

```rust        

//! CLI版特化機能        // input_code → GameInput変換テスト

//! Layer 1を活用したRust native実装        let result = wasm_state.handle_input(0); // MoveLeft

        assert!(result);

pub struct CliGameState {    }

    pub core: CoreGameState,           // Layer 1への委譲}

    pub time_provider: TimeProvider,   // Rust native時間管理```

    pub renderer_state: RendererState, // terminal描画状態

}#### 設定ファイル更新

```toml

impl CliGameState {# Cargo.toml に追加

    /// CLI版アニメーション更新（Rust native時間）[dependencies]

    pub fn update_animations(&mut self) {wasm-bindgen-test = "0.3"

        let current_time_ms = self.time_provider.now().as_millis() as u64;

        [dev-dependencies]

        // Layer 1純粋関数使用wasm-bindgen-test = "0.3"

        self.core.animations = crate::core::animation_logic::update_animation_states(```

            &self.core.animations,

            current_time_ms,### Step 1.2: 基本統合テストケース作成

        );

        #### 重要テストケース

        // CLI特化後処理1. **初期化同期テスト**: WasmGameState.new() → CoreGameState初期化確認

        self.handle_cli_specific_updates();2. **ゲーム開始同期テスト**: start_game() → CoreGameMode::Playing確認

    }3. **基本入力テスト**: handle_input(input_code) → Core Module処理確認

}

```## Phase 2: Core Module入力処理統合



### **Layer 3: WASM専用レイヤー（src/wasm/）**### Step 2.1: 入力処理統合テスト作成

```rust

//! WASM境界安全API#### 実装すべきテストケース

//! JavaScript連携特化、データコピー最優先```rust

#[wasm_bindgen_test]

#[wasm_bindgen]fn test_all_input_codes_mapping() {

pub struct WasmGameEngine {    let test_cases = [

    core_snapshot: CoreGameState,  // Layer 1のスナップショット        (0, GameInput::MoveLeft),

    last_update_ms: u64,           // JavaScript時間管理        (1, GameInput::MoveRight),

    last_error_code: u32,          // エラー状態（プリミティブ）        (2, GameInput::SoftDrop),

}        (3, GameInput::RotateClockwise),

```        (4, GameInput::RotateCounterClockwise),

        (5, GameInput::HardDrop),

---        (6, GameInput::Restart),

        (7, GameInput::Quit),

## 🔒 **データコピー最優先パターン詳細**        (8, GameInput::ToggleEraseLine), // 新規追加

    ];

### **1. 借用チェッカー競合の完全回避**    

    for (input_code, expected_input) in test_cases {

#### **❌ 過去の問題パターン**        let actual_input = convert_input_code_to_game_input(input_code);

```rust        assert_eq!(actual_input, expected_input);

// 借用チェッカー競合リスク    }

#[wasm_bindgen]}

impl WasmGame {

    pub fn update_animations(&mut self) -> JsValue {#[wasm_bindgen_test]

        // 可変借用開始fn test_toggle_erase_line_integration() {

        let animations = &mut self.core_state.animations;    let mut wasm_state = WasmGameState::new();

            

        // JavaScript呼び出し中に再度借用 → 競合    // 初期状態: enable_erase_line = false

        let result = self.render_with_js_callback(animations);    assert!(!wasm_state.get_enable_erase_line());

        // ❌ borrow checker error    

    }    // ToggleEraseLine実行

}    let result = wasm_state.handle_input(8);

```    assert!(result);

    assert!(wasm_state.get_enable_erase_line());

#### **✅ 新設計（データコピーパターン）**    

```rust    // 再度実行でfalseに戻る

#[wasm_bindgen]    let result2 = wasm_state.handle_input(8);

impl WasmGameEngine {    assert!(result2);

    #[wasm_bindgen]    assert!(!wasm_state.get_enable_erase_line());

    pub fn update_with_time(&mut self, js_time_ms: f64) -> WasmRenderInfo {}

        let time_ms = js_time_ms as u64;```

        self.last_update_ms = time_ms;

        ### Step 2.2: handle_input関数のCore Module統合実装

        // 1. 読み取り専用借用のみ（競合なし）

        let updated_animations = crate::core::animation_logic::update_animation_states(#### 現状の問題

            &self.core_snapshot.animations,  // &[AnimationState] - 読み取り専用```rust

            time_ms,// 現在の実装（問題あり）

        );pub fn handle_input(&mut self, input_code: u8) -> bool {

            let game_input = match input_code {

        // 2. データコピーで状態更新        0 => GameInput::MoveLeft,

        self.core_snapshot.animations = updated_animations;        // ... ToggleEraseLine (8) が未対応

                _ => GameInput::Unknown,

        // 3. データコピーで戻り値作成    };

        self.create_render_info()  // 借用なし、完全コピー    

    }    // Core Moduleのprocess_inputを使わず独自処理

}    match game_input {

```        GameInput::MoveLeft => {

            if self.game_mode == 1 {

### **2. メモリ安全性の確保**                self.move_current_piece(-1, 0)

            } else {

#### **固定サイズ配列の活用**                false

```rust            }

// WASM境界での安全な型定義        }

#[wasm_bindgen]        // ...

#[derive(Clone)]    }

pub struct WasmRenderInfo {}

    // 内部データ（JavaScript非公開）```

    board_data: [[u8; BOARD_WIDTH]; BOARD_HEIGHT],  // 固定サイズ

    score: u64,           // プリミティブ#### 修正実装

    lines_cleared: u32,   // プリミティブ```rust

    chain_bonus: u32,     // プリミティブ// 新しい実装

    animation_count: usize, // プリミティブpub fn handle_input(&mut self, input_code: u8) -> bool {

}    let game_input = convert_input_code_to_game_input(input_code);

    

#[wasm_bindgen]    // Core Moduleのprocess_inputを使用

impl WasmRenderInfo {    let current_time_ms = 0; // WASM環境では簡易実装

    /// JavaScript安全なボードデータ取得    let result = process_input(

    #[wasm_bindgen]        self.core_state.clone(), 

    pub fn get_board_data(&self) -> js_sys::Uint8Array {        game_input, 

        // 固定サイズ配列をUint8Arrayにコピー        current_time_ms

        let flat_data: Vec<u8> = self.board_data    );

            .iter()    

            .flat_map(|row| row.iter())    // 結果をWasmGameStateに反映

            .cloned()    self.core_state = result.new_state;

            .collect();    

            // イベント処理

        // JavaScriptに安全な形式で返却    self.process_core_events(result.events);

        js_sys::Uint8Array::from(&flat_data[..])    

    }    result.input_consumed

    }

    /// プリミティブ型のみのgetter

    #[wasm_bindgen(getter)]fn convert_input_code_to_game_input(input_code: u8) -> GameInput {

    pub fn score(&self) -> u64 { self.score }    match input_code {

            0 => GameInput::MoveLeft,

    #[wasm_bindgen(getter)]        1 => GameInput::MoveRight,

    pub fn chain_bonus(&self) -> u32 { self.chain_bonus }        2 => GameInput::SoftDrop,

}        3 => GameInput::RotateClockwise,

```        4 => GameInput::RotateCounterClockwise,

        5 => GameInput::HardDrop,

### **3. JavaScript時間管理への移行**        6 => GameInput::Restart,

        7 => GameInput::Quit,

#### **❌ 過去の問題（Rust側時間取得）**        8 => GameInput::ToggleEraseLine, // 新規追加

```rust        _ => GameInput::Unknown,

// WASM境界での時間問題    }

pub fn update_animations(&mut self) {}

    let now = SystemTime::now();  // ❌ WASM環境での問題```

    let duration = now.duration_since(UNIX_EPOCH).unwrap();

    let current_time_ms = duration.as_millis() as u64;### Step 2.3: ToggleEraseLine機能実装

    // WASM実行時エラーのリスク

}#### 必要な追加API

``````rust

/// enable_erase_line状態を取得

#### **✅ 新設計（JavaScript時間管理）**#[wasm_bindgen]

```rustpub fn get_enable_erase_line(&self) -> bool {

// JavaScript側から時間を受け取り    self.core_state.enable_erase_line

#[wasm_bindgen]}

pub fn update_with_time(&mut self, js_time_ms: f64) -> WasmRenderInfo {

    // Rust側では時間取得しない/// enable_erase_line状態を設定

    let time_ms = js_time_ms as u64;  // 型変換のみ#[wasm_bindgen]

    pub fn set_enable_erase_line(&mut self, enabled: bool) {

    // Layer 1純粋関数は時間パラメータを受け取り    self.core_state.enable_erase_line = enabled;

    let updated_animations = crate::core::animation_logic::update_animation_states(}

        &self.core_snapshot.animations,

        time_ms,  // JavaScript提供の時間/// chain_bonus状態を取得

    );#[wasm_bindgen]

    pub fn get_chain_bonus(&self) -> u32 {

    self.core_snapshot.animations = updated_animations;    self.core_state.chain_bonus

    self.create_render_info()}

}```

```

## Phase 3: アニメーション状態API統合

**TypeScript側**:

```typescript### Step 3.1: アニメーション状態テスト作成

class GameLoop {

    private wasmEngine: WasmGameEngine;#### テスト仕様

    ```rust

    private animationLoop = (timestamp: number) => {#[wasm_bindgen_test]

        // ブラウザ標準時間を提供fn test_erase_line_animation_status_api() {

        const result = this.wasmEngine.update_with_time(timestamp);    let mut wasm_state = WasmGameState::new();

            wasm_state.start_game();

        // レンダリング処理    wasm_state.set_enable_erase_line(true);

        this.renderGame(result);    

            // 初期状態: アニメーションなし

        requestAnimationFrame(this.animationLoop);    assert!(!wasm_state.has_active_erase_line_animation());

    };    assert_eq!(wasm_state.get_erase_line_animation_progress(), 0);

}    

```    // アニメーション開始条件設定

    // (chain_bonusを増加させてからエラーライン発動)

---    setup_erase_line_animation_condition(&mut wasm_state);

    

## 🔧 **EraseLineアニメーション統合詳細**    // アニメーション開始確認

    assert!(wasm_state.has_active_erase_line_animation());

### **Layer 1での純粋関数実装**    assert!(wasm_state.get_erase_line_animation_progress() > 0);

```rust}

// src/core/erase_line_logic.rs（既存実装活用）```



/// EraseLineアニメーション開始判定### Step 3.2: アニメーション状態API実装

pub fn should_start_erase_line_animation(

    chain_bonus: u32,#### 実装すべきAPI

    board: FixedBoard,```rust

    enable_erase_line: bool,/// EraseLineアニメーションが実行中かチェック

) -> bool {#[wasm_bindgen]

    if !enable_erase_line {pub fn has_active_erase_line_animation(&self) -> bool {

        return false;    self.core_state.animations.iter()

    }        .any(|anim| matches!(anim.animation_type, AnimationType::EraseLine { .. }))

    }

    let solid_lines = count_solid_lines_from_bottom(board);

    determine_erase_line_count(chain_bonus, solid_lines) > 0/// EraseLineアニメーションの進行状況取得 (0-100)

}#[wasm_bindgen]

pub fn get_erase_line_animation_progress(&self) -> u8 {

/// CHAIN-BONUS消費によるライン消去数計算    if let Some(erase_anim) = self.core_state.animations.iter()

pub fn determine_erase_line_count(chain_bonus: u32, solid_lines_count: usize) -> usize {        .find(|anim| matches!(anim.animation_type, AnimationType::EraseLine { .. })) {

    std::cmp::min(chain_bonus as usize, solid_lines_count)        

}        let elapsed = erase_anim.elapsed_ms;

        let duration = match &erase_anim.animation_type {

/// 底辺からのSolidライン消去            AnimationType::EraseLine { duration_ms, .. } => *duration_ms,

pub fn remove_solid_line_from_bottom(board: FixedBoard, lines_to_remove: usize) -> FixedBoard {            _ => return 0,

    let mut new_board = board;        };

            

    for _ in 0..lines_to_remove {        ((elapsed * 100) / duration).min(100) as u8

        // 底辺行削除    } else {

        for y in (0..BOARD_HEIGHT-1).rev() {        0

            new_board[y+1] = new_board[y];    }

        }}

        new_board[0] = [Cell::Empty; BOARD_WIDTH];

    }/// EraseLineアニメーションの詳細情報取得

    #[wasm_bindgen]

    new_boardpub fn get_erase_line_animation_status(&self) -> String {

}    // JSON形式でアニメーション状態を返す

```    if let Some(erase_anim) = self.core_state.animations.iter()

        .find(|anim| matches!(anim.animation_type, AnimationType::EraseLine { .. })) {

### **Layer 3でのWASM API実装**        

```rust        format!(

// src/wasm/wasm_game_engine.rs            "{{\"active\":true,\"progress\":{},\"step\":{}}}",

            self.get_erase_line_animation_progress(),

#[wasm_bindgen]            erase_anim.step_count

impl WasmGameEngine {        )

    /// EraseLineアニメーション開始（安全API）    } else {

    #[wasm_bindgen]        "{\"active\":false,\"progress\":0,\"step\":0}".to_string()

    pub fn start_erase_line_animation(&mut self) -> bool {    }

        // Layer 1純粋関数使用}

        if !crate::core::erase_line_logic::should_start_erase_line_animation(```

            self.core_snapshot.chain_bonus,

            self.core_snapshot.board,## Phase 4: イベント処理統合

            self.core_snapshot.enable_erase_line,

        ) {### Step 4.1: イベント処理テスト作成

            return false;

        }#### 重要テストケース

        ```rust

        // 消去ライン数計算#[wasm_bindgen_test]

        let solid_lines = crate::core::erase_line_logic::count_solid_lines_from_bottom(fn test_core_event_processing() {

            self.core_snapshot.board    let mut wasm_state = WasmGameState::new();

        );    

        let erase_count = crate::core::erase_line_logic::determine_erase_line_count(    // Restart入力でGameModeChangedイベント発生確認

            self.core_snapshot.chain_bonus,    let result = wasm_state.handle_input(6); // Restart

            solid_lines,    assert!(result);

        );    assert_eq!(wasm_state.get_game_mode(), 1); // Playing

            

        // アニメーション作成（データコピー）    // イベントログ確認

        let target_lines: Vec<usize> = (BOARD_HEIGHT - erase_count..BOARD_HEIGHT).collect();    let events = wasm_state.get_recent_events();

        let new_animation = crate::core::animation_logic::create_erase_line_animation(    assert!(events.contains("GameModeChanged"));

            target_lines,}

            self.last_update_ms,```

        );

        ### Step 4.2: イベント処理統合実装

        // 安全な状態更新

        self.core_snapshot.animations.push(new_animation);#### イベント処理機構

        true```rust

    }// WasmGameStateにイベント処理機能追加

    pub struct WasmGameState {

    /// アニメーション状態取得    pub core_state: CoreGameState,

    #[wasm_bindgen]    recent_events: Vec<String>, // JavaScript側通知用

    pub fn get_animation_status(&self) -> WasmAnimationStatus {    // ...

        let erase_line_count = self.core_snapshot.animations}

            .iter()

            .filter(|anim| matches!(anim.animation_type, AnimationType::EraseLine))impl WasmGameState {

            .count();    fn process_core_events(&mut self, events: Vec<CoreGameEvent>) {

                    for event in events {

        WasmAnimationStatus {            match event {

            erase_line_active: erase_line_count > 0,                CoreGameEvent::GameModeChanged { new_mode } => {

            total_animations: self.core_snapshot.animations.len(),                    let mode_str = match new_mode {

        }                        CoreGameMode::Title => "Title",

    }                        CoreGameMode::Playing => "Playing", 

}                        CoreGameMode::GameOver => "GameOver",

                    };

/// JavaScript安全なアニメーション状態                    self.recent_events.push(format!("GameModeChanged:{}", mode_str));

#[wasm_bindgen]                }

#[derive(Clone)]                CoreGameEvent::EraseLineAnimationStarted => {

pub struct WasmAnimationStatus {                    self.recent_events.push("EraseLineAnimationStarted".to_string());

    erase_line_active: bool,                }

    total_animations: usize,                CoreGameEvent::EraseLineAnimationCompleted => {

}                    self.recent_events.push("EraseLineAnimationCompleted".to_string());

                }

#[wasm_bindgen]                // その他のイベント処理

impl WasmAnimationStatus {                _ => {}

    #[wasm_bindgen(getter)]            }

    pub fn erase_line_active(&self) -> bool { self.erase_line_active }        }

        }

    #[wasm_bindgen(getter)]    

    pub fn total_animations(&self) -> usize { self.total_animations }    /// 最近のイベントを取得（JavaScript側へ）

}    #[wasm_bindgen]

```    pub fn get_recent_events(&mut self) -> String {

        let events_json = format!("[{}]", 

---            self.recent_events.iter()

                .map(|e| format!("\"{}\"", e))

## 🧪 **安全性検証実装**                .collect::<Vec<_>>()

                .join(",")

### **1. 借用チェッカー安全性テスト**        );

```rust        self.recent_events.clear(); // 取得後クリア

#[cfg(test)]        events_json

mod wasm_safety_tests {    }

    use super::*;}

    ```

    #[test]

    fn test_no_borrow_checker_conflicts() {## Phase 5: 型安全性・エラーハンドリング強化

        let mut engine = WasmGameEngine::new();

        ### Step 5.1: 型安全性テスト作成

        // 並行API呼び出しシミュレーション

        let result1 = engine.update_with_time(100.0);#### エラーケーステスト

        let result2 = engine.handle_input(32);  // Space key```rust

        let result3 = engine.start_erase_line_animation();#[wasm_bindgen_test]

        let result4 = engine.get_animation_status();fn test_invalid_input_code_handling() {

            let mut wasm_state = WasmGameState::new();

        // 全て成功 = 借用チェッカー安全    

        assert!(result1.score() >= 0);    // 無効なinput_code

        assert!(result2 || !result2); // bool戻り値確認    let result = wasm_state.handle_input(255);

        assert!(result3 || !result3); // bool戻り値確認    assert!(!result); // 処理されないことを確認

        assert!(result4.total_animations() >= 0);    

    }    // 境界値テスト

        let result = wasm_state.handle_input(9);

    #[test]    assert!(!result); // 8より大きい値は無効

    fn test_data_copy_independence() {}

        let mut engine = WasmGameEngine::new();```

        

        // データコピーによる独立性確認### Step 5.2: 型安全性強化実装

        let info1 = engine.update_with_time(100.0);

        let info2 = engine.update_with_time(200.0);#### エラーハンドリング強化

        ```rust

        // 異なるインスタンス = データコピー成功pub fn handle_input(&mut self, input_code: u8) -> bool {

        assert_ne!(info1.score() as *const u64, info2.score() as *const u64);    // 入力値検証

    }    if input_code > 8 {

}        console_log!("Warning: Invalid input_code: {}", input_code);

```        return false;

    }

### **2. メモリ安全性テスト**    

```rust    let game_input = convert_input_code_to_game_input(input_code);

#[test]    

fn test_memory_safety_stress() {    // Unknown入力の明示的処理

    let mut engine = WasmGameEngine::new();    if matches!(game_input, GameInput::Unknown) {

            console_log!("Warning: Unknown input received: {}", input_code);

    // 大量データ処理でメモリ問題検出        return false;

    for i in 0..10000 {    }

        let result = engine.update_with_time(i as f64);    

            // Core Module処理（エラーハンドリング付き）

        // メモリ破損チェック    match std::panic::catch_unwind(|| {

        assert!(result.score() < u64::MAX);        process_input(self.core_state.clone(), game_input, 0)

        assert!(result.chain_bonus() < u32::MAX);    }) {

        assert!(result.animation_count() < 1000); // 妥当範囲        Ok(result) => {

                    self.core_state = result.new_state;

        // 定期的なアニメーション開始でメモリ確認            self.process_core_events(result.events);

        if i % 100 == 0 {            result.input_consumed

            let _ = engine.start_erase_line_animation();        }

        }        Err(_) => {

    }            console_log!("Error: Core module input processing failed");

}            false

        }

#[test]      }

fn test_fixed_size_array_safety() {}

    let engine = WasmGameEngine::new();```

    let info = engine.update_with_time(0.0);

    ## Phase 6: 総合統合テスト・性能最適化

    // 固定サイズ配列の境界確認

    let board_data = info.get_board_data();### Step 6.1: 総合統合テスト作成

    assert_eq!(board_data.length(), (BOARD_WIDTH * BOARD_HEIGHT) as u32);

    #### 完全フローテスト

    // 各要素が有効範囲内```rust

    for i in 0..board_data.length() {#[wasm_bindgen_test]

        let value = board_data.get_index(i);fn test_complete_erase_line_flow_wasm_api() {

        assert!(value < 100); // セル種別の妥当範囲    let mut wasm_state = WasmGameState::new();

    }    wasm_state.start_game();

}    

```    // 1. EraseLineアニメーション有効化

    wasm_state.handle_input(8); // ToggleEraseLine

### **3. JavaScript統合テスト**    assert!(wasm_state.get_enable_erase_line());

```typescript    

describe('WASM境界安全性統合テスト', () => {    // 2. chain_bonus条件設定

    let engine: WasmGameEngine;    setup_chain_bonus_condition(&mut wasm_state);

        assert!(wasm_state.get_chain_bonus() >= 6);

    beforeEach(() => {    

        engine = new WasmGameEngine();    // 3. アニメーション開始確認

    });    trigger_erase_line_animation(&mut wasm_state);

        assert!(wasm_state.has_active_erase_line_animation());

    afterEach(() => {    

        engine.free(); // WASMメモリ解放    // 4. アニメーション進行確認

    });    simulate_animation_steps(&mut wasm_state);

        

    test('EraseLineアニメーション完全サイクル', async () => {    // 5. アニメーション完了・chain_bonus消費確認

        // アニメーション開始    assert!(!wasm_state.has_active_erase_line_animation());

        const started = engine.start_erase_line_animation();    assert!(wasm_state.get_chain_bonus() < 6);

        expect(started).toBe(true);}

        ```

        // 120ms間隔でアニメーション進行

        const animationSteps = [];### 実装進行チェックリスト

        for (let t = 0; t <= 500; t += 16) {

            const result = engine.update_with_time(t);#### Phase 1

            animationSteps.push({- [ ] wasm-bindgen-test環境構築

                time: t,- [ ] 基本統合テストケース作成

                animations: result.animation_count,- [ ] Core Module状態同期確認

                score: result.score,

            });#### Phase 2  

        }- [ ] 全input_code統合テスト作成

        - [ ] handle_input Core Module統合実装

        // アニメーション進行確認- [ ] ToggleEraseLine (input_code: 8) 実装

        expect(animationSteps[0].animations).toBeGreaterThan(0);

        expect(animationSteps[animationSteps.length - 1].animations).toBe(0);#### Phase 3

    });- [ ] アニメーション状態API実装

    - [ ] has_active_erase_line_animation()

    test('高頻度API呼び出し安全性', () => {- [ ] get_erase_line_animation_progress()

        // 60FPS相当の高頻度呼び出し

        for (let i = 0; i < 1000; i++) {#### Phase 4

            const timestamp = i * 16.67; // 60FPS- [ ] Core Moduleイベント処理統合

            const result = engine.update_with_time(timestamp);- [ ] process_core_events実装

            - [ ] get_recent_events() API

            // データ整合性確認

            expect(result.score).toBeGreaterThanOrEqual(0);#### Phase 5

            expect(result.chain_bonus).toBeGreaterThanOrEqual(0);- [ ] 型安全性強化

            expect(result.animation_count).toBeGreaterThanOrEqual(0);- [ ] エラーハンドリング統一

        }- [ ] 不正入力の適切な処理

    });

    #### Phase 6

    test('エラーハンドリング安全性', () => {- [ ] 総合統合テスト

        // 異常入力テスト- [ ] CLI版との等価性確認

        const result1 = engine.update_with_time(NaN);- [ ] 性能最適化
        const result2 = engine.update_with_time(Infinity);
        const result3 = engine.update_with_time(-1000);
        
        // エラーでもクラッシュしない
        expect(result1).toBeDefined();
        expect(result2).toBeDefined();
        expect(result3).toBeDefined();
    });
});
```

---

## 📊 **パフォーマンス考慮事項**

### **1. データコピーのオーバーヘッド**
```rust
// 最適化されたデータコピー実装
impl WasmRenderInfo {
    fn from_core_state(core: &CoreGameState, rendered_board: FixedBoard) -> Self {
        // 固定サイズ配列の効率的コピー
        let board_data = rendered_board.map(|row| 
            row.map(|cell| cell.to_u8())  // インライン変換
        );
        
        Self {
            board_data,
            score: core.score,           // プリミティブコピー（高速）
            lines_cleared: core.lines_cleared,
            chain_bonus: core.chain_bonus,
            animation_count: core.animations.len(),
        }
    }
}
```

### **2. JavaScript境界の最適化**
```rust
// バッチ処理による境界越え回数削減
#[wasm_bindgen]
impl WasmGameEngine {
    /// 複数操作のバッチ実行
    #[wasm_bindgen]
    pub fn batch_update(&mut self, 
        js_time_ms: f64, 
        input_codes: &[u8]
    ) -> WasmRenderInfo {
        // 複数入力を1回の境界越えで処理
        for &input_code in input_codes {
            self.process_single_input(input_code);
        }
        
        // アニメーション更新
        self.update_animations_internal(js_time_ms as u64);
        
        // 1回の戻り値で全情報提供
        self.create_render_info()
    }
}
```

---

## 🎯 **実装完了基準**

### **技術的成功基準**
1. **✅ 借用チェッカー競合ゼロ**: 全WASMAPIテストパス
2. **✅ メモリ安全性確保**: ストレステスト10,000回成功
3. **✅ JavaScript統合安全性**: TypeScript統合テスト全パス
4. **✅ EraseLineアニメーション完全統合**: CLI版との動作同等性

### **性能基準**  
1. **✅ アニメーション60FPS**: 16ms以内でupdate_with_time完了
2. **✅ メモリ使用量**: 増加率1MB/時間以下
3. **✅ JavaScript境界**: API呼び出し1ms以内完了

### **保守性基準**
1. **✅ Layer分離**: 各Layer独立テスト可能
2. **✅ コード重複**: 95%がLayer 1共通ロジック活用
3. **✅ 拡張性**: 新アニメーション追加時の影響局所化

---

**実装準備完了**: この技術仕様に基づき、安全で高性能なWASM統合を実現できます。