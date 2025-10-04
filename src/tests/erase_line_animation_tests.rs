//! EraseLineアニメーション関連のテスト
//! Phase 5の実装に対応するテスト群

use crate::animation::{process_erase_line_step, Animation, EraseLineStepResult};
use std::time::Duration;

#[test]
fn test_erase_line_animation_creation() {
    let animation = Animation::EraseLine {
        lines_remaining: 3,
        last_update: Duration::from_millis(0),
    };

    if let Animation::EraseLine {
        lines_remaining,
        last_update,
    } = animation
    {
        assert_eq!(lines_remaining, 3);
        assert_eq!(last_update, Duration::from_millis(0));
    } else {
        panic!("Expected EraseLine animation");
    }
}

#[test]
fn test_erase_line_animation_fields() {
    let animation = Animation::EraseLine {
        lines_remaining: 5,
        last_update: Duration::from_millis(100),
    };

    match animation {
        Animation::EraseLine {
            lines_remaining,
            last_update,
        } => {
            assert_eq!(lines_remaining, 5);
            assert_eq!(last_update, Duration::from_millis(100));
        }
        _ => panic!("Expected EraseLine animation"),
    }
}

#[test]
fn test_erase_line_animation_progress() {
    let mut animation = Animation::EraseLine {
        lines_remaining: 3,
        last_update: Duration::from_millis(0),
    };

    let result = process_erase_line_step(&mut animation, Duration::from_millis(100));
    assert!(matches!(result, EraseLineStepResult::Continue));

    if let Animation::EraseLine {
        lines_remaining, ..
    } = animation
    {
        assert_eq!(lines_remaining, 2);
    }
}

#[test]
fn test_erase_line_animation_completion() {
    let mut animation = Animation::EraseLine {
        lines_remaining: 1,
        last_update: Duration::from_millis(0),
    };

    let result = process_erase_line_step(&mut animation, Duration::from_millis(100));
    assert!(matches!(result, EraseLineStepResult::Complete));

    if let Animation::EraseLine {
        lines_remaining, ..
    } = animation
    {
        assert_eq!(lines_remaining, 0);
    }
}
