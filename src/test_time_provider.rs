//! テスト用のTimeProvider実装

use std::time::Duration;
use crate::unified_scheduler::TimeProvider;

/// テスト制御用のMockTimeProvider（既存テストとの互換性）
pub struct ControllableTimeProvider {
    current_time: Duration,
}

impl ControllableTimeProvider {
    pub fn new() -> Self {
        Self {
            current_time: Duration::ZERO,
        }
    }
    
    pub fn advance(&mut self, duration: Duration) {
        self.current_time += duration;
    }
    
    pub fn set_time(&mut self, time: Duration) {
        self.current_time = time;
    }
}

impl TimeProvider for ControllableTimeProvider {
    fn now(&self) -> Duration {
        self.current_time
    }
}