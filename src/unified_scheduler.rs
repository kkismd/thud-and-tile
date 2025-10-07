//! 統一スケジューラー: CLI版とWeb版で共通のタイミング制御//! 統一スケジューラー: イベント駆動型ゲームエンジン対応

//! //! 

//! sleep依存を削除し、イベント駆動型のタイミング制御を提供します。//! CLI版とWeb版で共通のイベント駆動アーキテクチャを提供します。

//! sleep依存を削除し、時間ベースのイベント生成に特化します。

use std::time::Duration;

use std::collections::HashMap;use std::time::Duration;

use std::collections::VecDeque;

/// ゲームイベント

#[derive(Debug, Clone, PartialEq)]/// ゲームイベント定義

pub enum GameEvent {#[derive(Debug, Clone, PartialEq)]

    /// 自動落下イベントpub enum GameEvent {

    AutoFall,    /// ピース自動落下イベント

    /// アニメーション更新    AutoFall,

    AnimationUpdate,    /// アニメーション更新イベント

    /// 描画イベント    AnimationUpdate,

    Render,    /// ピース固定イベント

    /// ピースロック    PieceLock,

    PieceLock,    /// ライン消去イベント

    /// ゲーム開始（自動落下開始）    LineClear(Vec<usize>),

    StartAutoFall,    /// ゲームオーバーイベント

    /// ゲームオーバー    GameOver,

    GameOver,    /// スコア更新イベント

    /// タイトル画面表示    ScoreUpdate,

    ShowTitle,    /// 描画更新イベント

    /// アプリケーション終了    Render,

    ApplicationExit,    /// アプリケーション終了イベント

    /// スコア更新    ApplicationExit,

    ScoreUpdate,    /// 自動落下開始イベント

}    StartAutoFall,

    /// タイトル画面表示イベント

/// 時間プロバイダー（CLI/Web/テスト用を統一）    ShowTitle,

pub trait TimeProvider {}

    fn now(&self) -> Duration;

}/// タイマー管理構造体

#[derive(Debug)]

/// ネイティブ時間プロバイダー（CLI用）pub struct GameTimer {

pub struct NativeTimeProvider {    pub id: u32,

    start_time: std::time::Instant,    pub next_fire: Duration,

}    pub interval: Duration,

    pub event: GameEvent,

impl NativeTimeProvider {    pub repeating: bool,

    pub fn new() -> Self {}

        Self {

            start_time: std::time::Instant::now(),/// 統一ゲームスケジューラー（CLI版・Web版共通）

        }pub struct UnifiedScheduler {

    }    timers: Vec<GameTimer>,

}    next_timer_id: u32,

    current_time: Duration,

impl TimeProvider for NativeTimeProvider {}

    fn now(&self) -> Duration {

        self.start_time.elapsed()impl UnifiedScheduler {

    }    pub fn new() -> Self {

}        Self {

            timers: Vec::new(),

/// Web用時間プロバイダー（将来実装予定）            next_timer_id: 1,

pub struct WebTimeProvider {            current_time: Duration::ZERO,

    // Web版では performance.now() を使用予定        }

}    }

    

impl Default for WebTimeProvider {    /// タイマーを追加

    fn default() -> Self {    pub fn add_timer(&mut self, interval: Duration, event: GameEvent, repeating: bool) -> u32 {

        Self {}        let id = self.next_timer_id;

    }        self.next_timer_id += 1;

}        

        let timer = GameTimer {

impl TimeProvider for WebTimeProvider {            id,

    fn now(&self) -> Duration {            next_fire: self.current_time + interval,

        // Web版では将来実装            interval,

        Duration::ZERO            event,

    }            repeating,

}        };

        

/// タイマー情報        self.timers.push(timer);

#[derive(Debug, Clone)]        id

struct Timer {    }

    id: u32,    

    interval: Duration,    /// タイマーを削除

    last_fired: Duration,    pub fn remove_timer(&mut self, id: u32) {

    event: GameEvent,        self.timers.retain(|timer| timer.id != id);

    active: bool,    }

}    

    /// 時間を進めて発火すべきイベントを取得

/// 統一スケジューラー（イベント駆動）    pub fn update(&mut self, delta_time: Duration) -> Vec<GameEvent> {

pub struct UnifiedScheduler {        self.current_time += delta_time;

    timers: HashMap<u32, Timer>,        let mut events = Vec::new();

    next_timer_id: u32,        

    pending_events: Vec<GameEvent>,        // 発火すべきタイマーを探す

}        let mut timers_to_update = Vec::new();

        

impl UnifiedScheduler {        for timer in &mut self.timers {

    pub fn new() -> Self {            if self.current_time >= timer.next_fire {

        Self {                events.push(timer.event.clone());

            timers: HashMap::new(),                

            next_timer_id: 1,                if timer.repeating {

            pending_events: Vec::new(),                    timer.next_fire += timer.interval;

        }                } else {

    }                    timers_to_update.push(timer.id);

                    }

    /// 自動落下タイマーを設定            }

    pub fn set_auto_fall_timer(&mut self, interval: Duration) -> u32 {        }

        let id = self.next_timer_id;        

        self.next_timer_id += 1;        // 一回限りのタイマーを削除

                for id in timers_to_update {

        let timer = Timer {            self.remove_timer(id);

            id,        }

            interval,        

            last_fired: Duration::ZERO,        events

            event: GameEvent::AutoFall,    }

            active: true,    

        };    /// 自動落下タイマーを設定

            pub fn set_auto_fall_timer(&mut self, fall_speed: Duration) -> u32 {

        self.timers.insert(id, timer);        // 既存の自動落下タイマーを削除

        id        self.timers.retain(|timer| !matches!(timer.event, GameEvent::AutoFall));

    }        

            // 新しいタイマーを追加

    /// 描画タイマーを設定        self.add_timer(fall_speed, GameEvent::AutoFall, true)

    pub fn set_render_timer(&mut self) -> u32 {    }

        let id = self.next_timer_id;    

        self.next_timer_id += 1;    /// 描画タイマーを設定（60FPS）

            pub fn set_render_timer(&mut self) -> u32 {

        let timer = Timer {        self.add_timer(Duration::from_millis(16), GameEvent::Render, true)

            id,    }

            interval: Duration::from_millis(16), // 60 FPS    

            last_fired: Duration::ZERO,    /// アニメーション更新タイマーを設定

            event: GameEvent::Render,    pub fn set_animation_timer(&mut self) -> u32 {

            active: true,        self.add_timer(Duration::from_millis(120), GameEvent::AnimationUpdate, true)

        };    }

        }

        self.timers.insert(id, timer);

        id/// プラットフォーム独立なタイミングプロバイダー

    }pub trait TimeProvider {

        fn now(&self) -> Duration;

    /// アニメーションタイマーを設定}

    pub fn set_animation_timer(&mut self) -> u32 {

        let id = self.next_timer_id;/// ネイティブ環境用タイムプロバイダー

        self.next_timer_id += 1;pub struct NativeTimeProvider {

            start: std::time::Instant,

        let timer = Timer {}

            id,

            interval: Duration::from_millis(50), // 20 FPS for animationsimpl NativeTimeProvider {

            last_fired: Duration::ZERO,    pub fn new() -> Self {

            event: GameEvent::AnimationUpdate,        Self {

            active: true,            start: std::time::Instant::now(),

        };        }

            }

        self.timers.insert(id, timer);}

        id

    }impl TimeProvider for NativeTimeProvider {

        fn now(&self) -> Duration {

    /// タイマーを削除        self.start.elapsed()

    pub fn remove_timer(&mut self, id: u32) {    }

        self.timers.remove(&id);}

    }

    /// WASM環境用タイムプロバイダー

    /// タイマーを停止/再開#[cfg(target_arch = "wasm32")]

    pub fn set_timer_active(&mut self, id: u32, active: bool) {pub struct WasmTimeProvider {

        if let Some(timer) = self.timers.get_mut(&id) {    start_time: f64,

            timer.active = active;}

        }

    }#[cfg(target_arch = "wasm32")]

    impl WasmTimeProvider {

    /// スケジューラーを更新し、発火するイベントを取得    pub fn new() -> Self {

    pub fn update(&mut self, current_time: Duration) -> Vec<GameEvent> {        #[cfg(all(target_arch = "wasm32", feature = "wasm"))]

        let mut events = Vec::new();        let start_time = crate::js_date_now();

                

        // ペンディングイベントを追加        #[cfg(not(all(target_arch = "wasm32", feature = "wasm")))]

        events.append(&mut self.pending_events);        let start_time = 0.0;

                

        // タイマーチェック        Self { start_time }

        for timer in self.timers.values_mut() {    }

            if !timer.active {}

                continue;

            }#[cfg(target_arch = "wasm32")]

            impl TimeProvider for WasmTimeProvider {

            let elapsed_since_last = current_time.saturating_sub(timer.last_fired);    fn now(&self) -> Duration {

            if elapsed_since_last >= timer.interval {        #[cfg(all(target_arch = "wasm32", feature = "wasm"))]

                events.push(timer.event.clone());        let current_time = crate::js_date_now();

                timer.last_fired = current_time;        

            }        #[cfg(not(all(target_arch = "wasm32", feature = "wasm")))]

        }        let current_time = self.start_time;

                

        events        let elapsed_ms = current_time - self.start_time;

    }        Duration::from_millis(elapsed_ms as u64)

        }

    /// 即座にイベントを追加}

    pub fn add_immediate_event(&mut self, event: GameEvent) {

        self.pending_events.push(event);/// デフォルトタイムプロバイダーを作成

    }pub fn create_time_provider() -> Box<dyn TimeProvider> {

}    #[cfg(not(target_arch = "wasm32"))]

    {

/// 時間プロバイダーファクトリー関数        Box::new(NativeTimeProvider::new())

pub fn create_time_provider() -> Box<dyn TimeProvider> {    }

    Box::new(NativeTimeProvider::new())    

}    #[cfg(target_arch = "wasm32")]

    {

/// Web用時間プロバイダーファクトリー関数（将来実装予定）        Box::new(WasmTimeProvider::new())

pub fn create_web_time_provider() -> Box<dyn TimeProvider> {    }

    Box::new(WebTimeProvider::default())}

}

#[cfg(test)]

#[cfg(test)]mod tests {

mod tests {    use super::*;

    use super::*;

        #[test]

    #[test]    fn test_unified_scheduler() {

    fn test_unified_scheduler_basic() {        let mut scheduler = UnifiedScheduler::new();

        let mut scheduler = UnifiedScheduler::new();        

                // 100msタイマーを追加

        // 自動落下タイマーを設定        scheduler.add_timer(Duration::from_millis(100), GameEvent::AutoFall, false);

        let timer_id = scheduler.set_auto_fall_timer(Duration::from_millis(100));        

                // 50ms経過 - イベントなし

        // 初期状態では発火しない        let events = scheduler.update(Duration::from_millis(50));

        let events = scheduler.update(Duration::from_millis(50));        assert_eq!(events.len(), 0);

        assert!(events.is_empty());        

                // さらに60ms経過（合計110ms） - イベント発火

        // 時間経過後に発火        let events = scheduler.update(Duration::from_millis(60));

        let events = scheduler.update(Duration::from_millis(150));        assert_eq!(events.len(), 1);

        assert_eq!(events.len(), 1);        assert_eq!(events[0], GameEvent::AutoFall);

        assert_eq!(events[0], GameEvent::AutoFall);        

                // タイマーは一回限りなので削除されている

        // タイマーを削除        let events = scheduler.update(Duration::from_millis(100));

        scheduler.remove_timer(timer_id);        assert_eq!(events.len(), 0);

        let events = scheduler.update(Duration::from_millis(300));    }

        assert!(events.iter().all(|e| *e != GameEvent::AutoFall));    

    }    #[test]

        fn test_repeating_timer() {

    #[test]        let mut scheduler = UnifiedScheduler::new();

    fn test_time_provider() {        

        let provider = NativeTimeProvider::new();        // 50ms間隔の繰り返しタイマー

        let time1 = provider.now();        scheduler.add_timer(Duration::from_millis(50), GameEvent::Render, true);

        std::thread::sleep(Duration::from_millis(1));        

        let time2 = provider.now();        // 100ms経過 - 2回発火するはず

        assert!(time2 > time1);        let events = scheduler.update(Duration::from_millis(100));

    }        assert_eq!(events.len(), 2);

            assert_eq!(events[0], GameEvent::Render);

    #[test]        assert_eq!(events[1], GameEvent::Render);

    fn test_immediate_events() {    }

        let mut scheduler = UnifiedScheduler::new();}
        
        scheduler.add_immediate_event(GameEvent::GameOver);
        scheduler.add_immediate_event(GameEvent::ShowTitle);
        
        let events = scheduler.update(Duration::ZERO);
        assert_eq!(events.len(), 2);
        assert!(events.contains(&GameEvent::GameOver));
        assert!(events.contains(&GameEvent::ShowTitle));
        
        // 即座イベントは一度しか発火しない
        let events = scheduler.update(Duration::from_millis(100));
        assert!(events.iter().all(|e| *e != GameEvent::GameOver && *e != GameEvent::ShowTitle));
    }
}