//! Dual of `web/js/swipe.mjs` — keep goldens in both suites.

use crate::tap::ScreenSize;

pub const SWIPE_COALESCE_MS: u64 = 50;
pub const SWIPE_DURATION_MS: u32 = 200;
pub const SWIPE_MIN_TRAVEL: i32 = 120;
pub const SWIPE_MAX_TRAVEL: i32 = 240;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Swipe {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub duration_ms: u32,
}

/// Convert a WheelEvent deltaY into pixels. Unknown deltaMode is ignored.
///
/// ```
/// use hy300_remoto::swipe::normalize_wheel_delta;
/// assert_eq!(normalize_wheel_delta(2.0, 1), Some(32.0));
/// ```
pub fn normalize_wheel_delta(delta_y: f64, delta_mode: u32) -> Option<f64> {
    match delta_mode {
        0 => Some(delta_y),
        1 => Some(delta_y * 16.0),
        2 => Some(delta_y * 400.0),
        _ => None,
    }
}

/// Finger travel in pixels. One notch saturates at 120; a flick at 240.
///
/// ```
/// use hy300_remoto::swipe::swipe_travel;
/// assert_eq!(swipe_travel(4.0), 120);
/// ```
pub fn swipe_travel(abs_normalized_delta: f64) -> i32 {
    let raw = (abs_normalized_delta.abs() * 4.0).round() as i32;
    raw.clamp(SWIPE_MIN_TRAVEL, SWIPE_MAX_TRAVEL)
}

/// Map normalized wheel delta + origin pixel to a vertical `input swipe`.
/// `delta_y > 0` (scroll down) moves the finger up.
///
/// ```
/// use hy300_remoto::swipe::wheel_to_swipe;
/// use hy300_remoto::tap::HY300_SCREEN;
/// let swipe = wheel_to_swipe(30.0, Some((640, 360)), HY300_SCREEN).unwrap();
/// assert_eq!(swipe.y2, 240);
/// ```
pub fn wheel_to_swipe(
    normalized_delta_y: f64,
    origin: Option<(i32, i32)>,
    screen: ScreenSize,
) -> Option<Swipe> {
    let (x1, y1) = origin?;
    if normalized_delta_y == 0.0 || !normalized_delta_y.is_finite() {
        return None;
    }
    if screen.width == 0 || screen.height == 0 {
        return None;
    }
    let y2 = finger_end_y(y1, normalized_delta_y, screen.height);
    if y2 == y1 {
        return None;
    }
    Some(vertical_swipe(x1, y1, y2))
}

fn finger_end_y(y1: i32, normalized_delta_y: f64, height: u32) -> i32 {
    let travel = swipe_travel(normalized_delta_y.abs());
    let dir = if normalized_delta_y > 0.0 { -1 } else { 1 };
    let max_y = height.saturating_sub(1) as i32;
    (y1 + dir * travel).clamp(0, max_y)
}

fn vertical_swipe(x1: i32, y1: i32, y2: i32) -> Swipe {
    Swipe {
        x1,
        y1,
        x2: x1,
        y2,
        duration_ms: SWIPE_DURATION_MS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tap::HY300_SCREEN;
    use fake::Fake;

    const CENTER: (i32, i32) = (640, 360);

    #[test]
    fn constants_match_js() {
        assert_eq!(SWIPE_COALESCE_MS, 50);
        assert_eq!(SWIPE_DURATION_MS, 200);
    }

    #[test]
    fn delta_mode_normalizes() {
        let dy: f64 = (1u32..10u32).fake::<u32>() as f64;
        assert_eq!(normalize_wheel_delta(dy, 0), Some(dy));
        assert_eq!(normalize_wheel_delta(dy, 1), Some(dy * 16.0));
        assert_eq!(normalize_wheel_delta(dy, 2), Some(dy * 400.0));
        assert_eq!(normalize_wheel_delta(dy, 99), None);
    }

    #[test]
    fn travel_clamps_120_240() {
        assert_eq!(swipe_travel(4.0), 120);
        assert_eq!(swipe_travel(40.0), 160);
        assert_eq!(swipe_travel(60.0), 240);
    }

    #[test]
    fn scroll_down_at_center_moves_finger_up() {
        let swipe = wheel_to_swipe(30.0, Some(CENTER), HY300_SCREEN).unwrap();
        assert_eq!(
            swipe,
            Swipe {
                x1: 640,
                y1: 360,
                x2: 640,
                y2: 240,
                duration_ms: 200,
            }
        );
    }

    #[test]
    fn flick_saturates_travel_at_240() {
        let swipe = wheel_to_swipe(80.0, Some(CENTER), HY300_SCREEN).unwrap();
        assert_eq!(swipe.y2, 120);
        assert_eq!(swipe.x1, swipe.x2);
    }

    #[test]
    fn scroll_up_at_center_moves_finger_down() {
        let swipe = wheel_to_swipe(-30.0, Some(CENTER), HY300_SCREEN).unwrap();
        assert_eq!(swipe.y2, 480);
    }

    #[test]
    fn zero_delta_is_none() {
        assert_eq!(wheel_to_swipe(0.0, Some(CENTER), HY300_SCREEN), None);
    }

    #[test]
    fn missing_origin_is_none() {
        assert_eq!(wheel_to_swipe(30.0, None, HY300_SCREEN), None);
    }

    #[test]
    fn top_edge_cannot_swipe_up() {
        assert_eq!(wheel_to_swipe(30.0, Some((640, 0)), HY300_SCREEN), None);
    }

    #[test]
    fn bottom_edge_cannot_swipe_down() {
        assert_eq!(wheel_to_swipe(-30.0, Some((640, 719)), HY300_SCREEN), None);
    }
}
