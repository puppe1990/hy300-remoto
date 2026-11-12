//! Dual of `web/js/power.mjs` — keep goldens in both suites.
//! Android 12 power-off confirm is `LegacyGlobalActions`; SoC/ADB stay up.

pub const POWER_OFF_FOCUS: &str = "GlobalActions";

/// Lamp/user power from `dumpsys window displays`. ADB can still be up.
///
/// ```
/// use hy300_remoto::power::from_window_dump;
/// assert_eq!(
///     from_window_dump("mCurrentFocus=Window{c85f1b1 u0 LegacyGlobalActions}"),
///     "off"
/// );
/// ```
pub fn from_window_dump(raw: &str) -> &'static str {
    let line = raw.lines().find(|row| row.contains("mCurrentFocus="));
    match line {
        Some(focus) if focus.contains(POWER_OFF_FOCUS) => "off",
        _ => "on",
    }
}

/// Header power: Android confirm or GPIO lamp rails.
///
/// ```
/// use hy300_remoto::power::combine_power;
/// assert_eq!(combine_power("on", false), "off");
/// ```
pub fn combine_power(focus: &str, lamp_on: bool) -> &'static str {
    if focus == "off" || !lamp_on {
        "off"
    } else {
        "on"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;

    #[test]
    fn global_actions_focus_is_standby() {
        assert_eq!(POWER_OFF_FOCUS, "GlobalActions");
        let dump = "  mCurrentFocus=Window{c85f1b1 u0 LegacyGlobalActions}\n  mFocusedApp=ActivityRecord{dc5603d u0 com.bozee.usbdisplay/.StartupActivity t220}";
        assert_eq!(from_window_dump(dump), "off");
    }

    #[test]
    fn app_focus_is_on() {
        let pkg: String = (4..12).fake();
        let dump = format!(
            "  mCurrentFocus=Window{{ac9b6f0 u0 com.{pkg}.usbdisplay/com.{pkg}.MainActivity}}"
        );
        assert_eq!(from_window_dump(&dump), "on");
    }

    #[test]
    fn empty_or_missing_focus_stays_on() {
        assert_eq!(from_window_dump(""), "on");
        let junk: String = (8..24).fake();
        assert_eq!(from_window_dump(&junk), "on");
    }

    #[test]
    fn lamp_gpio_or_global_actions_is_off() {
        assert_eq!(combine_power("on", true), "on");
        assert_eq!(combine_power("off", true), "off");
        assert_eq!(combine_power("on", false), "off");
        assert_eq!(combine_power("off", false), "off");
    }
}
