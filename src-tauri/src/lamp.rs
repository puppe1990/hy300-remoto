//! Dual of `web/js/lamp.mjs` — keep goldens in both suites.
//! TXCZ LED rails. Not KEYCODE_POWER (that opens Android shutdown).

pub const LAMP_EN_PATH: &str = "/sys/class/leds/Lcd_en/brightness";
pub const LAMP_POWER_PATH: &str = "/sys/class/leds/Lcd_power/brightness";
pub const LAMP_ON_VALUE: i32 = 255;
pub const LAMP_OFF_VALUE: i32 = 0;

/// Parse `cat Lcd_power/brightness`. Empty or 0 → off.
///
/// ```
/// use hy300_remoto::lamp::parse_lamp;
/// assert_eq!(parse_lamp("255\n").unwrap(), true);
/// ```
pub fn parse_lamp(raw: &str) -> Result<bool, String> {
    let text = raw.trim();
    if text.is_empty() {
        return Ok(false);
    }
    if !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("brilho da tela {raw:?} não é 0 ou 255"));
    }
    let n: i32 = text
        .parse()
        .map_err(|_| format!("brilho da tela {raw:?} não é 0 ou 255"))?;
    Ok(n > 0)
}

/// Pad id from a `[data-lamp]` value.
///
/// ```
/// use hy300_remoto::lamp::from_pad;
/// assert_eq!(from_pad("lamp_off"), Some("lamp_off"));
/// ```
pub fn from_pad(id: &str) -> Option<&'static str> {
    match id {
        "lamp_on" => Some("lamp_on"),
        "lamp_off" => Some("lamp_off"),
        _ => None,
    }
}

pub fn lamp_on_from_pad(id: &str) -> Option<bool> {
    match id {
        "lamp_on" => Some(true),
        "lamp_off" => Some(false),
        _ => None,
    }
}

pub fn write_value(on: bool) -> i32 {
    if on {
        LAMP_ON_VALUE
    } else {
        LAMP_OFF_VALUE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;

    #[test]
    fn gpio_zero_is_off_255_is_on() {
        assert_eq!(LAMP_EN_PATH, "/sys/class/leds/Lcd_en/brightness");
        assert_eq!(LAMP_POWER_PATH, "/sys/class/leds/Lcd_power/brightness");
        assert_eq!(LAMP_ON_VALUE, 255);
        assert_eq!(LAMP_OFF_VALUE, 0);
        assert_eq!(parse_lamp("255\n").unwrap(), true);
        assert_eq!(parse_lamp("0").unwrap(), false);
        assert_eq!(parse_lamp("").unwrap(), false);
    }

    #[test]
    fn parse_rejects_junk_and_cites_value() {
        let junk = format!("abc{}", (1u32..999u32).fake::<u32>());
        let err = parse_lamp(&junk).unwrap_err();
        assert!(err.contains(&junk), "{err}");
        assert!(err.contains("0 ou 255"), "{err}");
    }

    #[test]
    fn pad_tela_on_off() {
        assert_eq!(from_pad("lamp_on"), Some("lamp_on"));
        assert_eq!(from_pad("lamp_off"), Some("lamp_off"));
        assert_eq!(lamp_on_from_pad("lamp_on"), Some(true));
        assert_eq!(lamp_on_from_pad("lamp_off"), Some(false));
        let junk: String = (6..12).fake();
        assert_eq!(from_pad(&junk), None);
    }
}
