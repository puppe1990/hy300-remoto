//! Pad e teclado do Mac → keycode Android (AOSP KeyEvent).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteKey {
    Up,
    Down,
    Left,
    Right,
    Ok,
    Back,
    Home,
    VolumeUp,
    VolumeDown,
    VolumeMute,
    Power,
    Rewind,
    FastForward,
}

pub fn keycode(key: RemoteKey) -> i32 {
    match key {
        RemoteKey::Up => 19,
        RemoteKey::Down => 20,
        RemoteKey::Left => 21,
        RemoteKey::Right => 22,
        RemoteKey::Ok => 23,
        RemoteKey::Back => 4,
        RemoteKey::Home => 3,
        RemoteKey::VolumeUp => 24,
        RemoteKey::VolumeDown => 25,
        RemoteKey::VolumeMute => 164,
        RemoteKey::Power => 26,
        RemoteKey::Rewind => 89,
        RemoteKey::FastForward => 90,
    }
}

pub fn from_pad(id: &str) -> Option<RemoteKey> {
    match id {
        "up" => Some(RemoteKey::Up),
        "down" => Some(RemoteKey::Down),
        "left" => Some(RemoteKey::Left),
        "right" => Some(RemoteKey::Right),
        "ok" => Some(RemoteKey::Ok),
        "back" => Some(RemoteKey::Back),
        "home" => Some(RemoteKey::Home),
        "vol_up" => Some(RemoteKey::VolumeUp),
        "vol_down" => Some(RemoteKey::VolumeDown),
        "mute" => Some(RemoteKey::VolumeMute),
        "power" => Some(RemoteKey::Power),
        "rew" => Some(RemoteKey::Rewind),
        "ff" => Some(RemoteKey::FastForward),
        _ => None,
    }
}

pub fn from_event_code(code: &str) -> Option<RemoteKey> {
    match code {
        "ArrowUp" => Some(RemoteKey::Up),
        "ArrowDown" => Some(RemoteKey::Down),
        "ArrowLeft" => Some(RemoteKey::Left),
        "ArrowRight" => Some(RemoteKey::Right),
        "Enter" | "NumpadEnter" => Some(RemoteKey::Ok),
        "Escape" => Some(RemoteKey::Back),
        "KeyH" => Some(RemoteKey::Home),
        "Equal" | "NumpadAdd" => Some(RemoteKey::VolumeUp),
        "Minus" | "NumpadSubtract" => Some(RemoteKey::VolumeDown),
        "KeyM" => Some(RemoteKey::VolumeMute),
        "BracketLeft" => Some(RemoteKey::Rewind),
        "BracketRight" => Some(RemoteKey::FastForward),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;

    #[test]
    fn pad_ids_match_aosp_keycodes() {
        assert_eq!(keycode(from_pad("up").unwrap()), 19);
        assert_eq!(keycode(from_pad("down").unwrap()), 20);
        assert_eq!(keycode(from_pad("left").unwrap()), 21);
        assert_eq!(keycode(from_pad("right").unwrap()), 22);
        assert_eq!(keycode(from_pad("ok").unwrap()), 23);
        assert_eq!(keycode(from_pad("back").unwrap()), 4);
        assert_eq!(keycode(from_pad("home").unwrap()), 3);
        assert_eq!(keycode(from_pad("vol_up").unwrap()), 24);
        assert_eq!(keycode(from_pad("vol_down").unwrap()), 25);
        assert_eq!(keycode(from_pad("mute").unwrap()), 164);
        assert_eq!(keycode(from_pad("power").unwrap()), 26);
        assert_eq!(keycode(from_pad("rew").unwrap()), 89);
        assert_eq!(keycode(from_pad("ff").unwrap()), 90);
    }

    #[test]
    fn event_codes_match_the_same_keycodes() {
        assert_eq!(from_event_code("ArrowUp"), from_pad("up"));
        assert_eq!(from_event_code("ArrowDown"), from_pad("down"));
        assert_eq!(from_event_code("ArrowLeft"), from_pad("left"));
        assert_eq!(from_event_code("ArrowRight"), from_pad("right"));
        assert_eq!(from_event_code("Enter"), from_pad("ok"));
        assert_eq!(from_event_code("NumpadEnter"), from_pad("ok"));
        assert_eq!(from_event_code("Escape"), from_pad("back"));
        assert_eq!(from_event_code("KeyH"), from_pad("home"));
        assert_eq!(from_event_code("Equal"), from_pad("vol_up"));
        assert_eq!(from_event_code("NumpadAdd"), from_pad("vol_up"));
        assert_eq!(from_event_code("Minus"), from_pad("vol_down"));
        assert_eq!(from_event_code("NumpadSubtract"), from_pad("vol_down"));
        assert_eq!(from_event_code("KeyM"), from_pad("mute"));
        assert_eq!(from_event_code("BracketLeft"), from_pad("rew"));
        assert_eq!(from_event_code("BracketRight"), from_pad("ff"));
    }

    #[test]
    fn unknown_pad_and_code_are_ignored() {
        let junk: String = (8..24).fake::<String>();
        assert_eq!(from_pad(&junk), None);
        assert_eq!(from_event_code(&junk), None);
        assert_eq!(from_pad(""), None);
        assert_eq!(from_event_code("KeyA"), None);
    }
}
