# WASM API レイヤー詳細設計

**日時:** 2025年10月6日  
**目的:** 借用チェッカー競合を回避するWASM境界安全API設計  
**基盤:** CLI版完成機能の知見とWASMインシデント分析

## 🎯 設計方針

### 核心原則
1. **ゼロ借用**: WASM境界でのすべての値はコピー渡し
2. **JavaScript時間管理**: Rust側での時間取得を完全廃止
3. **固定サイズ構造**: 動的メモリ確保を最小化
4. **純粋関数パターン**: 副作用のない関数設計
5. **段階的エラー処理**: panic防止と段階的フォールバック

## 📚 データ構造設計

### 基本データ型
```rust
//! WASM境界安全なデータ構造

/// セル状態 (Copy可能)
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WasmCell {
    Empty = 0,
    OccupiedRed = 1,
    OccupiedBlue = 2,
    OccupiedGreen = 3,
    OccupiedYellow = 4,
    OccupiedPurple = 5,
    OccupiedCyan = 6,
    OccupiedOrange = 7,
    Gray = 8,
}

/// アニメーション種別
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WasmAnimationType {
    LineBlink = 0,
    PushDown = 1,
    EraseLine = 2,
}

/// アニメーション状態スナップショット
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct WasmAnimationState {
    animation_type: WasmAnimationType,
    line_0: u32,  // lines[0], None = u32::MAX
    line_1: u32,  // lines[1], None = u32::MAX
    line_2: u32,  // lines[2], None = u32::MAX
    line_3: u32,  // lines[3], None = u32::MAX
    current_step: u32,
    start_time_ms: u64,
    elapsed_ms: u64,
    is_completed: bool,
}

#[wasm_bindgen]
impl WasmAnimationState {
    #[wasm_bindgen(getter)]
    pub fn animation_type(&self) -> WasmAnimationType {
        self.animation_type
    }
    
    #[wasm_bindgen(getter)]
    pub fn is_completed(&self) -> bool {
        self.is_completed
    }
    
    #[wasm_bindgen(getter)]
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed_ms
    }
    
    /// アニメーション対象ライン取得
    #[wasm_bindgen]
    pub fn get_lines(&self) -> Vec<u32> {
        let mut lines = Vec::new();
        if self.line_0 != u32::MAX { lines.push(self.line_0); }
        if self.line_1 != u32::MAX { lines.push(self.line_1); }
        if self.line_2 != u32::MAX { lines.push(self.line_2); }
        if self.line_3 != u32::MAX { lines.push(self.line_3); }
        lines
    }
    
    /// LineBlink可視性判定 (JavaScript側補助関数)
    #[wasm_bindgen]
    pub fn is_line_visible(&self) -> bool {
        if self.animation_type != WasmAnimationType::LineBlink {
            return true;
        }
        
        let blink_step_ms = 120; // BLINK_ANIMATION_STEP相当
        let count = (self.elapsed_ms / blink_step_ms) as usize;
        (count % 2) == 0  // 偶数=表示、奇数=非表示
    }
}

/// ゲーム状態スナップショット
#[wasm_bindgen]
pub struct WasmGameStateSnapshot {
    board: Box<[[WasmCell; BOARD_WIDTH]; BOARD_HEIGHT]>, // Boxで固定サイズ
    current_board_height: u32,
    animations: Box<[WasmAnimationState]>, // 動的だが単純構造
    score: u64,
    lines_cleared: u32,
    game_mode: u32, // GameMode enum as u32
}

#[wasm_bindgen]
impl WasmGameStateSnapshot {
    #[wasm_bindgen(getter)]
    pub fn score(&self) -> u64 {
        self.score
    }
    
    #[wasm_bindgen(getter)]
    pub fn lines_cleared(&self) -> u32 {
        self.lines_cleared
    }
    
    #[wasm_bindgen(getter)]
    pub fn current_board_height(&self) -> u32 {
        self.current_board_height
    }
    
    /// ボード状態の行取得 (JavaScript側イテレーション用)
    #[wasm_bindgen]
    pub fn get_board_row(&self, y: u32) -> Vec<u32> {
        if y as usize >= BOARD_HEIGHT {
            return vec![0; BOARD_WIDTH]; // Empty row
        }
        
        self.board[y as usize]
            .iter()
            .map(|cell| *cell as u32)
            .collect()
    }
    
    /// アニメーション状態一覧取得
    #[wasm_bindgen]
    pub fn get_animations(&self) -> Vec<WasmAnimationState> {
        self.animations.to_vec()
    }
    
    /// セル可視性判定 (アニメーション考慮)
    #[wasm_bindgen]
    pub fn is_cell_visible(&self, x: u32, y: u32, current_time_ms: u64) -> bool {
        // 基本可視性
        if x as usize >= BOARD_WIDTH || y as usize >= BOARD_HEIGHT {
            return false;
        }
        
        // アニメーション影響判定
        for animation in self.animations.iter() {
            let lines = animation.get_lines();
            if lines.contains(&y) {
                match animation.animation_type {
                    WasmAnimationType::LineBlink => {
                        // LineBlink: 点滅状態を計算
                        let elapsed = current_time_ms.saturating_sub(animation.start_time_ms);
                        let blink_step_ms = 120;
                        let count = (elapsed / blink_step_ms) as usize;
                        return (count % 2) == 0; // 偶数=表示
                    },
                    WasmAnimationType::PushDown => {
                        // PushDown: グレーライン判定
                        return animation.line_0 != y; // gray_line_y以外は表示
                    },
                    WasmAnimationType::EraseLine => {
                        // EraseLine: 削除中ライン判定
                        return false; // 削除中は非表示
                    }
                }
            }
        }
        
        true // デフォルト: 表示
    }
}
```

## 🔧 コアエンジン設計

### WasmGameEngine
```rust
//! WASM境界メインエンジン

#[wasm_bindgen]
pub struct WasmGameEngine {
    // 内部状態: コピー可能な単純構造のみ
    board: [[WasmCell; BOARD_WIDTH]; BOARD_HEIGHT],
    current_board_height: u32,
    animations: Vec<WasmAnimationState>,
    score: u64,
    lines_cleared: u32,
    game_mode: u32,
    
    // JavaScript管理項目
    last_update_time_ms: u64,
    next_animation_id: u32,
}

#[wasm_bindgen]
impl WasmGameEngine {
    /// コンストラクタ
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGameEngine {
        WasmGameEngine {
            board: [[WasmCell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            current_board_height: BOARD_HEIGHT as u32,
            animations: Vec::new(),
            score: 0,
            lines_cleared: 0,
            game_mode: 1, // Playing
            last_update_time_ms: 0,
            next_animation_id: 1,
        }
    }
    
    /// メインアップデート関数 (JavaScript時間管理)
    #[wasm_bindgen]
    pub fn update(&mut self, current_time_ms: u64) -> WasmUpdateResult {
        let delta_ms = current_time_ms.saturating_sub(self.last_update_time_ms);
        self.last_update_time_ms = current_time_ms;
        
        // アニメーション更新 (純粋関数パターン)
        let (updated_animations, completed_animations) = self.update_animations_safe(current_time_ms);
        self.animations = updated_animations;
        
        // 完了アニメーション処理
        let mut result = WasmUpdateResult::new();
        for completed in completed_animations {
            match completed.animation_type {
                WasmAnimationType::LineBlink => {
                    result.completed_line_blinks.push(completed);
                    // PushDown生成は次回updateで処理
                }
                WasmAnimationType::PushDown => {
                    result.completed_push_downs.push(completed);
                    // ボード更新は次回updateで処理
                }
                WasmAnimationType::EraseLine => {
                    result.completed_erase_lines.push(completed);
                    // スコア更新は次回updateで処理
                }
            }
        }
        
        result
    }
    
    /// 現在のゲーム状態取得 (スナップショット)
    #[wasm_bindgen]
    pub fn get_state(&self) -> WasmGameStateSnapshot {
        WasmGameStateSnapshot {
            board: Box::new(self.board),
            current_board_height: self.current_board_height,
            animations: self.animations.clone().into_boxed_slice(),
            score: self.score,
            lines_cleared: self.lines_cleared,
            game_mode: self.game_mode,
        }
    }
    
    /// ライン消去開始 (JavaScript側からトリガー)
    #[wasm_bindgen]
    pub fn trigger_line_clear(&mut self, lines_js: &[u32], start_time_ms: u64) -> bool {
        let lines: Vec<usize> = lines_js.iter()
            .filter_map(|&line| {
                if line < BOARD_HEIGHT as u32 {
                    Some(line as usize)
                } else {
                    None
                }
            })
            .collect();
            
        if lines.is_empty() || lines.len() > 4 {
            return false; // 無効な入力
        }
        
        // LineBlink開始
        let mut animation = WasmAnimationState {
            animation_type: WasmAnimationType::LineBlink,
            line_0: u32::MAX,
            line_1: u32::MAX,
            line_2: u32::MAX,
            line_3: u32::MAX,
            current_step: 0,
            start_time_ms,
            elapsed_ms: 0,
            is_completed: false,
        };
        
        // ライン設定
        for (i, &line) in lines.iter().enumerate() {
            match i {
                0 => animation.line_0 = line as u32,
                1 => animation.line_1 = line as u32,
                2 => animation.line_2 = line as u32,
                3 => animation.line_3 = line as u32,
                _ => break,
            }
        }
        
        self.animations.push(animation);
        true
    }
    
    /// エラー安全なアニメーション更新
    fn update_animations_safe(&self, current_time_ms: u64) -> (Vec<WasmAnimationState>, Vec<WasmAnimationState>) {
        let mut updated = Vec::new();
        let mut completed = Vec::new();
        
        for animation in &self.animations {
            let elapsed = current_time_ms.saturating_sub(animation.start_time_ms);
            let mut updated_anim = *animation;
            updated_anim.elapsed_ms = elapsed;
            
            match animation.animation_type {
                WasmAnimationType::LineBlink => {
                    let blink_step_ms = 120;
                    let max_count = 6;
                    let count = (elapsed / blink_step_ms) as u32;
                    
                    if count >= max_count {
                        updated_anim.is_completed = true;
                        completed.push(updated_anim);
                    } else {
                        updated_anim.current_step = count;
                        updated.push(updated_anim);
                    }
                }
                WasmAnimationType::PushDown => {
                    let step_duration_ms = 150;
                    
                    if elapsed >= step_duration_ms {
                        updated_anim.is_completed = true;
                        completed.push(updated_anim);
                    } else {
                        updated.push(updated_anim);
                    }
                }
                WasmAnimationType::EraseLine => {
                    let step_interval_ms = 120;
                    let steps_elapsed = (elapsed / step_interval_ms) as u32;
                    
                    // 削除対象ライン数を計算
                    let lines = animation.get_lines();
                    let total_lines = lines.len() as u32;
                    
                    if steps_elapsed >= total_lines {
                        updated_anim.is_completed = true;
                        completed.push(updated_anim);
                    } else {
                        updated_anim.current_step = steps_elapsed;
                        updated.push(updated_anim);
                    }
                }
            }
        }
        
        (updated, completed)
    }
}

/// アップデート結果
#[wasm_bindgen]
pub struct WasmUpdateResult {
    completed_line_blinks: Vec<WasmAnimationState>,
    completed_push_downs: Vec<WasmAnimationState>,
    completed_erase_lines: Vec<WasmAnimationState>,
}

#[wasm_bindgen]
impl WasmUpdateResult {
    fn new() -> Self {
        Self {
            completed_line_blinks: Vec::new(),
            completed_push_downs: Vec::new(),
            completed_erase_lines: Vec::new(),
        }
    }
    
    #[wasm_bindgen]
    pub fn get_completed_line_blinks(&self) -> Vec<WasmAnimationState> {
        self.completed_line_blinks.clone()
    }
    
    #[wasm_bindgen]
    pub fn get_completed_push_downs(&self) -> Vec<WasmAnimationState> {
        self.completed_push_downs.clone()
    }
    
    #[wasm_bindgen]
    pub fn get_completed_erase_lines(&self) -> Vec<WasmAnimationState> {
        self.completed_erase_lines.clone()
    }
}
```

## 🌐 JavaScript統合インターフェース

### TypeScript型定義
```typescript
// wasm_types.ts
export interface WasmGameEngine {
    new(): WasmGameEngine;
    update(currentTimeMs: number): WasmUpdateResult;
    get_state(): WasmGameStateSnapshot;
    trigger_line_clear(lines: number[], startTimeMs: number): boolean;
}

export interface WasmAnimationState {
    readonly animation_type: number;
    readonly is_completed: boolean;
    readonly elapsed_ms: number;
    get_lines(): number[];
    is_line_visible(): boolean;
}

export interface WasmGameStateSnapshot {
    readonly score: number;
    readonly lines_cleared: number;
    readonly current_board_height: number;
    get_board_row(y: number): number[];
    get_animations(): WasmAnimationState[];
    is_cell_visible(x: number, y: number, currentTimeMs: number): boolean;
}

export interface WasmUpdateResult {
    get_completed_line_blinks(): WasmAnimationState[];
    get_completed_push_downs(): WasmAnimationState[];
    get_completed_erase_lines(): WasmAnimationState[];
}
```

### アニメーション描画ループ
```typescript
// animation_loop.ts
import { WasmGameEngine } from './pkg/thud_and_tile';

export class GameAnimationLoop {
    private engine: WasmGameEngine;
    private lastFrameTime: number = 0;
    private animationFrameId: number | null = null;
    
    constructor() {
        this.engine = new WasmGameEngine();
    }
    
    start(): void {
        this.lastFrameTime = performance.now();
        this.loop();
    }
    
    stop(): void {
        if (this.animationFrameId !== null) {
            cancelAnimationFrame(this.animationFrameId);
            this.animationFrameId = null;
        }
    }
    
    private loop = (): void => {
        const currentTime = performance.now();
        
        try {
            // WASM エンジンアップデート
            const result = this.engine.update(currentTime);
            
            // 完了アニメーション処理
            this.handleCompletedAnimations(result);
            
            // 描画
            this.render(currentTime);
            
        } catch (error) {
            console.error('Animation loop error:', error);
            // エラー時も継続 (フェイルセーフ)
        }
        
        this.lastFrameTime = currentTime;
        this.animationFrameId = requestAnimationFrame(this.loop);
    };
    
    private handleCompletedAnimations(result: WasmUpdateResult): void {
        // LineBlink完了 → PushDown生成
        const completedLineBlinks = result.get_completed_line_blinks();
        for (const animation of completedLineBlinks) {
            this.handleLineBlinkCompletion(animation);
        }
        
        // PushDown完了 → ボード状態更新
        const completedPushDowns = result.get_completed_push_downs();
        for (const animation of completedPushDowns) {
            this.handlePushDownCompletion(animation);
        }
        
        // EraseLine完了 → スコア更新
        const completedEraseLines = result.get_completed_erase_lines();
        for (const animation of completedEraseLines) {
            this.handleEraseLineCompletion(animation);
        }
    }
    
    private render(currentTime: number): void {
        const state = this.engine.get_state();
        
        // ボード描画
        this.renderBoard(state, currentTime);
        
        // UI更新
        this.updateUI(state);
    }
    
    private renderBoard(state: WasmGameStateSnapshot, currentTime: number): void {
        const canvas = document.getElementById('game-canvas') as HTMLCanvasElement;
        const ctx = canvas.getContext('2d')!;
        
        // ボードクリア
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        
        // セルレンダリング
        for (let y = 0; y < 20; y++) {
            const row = state.get_board_row(y);
            for (let x = 0; x < 10; x++) {
                if (state.is_cell_visible(x, y, currentTime)) {
                    this.renderCell(ctx, x, y, row[x]);
                }
            }
        }
    }
    
    // ライン消去トリガー (外部から呼び出し)
    triggerLineClear(lines: number[]): boolean {
        return this.engine.trigger_line_clear(lines, performance.now());
    }
}
```

## 🛡️ エラー処理とフェイルセーフ

### Rust側エラー処理
```rust
// エラー処理専用モジュール
pub mod wasm_error_handling {
    use wasm_bindgen::prelude::*;
    
    /// WASM境界での安全なエラー処理
    pub fn safe_call<T, F>(operation: F, fallback: T) -> T
    where
        F: FnOnce() -> T + std::panic::UnwindSafe,
    {
        match std::panic::catch_unwind(operation) {
            Ok(result) => result,
            Err(_) => {
                web_sys::console::error_1(&"WASM operation panicked, using fallback".into());
                fallback
            }
        }
    }
    
    /// WASM境界での配列アクセス安全化
    pub fn safe_array_access<T: Copy + Default>(
        array: &[T],
        index: usize,
    ) -> T {
        array.get(index).copied().unwrap_or_default()
    }
}
```

### JavaScript側エラー処理
```typescript
// error_handling.ts
export class WasmErrorHandler {
    private static instance: WasmErrorHandler;
    private errorCount: number = 0;
    private maxErrors: number = 10;
    
    static getInstance(): WasmErrorHandler {
        if (!this.instance) {
            this.instance = new WasmErrorHandler();
        }
        return this.instance;
    }
    
    handleWasmError(error: Error, context: string): boolean {
        this.errorCount++;
        console.error(`WASM Error in ${context}:`, error);
        
        if (this.errorCount > this.maxErrors) {
            console.error('Too many WASM errors, entering safe mode');
            return false; // 安全モード移行
        }
        
        return true; // 継続可能
    }
    
    resetErrorCount(): void {
        this.errorCount = 0;
    }
}
```

## 📋 実装チェックリスト

### Phase 1: 基盤構造 ✅
- [x] WasmCell enum定義
- [x] WasmAnimationState構造体
- [x] WasmGameStateSnapshot構造体
- [x] 基本的なgetterメソッド

### Phase 2: コアエンジン
- [ ] WasmGameEngine実装
- [ ] update()メソッド
- [ ] アニメーション更新ロジック
- [ ] エラー処理統合

### Phase 3: JavaScript統合
- [ ] TypeScript型定義
- [ ] アニメーションループ
- [ ] レンダリングシステム
- [ ] エラーハンドリング

### Phase 4: テスト
- [ ] 単体テスト (Rust)
- [ ] 統合テスト (JS-WASM)
- [ ] 長時間実行テスト
- [ ] メモリリークテスト

---

**この設計により、過去のWASMインシデントの原因となった借用チェッカー競合を完全に回避し、安全でパフォーマンスの高いWASM統合を実現できます。**