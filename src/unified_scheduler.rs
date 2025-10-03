//! 統一スケジューラー: イベント駆動型ゲームエンジン対応
//! 
//! CLI版とWeb版で共通のイベント駆動アーキテクチャを提供します。
//! sleep依存を削除し、時間ベースのイベント生成に特化します。

use std::time::Duration;
use std::collections::VecDeque;

/// ゲームイベント定義
#[derive(Debug, Clone, PartialEq)]
pub enum GameEvent {
    /// ピース自動落下イベント
    AutoFall,
    /// アニメーション更新イベント
    AnimationUpdate,
    /// ピース固定イベント
    PieceLock,
    /// ライン消去イベント
    LineClear(Vec<usize>),
    /// ゲームオーバーイベント
    GameOver,
    /// スコア更新イベント
    ScoreUpdate,
    /// 描画更新イベント
    Render,
    /// アプリケーション終了イベント
    ApplicationExit,
    /// 自動落下開始イベント
    StartAutoFall,
    /// タイトル画面表示イベント
    ShowTitle,
}

/// タイマー管理構造体
#[derive(Debug)]
pub struct GameTimer {
    pub id: u32,
    pub next_fire: Duration,
    pub interval: Duration,
    pub event: GameEvent,
    pub repeating: bool,
}

/// 統一ゲームスケジューラー（CLI版・Web版共通）
pub struct UnifiedScheduler {
    timers: Vec<GameTimer>,
    next_timer_id: u32,
    current_time: Duration,
}

impl UnifiedScheduler {
    pub fn new() -> Self {
        Self {
            timers: Vec::new(),
            next_timer_id: 1,
            current_time: Duration::ZERO,
        }
    }
    
    /// タイマーを追加
    pub fn add_timer(&mut self, interval: Duration, event: GameEvent, repeating: bool) -> u32 {
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        
        let timer = GameTimer {
            id,
            next_fire: self.current_time + interval,
            interval,
            event,
            repeating,
        };
        
        self.timers.push(timer);
        id
    }
    
    /// タイマーを削除
    pub fn remove_timer(&mut self, id: u32) {
        self.timers.retain(|timer| timer.id != id);
    }
    
    /// 時間を進めて発火すべきイベントを取得
    pub fn update(&mut self, delta_time: Duration) -> Vec<GameEvent> {
        self.current_time += delta_time;
        let mut events = Vec::new();
        
        // 発火すべきタイマーを探す
        let mut timers_to_update = Vec::new();
        
        for timer in &mut self.timers {
            if self.current_time >= timer.next_fire {
                events.push(timer.event.clone());
                
                if timer.repeating {
                    timer.next_fire += timer.interval;
                } else {
                    timers_to_update.push(timer.id);
                }
            }
        }
        
        // 一回限りのタイマーを削除
        for id in timers_to_update {
            self.remove_timer(id);
        }
        
        events
    }
    
    /// 自動落下タイマーを設定
    pub fn set_auto_fall_timer(&mut self, fall_speed: Duration) -> u32 {
        // 既存の自動落下タイマーを削除
        self.timers.retain(|timer| !matches!(timer.event, GameEvent::AutoFall));
        
        // 新しいタイマーを追加
        self.add_timer(fall_speed, GameEvent::AutoFall, true)
    }
    
    /// 描画タイマーを設定（60FPS）
    pub fn set_render_timer(&mut self) -> u32 {
        self.add_timer(Duration::from_millis(16), GameEvent::Render, true)
    }
    
    /// アニメーション更新タイマーを設定
    pub fn set_animation_timer(&mut self) -> u32 {
        self.add_timer(Duration::from_millis(120), GameEvent::AnimationUpdate, true)
    }
}

/// プラットフォーム独立なタイミングプロバイダー
pub trait TimeProvider {
    fn now(&self) -> Duration;
}

/// ネイティブ環境用タイムプロバイダー
pub struct NativeTimeProvider {
    start: std::time::Instant,
}

impl NativeTimeProvider {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

impl TimeProvider for NativeTimeProvider {
    fn now(&self) -> Duration {
        self.start.elapsed()
    }
}

/// WASM環境用タイムプロバイダー
#[cfg(target_arch = "wasm32")]
pub struct WasmTimeProvider {
    start_time: f64,
}

#[cfg(target_arch = "wasm32")]
impl WasmTimeProvider {
    pub fn new() -> Self {
        #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
        let start_time = crate::js_date_now();
        
        #[cfg(not(all(target_arch = "wasm32", feature = "wasm")))]
        let start_time = 0.0;
        
        Self { start_time }
    }
}

#[cfg(target_arch = "wasm32")]
impl TimeProvider for WasmTimeProvider {
    fn now(&self) -> Duration {
        #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
        let current_time = crate::js_date_now();
        
        #[cfg(not(all(target_arch = "wasm32", feature = "wasm")))]
        let current_time = self.start_time;
        
        let elapsed_ms = current_time - self.start_time;
        Duration::from_millis(elapsed_ms as u64)
    }
}

/// デフォルトタイムプロバイダーを作成
pub fn create_time_provider() -> Box<dyn TimeProvider> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::new(NativeTimeProvider::new())
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        Box::new(WasmTimeProvider::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_scheduler() {
        let mut scheduler = UnifiedScheduler::new();
        
        // 100msタイマーを追加
        scheduler.add_timer(Duration::from_millis(100), GameEvent::AutoFall, false);
        
        // 50ms経過 - イベントなし
        let events = scheduler.update(Duration::from_millis(50));
        assert_eq!(events.len(), 0);
        
        // さらに60ms経過（合計110ms） - イベント発火
        let events = scheduler.update(Duration::from_millis(60));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], GameEvent::AutoFall);
        
        // タイマーは一回限りなので削除されている
        let events = scheduler.update(Duration::from_millis(100));
        assert_eq!(events.len(), 0);
    }
    
    #[test]
    fn test_repeating_timer() {
        let mut scheduler = UnifiedScheduler::new();
        
        // 50ms間隔の繰り返しタイマー
        scheduler.add_timer(Duration::from_millis(50), GameEvent::Render, true);
        
        // 100ms経過 - 2回発火するはず
        let events = scheduler.update(Duration::from_millis(100));
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], GameEvent::Render);
        assert_eq!(events[1], GameEvent::Render);
    }
}