//! Dual of `web/js/mouse.mjs` — keep goldens in both suites.

pub const MOUSE_COALESCE_MS: u64 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MousePoint {
    pub action: &'static str,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EscapeMouse {
    pub on: bool,
    pub consume: bool,
}

/// Android `input mouse motionevent` action. Case-sensitive.
///
/// ```
/// use hy300_remoto::mouse::parse_mouse_action;
/// assert_eq!(parse_mouse_action("MOVE"), Some("MOVE"));
/// ```
pub fn parse_mouse_action(value: &str) -> Option<&'static str> {
    match value {
        "MOVE" => Some("MOVE"),
        "DOWN" => Some("DOWN"),
        "UP" => Some("UP"),
        _ => None,
    }
}

/// Error when the action is not MOVE|DOWN|UP.
///
/// ```
/// use hy300_remoto::mouse::invalid_mouse_action;
/// assert!(invalid_mouse_action("tap").contains("MOVE|DOWN|UP"));
/// ```
pub fn invalid_mouse_action(value: &str) -> String {
    format!("ação mouse {value} não é MOVE|DOWN|UP")
}

/// Skip MOVE when (x, y) matches the last sent mouse event.
///
/// ```
/// use hy300_remoto::mouse::{should_skip_duplicate_move, MousePoint};
/// let mv = MousePoint { action: "MOVE", x: 1, y: 1 };
/// assert!(should_skip_duplicate_move(Some(mv), mv));
/// ```
pub fn should_skip_duplicate_move(last: Option<MousePoint>, next: MousePoint) -> bool {
    if next.action != "MOVE" {
        return false;
    }
    let Some(last) = last else {
        return false;
    };
    last.x == next.x && last.y == next.y
}

/// Flip the session mouse mode.
///
/// ```
/// use hy300_remoto::mouse::toggle_mouse_mode;
/// assert_eq!(toggle_mouse_mode(false), true);
/// ```
pub fn toggle_mouse_mode(on: bool) -> bool {
    !on
}

/// Esc with mode on exits and must not send BACK.
///
/// ```
/// use hy300_remoto::mouse::escape_exits_mouse_mode;
/// assert!(escape_exits_mouse_mode(true, "Escape").consume);
/// ```
pub fn escape_exits_mouse_mode(on: bool, event_code: &str) -> EscapeMouse {
    if on && event_code == "Escape" {
        return EscapeMouse {
            on: false,
            consume: true,
        };
    }
    EscapeMouse { on, consume: false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;

    #[test]
    fn coalesce_is_50_ms() {
        assert_eq!(MOUSE_COALESCE_MS, 50);
    }

    #[test]
    fn parse_accepts_move_down_up_and_rejects_rest() {
        assert_eq!(parse_mouse_action("MOVE"), Some("MOVE"));
        assert_eq!(parse_mouse_action("DOWN"), Some("DOWN"));
        assert_eq!(parse_mouse_action("UP"), Some("UP"));
        assert_eq!(parse_mouse_action("move"), None);
        assert_eq!(parse_mouse_action("Tap"), None);
        assert_eq!(parse_mouse_action("HOVER"), None);
        assert_eq!(parse_mouse_action(""), None);
        let junk = format!("jx{}", (1u32..999u32).fake::<u32>());
        assert_eq!(parse_mouse_action(&junk), None);
        let err = invalid_mouse_action(&junk);
        assert!(err.contains(&junk), "{err}");
        assert!(err.contains("MOVE|DOWN|UP"), "{err}");
    }

    #[test]
    fn skips_move_on_same_pixel_as_last() {
        let x: i32 = (0i32..1280i32).fake();
        let y: i32 = (0i32..720i32).fake();
        let mv = MousePoint {
            action: "MOVE",
            x,
            y,
        };
        assert!(!should_skip_duplicate_move(None, mv));
        assert!(should_skip_duplicate_move(Some(mv), mv));
        assert!(should_skip_duplicate_move(
            Some(MousePoint {
                action: "DOWN",
                x,
                y
            }),
            mv
        ));
        assert!(!should_skip_duplicate_move(
            Some(mv),
            MousePoint {
                action: "MOVE",
                x: x + 1,
                y
            }
        ));
        assert!(!should_skip_duplicate_move(
            Some(mv),
            MousePoint {
                action: "DOWN",
                x,
                y
            }
        ));
        assert!(!should_skip_duplicate_move(
            Some(mv),
            MousePoint { action: "UP", x, y }
        ));
    }

    #[test]
    fn toggle_flips_mode() {
        assert!(toggle_mouse_mode(false));
        assert!(!toggle_mouse_mode(true));
    }

    #[test]
    fn escape_with_mode_on_consumes_and_exits() {
        assert_eq!(
            escape_exits_mouse_mode(true, "Escape"),
            EscapeMouse {
                on: false,
                consume: true
            }
        );
        assert_eq!(
            escape_exits_mouse_mode(false, "Escape"),
            EscapeMouse {
                on: false,
                consume: false
            }
        );
        assert_eq!(
            escape_exits_mouse_mode(true, "KeyH"),
            EscapeMouse {
                on: true,
                consume: false
            }
        );
    }
}
