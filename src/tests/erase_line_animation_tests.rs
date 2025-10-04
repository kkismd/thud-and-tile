//! EraseLineアニメーション関連のテスト
//! Phase 5の実装に対応するテスト群

use crate::animation::Animation;
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
