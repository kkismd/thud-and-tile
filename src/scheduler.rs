//! スケジューラー抽象化モジュール
//! 
//! このモジュールは、プラットフォーム独立なタイミング制御を提供します。
//! ネイティブ環境ではthread::sleepを、WASM環境ではrequestAnimationFrameを使用します。

use std::time::Duration;

/// プラットフォーム独立なスケジューリングを提供するトレイト
pub trait Scheduler {
    /// 指定された時間だけ処理を遅延させる
    /// ネイティブ環境ではthread::sleep、WASM環境ではrequestAnimationFrameを使用
    fn sleep(&self, duration: Duration);
    
    /// 次のフレームまで待機する（通常16ms程度）
    /// ゲームループでの使用に最適化
    fn wait_for_next_frame(&self) {
        self.sleep(Duration::from_millis(16));
    }
}

/// ネイティブ環境用のスケジューラー
pub struct NativeScheduler;

impl NativeScheduler {
    pub fn new() -> Self {
        Self
    }
}

impl Scheduler for NativeScheduler {
    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

/// Web/WASM環境用のスケジューラー
#[cfg(target_arch = "wasm32")]
pub struct WebScheduler;

#[cfg(target_arch = "wasm32")]
impl WebScheduler {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_arch = "wasm32")]
impl Scheduler for WebScheduler {
    fn sleep(&self, duration: Duration) {
        // WASM環境では同期的なsleepは利用できないため、
        // 将来的にはPromise/async/awaitベースの実装に変更する必要がある
        // 現在は簡単な実装として何もしない（busy wait回避）
        // 本格的なWASM実装では、requestAnimationFrameやsetTimeoutを使用
        
        // 簡易的な実装：ビジーウェイトで時間を消費
        let start = web_sys::window()
            .unwrap()
            .performance()
            .unwrap()
            .now();
        
        let target_ms = duration.as_millis() as f64;
        
        loop {
            let now = web_sys::window()
                .unwrap()
                .performance()
                .unwrap()
                .now();
            
            if now - start >= target_ms {
                break;
            }
        }
    }
}

/// テスト用の決定的スケジューラー
pub struct DeterministicScheduler {
    sleep_count: std::cell::RefCell<usize>,
}

impl DeterministicScheduler {
    pub fn new() -> Self {
        Self {
            sleep_count: std::cell::RefCell::new(0),
        }
    }
    
    pub fn get_sleep_count(&self) -> usize {
        *self.sleep_count.borrow()
    }
    
    pub fn reset_sleep_count(&self) {
        *self.sleep_count.borrow_mut() = 0;
    }
}

impl Scheduler for DeterministicScheduler {
    fn sleep(&self, _duration: Duration) {
        // テスト用では実際にはsleepせず、カウントのみ記録
        *self.sleep_count.borrow_mut() += 1;
    }
}

/// Schedulerの具象実装のenum
pub enum SchedulerImpl {
    Native(NativeScheduler),
    #[cfg(target_arch = "wasm32")]
    Web(WebScheduler),
    Deterministic(DeterministicScheduler),
}

impl Scheduler for SchedulerImpl {
    fn sleep(&self, duration: Duration) {
        match self {
            SchedulerImpl::Native(scheduler) => scheduler.sleep(duration),
            #[cfg(target_arch = "wasm32")]
            SchedulerImpl::Web(scheduler) => scheduler.sleep(duration),
            SchedulerImpl::Deterministic(scheduler) => scheduler.sleep(duration),
        }
    }
}

/// デフォルトのSchedulerを作成する便利関数
pub fn create_default_scheduler() -> SchedulerImpl {
    #[cfg(not(target_arch = "wasm32"))]
    {
        SchedulerImpl::Native(NativeScheduler::new())
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        SchedulerImpl::Web(WebScheduler::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_scheduler() {
        let scheduler = DeterministicScheduler::new();
        
        assert_eq!(scheduler.get_sleep_count(), 0);
        
        scheduler.sleep(Duration::from_millis(100));
        assert_eq!(scheduler.get_sleep_count(), 1);
        
        scheduler.wait_for_next_frame();
        assert_eq!(scheduler.get_sleep_count(), 2);
        
        scheduler.reset_sleep_count();
        assert_eq!(scheduler.get_sleep_count(), 0);
    }

    #[test]
    fn test_native_scheduler() {
        let scheduler = NativeScheduler::new();
        let start = std::time::Instant::now();
        
        scheduler.sleep(Duration::from_millis(10));
        
        let elapsed = start.elapsed();
        // 最低でも要求した時間は経過している（システムの精度により多少の誤差は許容）
        assert!(elapsed >= Duration::from_millis(8));
    }

    #[test]
    fn test_scheduler_impl_enum() {
        let scheduler = SchedulerImpl::Deterministic(DeterministicScheduler::new());
        
        let start = std::time::Instant::now();
        scheduler.sleep(Duration::from_millis(1));
        let elapsed = start.elapsed();
        
        // DeterministicSchedulerは実際にはsleepしないので、すぐに完了する
        assert!(elapsed < Duration::from_millis(5));
    }

    #[test]
    fn test_create_default_scheduler() {
        let scheduler = create_default_scheduler();
        
        // デフォルトスケジューラーが作成できることを確認
        let start = std::time::Instant::now();
        scheduler.sleep(Duration::from_millis(1));
        let elapsed = start.elapsed();
        
        // ネイティブ環境では実際にsleepするはず
        #[cfg(not(target_arch = "wasm32"))]
        assert!(elapsed >= Duration::from_millis(1));
    }
}