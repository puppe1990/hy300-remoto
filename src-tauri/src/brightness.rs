//! Dual of `web/js/brightness.mjs` — keep goldens in both suites.
//! TXCZ picture brightness is `persist.vendor.brightness.main` (0–100).

pub const BRIGHTNESS_PROP: &str = "persist.vendor.brightness.main";
pub const BRIGHTNESS_MIN: i32 = 0;
pub const BRIGHTNESS_MAX: i32 = 100;
pub const BRIGHTNESS_STEP: i32 = 5;
pub const BRIGHTNESS_DEFAULT: i32 = 50;

/// Clamp picture brightness to the TXCZ 0–100 range.
///
/// ```
/// use hy300_remoto::brightness::clamp_brightness;
/// assert_eq!(clamp_brightness(140), 100);
/// ```
pub fn clamp_brightness(n: i32) -> i32 {
    n.clamp(BRIGHTNESS_MIN, BRIGHTNESS_MAX)
}

/// Parse `getprop persist.vendor.brightness.main`. Empty → 50.
///
/// ```
/// use hy300_remoto::brightness::parse_brightness;
/// assert_eq!(parse_brightness("50\n").unwrap(), 50);
/// ```
pub fn parse_brightness(raw: &str) -> Result<i32, String> {
    let text = raw.trim();
    if text.is_empty() {
        return Ok(BRIGHTNESS_DEFAULT);
    }
    let n: i32 = text
        .parse()
        .map_err(|_| format!("brilho {}; expected inteiro 0–100", format_raw(raw)))?;
    if n < BRIGHTNESS_MIN || n > BRIGHTNESS_MAX {
        return Err(format!(
            "brilho {}; expected inteiro 0–100",
            format_raw(raw)
        ));
    }
    Ok(n)
}

fn format_raw(raw: &str) -> String {
    format!("{raw:?}")
}

/// Next brightness after a pad step. Saturates at 0 and 100.
///
/// ```
/// use hy300_remoto::brightness::nudge_brightness;
/// assert_eq!(nudge_brightness(47, 5), 52);
/// ```
pub fn nudge_brightness(current: i32, delta: i32) -> i32 {
    clamp_brightness(current.saturating_add(delta))
}

/// Pad id from a `[data-bright]` value.
///
/// ```
/// use hy300_remoto::brightness::from_pad;
/// assert_eq!(from_pad("bright_up"), Some("bright_up"));
/// ```
pub fn from_pad(id: &str) -> Option<&'static str> {
    match id {
        "bright_down" => Some("bright_down"),
        "bright_up" => Some("bright_up"),
        _ => None,
    }
}

/// Pad id from a KeyboardEvent.code.
///
/// ```
/// use hy300_remoto::brightness::from_event_code;
/// assert_eq!(from_event_code("Comma"), Some("bright_down"));
/// ```
pub fn from_event_code(code: &str) -> Option<&'static str> {
    match code {
        "Comma" => Some("bright_down"),
        "Period" => Some("bright_up"),
        _ => None,
    }
}

/// ±5 for a brightness pad id.
///
/// ```
/// use hy300_remoto::brightness::delta_from_pad;
/// assert_eq!(delta_from_pad("bright_down"), Some(-5));
/// ```
pub fn delta_from_pad(id: &str) -> Option<i32> {
    match from_pad(id)? {
        "bright_down" => Some(-BRIGHTNESS_STEP),
        "bright_up" => Some(BRIGHTNESS_STEP),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;

    #[test]
    fn constants_match_txcz_pq_range() {
        assert_eq!(BRIGHTNESS_PROP, "persist.vendor.brightness.main");
        assert_eq!(BRIGHTNESS_MIN, 0);
        assert_eq!(BRIGHTNESS_MAX, 100);
        assert_eq!(BRIGHTNESS_STEP, 5);
        assert_eq!(BRIGHTNESS_DEFAULT, 50);
    }

    #[test]
    fn parse_reads_int_with_newline_and_empty_default() {
        let n: i32 = (0i32..=100i32).fake();
        assert_eq!(parse_brightness(&format!("{n}\n")).unwrap(), n);
        assert_eq!(parse_brightness("").unwrap(), BRIGHTNESS_DEFAULT);
        assert_eq!(parse_brightness("  ").unwrap(), BRIGHTNESS_DEFAULT);
    }

    #[test]
    fn parse_rejects_non_int_and_cites_value() {
        let junk = format!("abc{}", (1u32..999u32).fake::<u32>());
        let err = parse_brightness(&junk).unwrap_err();
        assert!(err.contains(&junk), "{err}");
        assert!(err.contains("0–100"), "{err}");
    }

    #[test]
    fn nudge_clamps_at_ends() {
        assert_eq!(nudge_brightness(0, -BRIGHTNESS_STEP), 0);
        assert_eq!(nudge_brightness(100, BRIGHTNESS_STEP), 100);
        assert_eq!(nudge_brightness(47, BRIGHTNESS_STEP), 52);
        assert_eq!(clamp_brightness(-3), 0);
        assert_eq!(clamp_brightness(140), 100);
    }

    #[test]
    fn pad_and_mac_keys_share_delta() {
        assert_eq!(from_pad("bright_down"), Some("bright_down"));
        assert_eq!(from_pad("bright_up"), Some("bright_up"));
        assert_eq!(from_event_code("Comma"), Some("bright_down"));
        assert_eq!(from_event_code("Period"), Some("bright_up"));
        assert_eq!(delta_from_pad("bright_down"), Some(-BRIGHTNESS_STEP));
        assert_eq!(delta_from_pad("bright_up"), Some(BRIGHTNESS_STEP));
    }

    #[test]
    fn unknown_pad_and_code_are_ignored() {
        let junk: String = (8..24).fake();
        assert_eq!(from_pad(&junk), None);
        assert_eq!(from_event_code(&junk), None);
        assert_eq!(delta_from_pad(&junk), None);
    }
}
