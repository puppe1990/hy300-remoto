//! ADB CLI. Testes cobrem argv; o binário não roda na CI.

pub mod raw;
mod run;

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

pub const SCREENCAP_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdbError {
    pub message: String,
}

impl AdbError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// CLI `adb` behind this trait so tests use [`FakeAdb`].
///
/// ```
/// use hy300_remoto::adb::{AdbDevice, FakeAdb};
/// let fake = FakeAdb::default();
/// fake.keyevent(23).unwrap();
/// assert_eq!(*fake.keys.borrow(), vec![23]);
/// ```
pub trait AdbDevice {
    fn connect(&mut self) -> Result<(), AdbError>;
    fn keyevent(&self, code: i32) -> Result<(), AdbError>;
    fn tap(&self, x: i32, y: i32) -> Result<(), AdbError>;
    fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<(), AdbError>;
    fn mouse(&self, action: &str, x: i32, y: i32) -> Result<(), AdbError>;
    fn screencap_png(&self) -> Result<Vec<u8>, AdbError>;
    fn getprop(&self, name: &str) -> Result<String, AdbError>;
    fn setprop(&self, name: &str, value: &str) -> Result<(), AdbError>;
    fn testgamma_brightness(&self, value: i32) -> Result<(), AdbError>;
    fn window_dump(&self) -> Result<String, AdbError>;
    fn lamp_read(&self) -> Result<String, AdbError>;
    fn lamp_write(&self, on: bool) -> Result<(), AdbError>;
}

pub fn connect_args(host: &str) -> Vec<String> {
    vec!["connect".into(), host.into()]
}

pub fn keyevent_args(host: &str, code: i32) -> Vec<String> {
    vec![
        "-s".into(),
        host.into(),
        "shell".into(),
        "input".into(),
        "keyevent".into(),
        code.to_string(),
    ]
}

pub fn tap_args(host: &str, x: i32, y: i32) -> Vec<String> {
    vec![
        "-s".into(),
        host.into(),
        "shell".into(),
        "input".into(),
        "tap".into(),
        x.to_string(),
        y.to_string(),
    ]
}

pub fn swipe_args(host: &str, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Vec<String> {
    vec![
        "-s".into(),
        host.into(),
        "shell".into(),
        "input".into(),
        "swipe".into(),
        x1.to_string(),
        y1.to_string(),
        x2.to_string(),
        y2.to_string(),
        duration_ms.to_string(),
    ]
}

pub fn mouse_args(host: &str, action: &str, x: i32, y: i32) -> Vec<String> {
    vec![
        "-s".into(),
        host.into(),
        "shell".into(),
        "input".into(),
        "mouse".into(),
        "motionevent".into(),
        action.into(),
        x.to_string(),
        y.to_string(),
    ]
}

pub fn screencap_args(host: &str) -> Vec<String> {
    vec![
        "-s".into(),
        host.into(),
        "exec-out".into(),
        "screencap".into(),
    ]
}

pub fn getprop_args(host: &str, name: &str) -> Vec<String> {
    vec![
        "-s".into(),
        host.into(),
        "shell".into(),
        "getprop".into(),
        name.into(),
    ]
}

pub fn setprop_args(host: &str, name: &str, value: &str) -> Vec<String> {
    vec![
        "-s".into(),
        host.into(),
        "shell".into(),
        "setprop".into(),
        name.into(),
        value.into(),
    ]
}

/// Factory `testgammatx.sh` WB/gamma; only `--brightness` changes.
pub fn testgamma_args(host: &str, brightness: i32) -> Vec<String> {
    vec![
        "-s".into(),
        host.into(),
        "shell".into(),
        "testgamma".into(),
        "--wb_r".into(),
        "1.0".into(),
        "--wb_g".into(),
        "0.96".into(),
        "--wb_b".into(),
        "0.95".into(),
        "--contrast".into(),
        "58".into(),
        "--brightness".into(),
        brightness.to_string(),
        "--saturation".into(),
        "80".into(),
        "--hue".into(),
        "35".into(),
        "--target_gamma".into(),
        "1.8".into(),
        "--native_gamma".into(),
        "2.9".into(),
    ]
}

pub fn window_dump_args(host: &str) -> Vec<String> {
    vec![
        "-s".into(),
        host.into(),
        "shell".into(),
        "dumpsys".into(),
        "window".into(),
        "displays".into(),
    ]
}

pub fn lamp_read_args(host: &str) -> Vec<String> {
    vec![
        "-s".into(),
        host.into(),
        "shell".into(),
        "cat".into(),
        crate::lamp::LAMP_POWER_PATH.into(),
    ]
}

pub fn lamp_write_args(host: &str, on: bool) -> Vec<String> {
    let v = crate::lamp::write_value(on);
    vec![
        "-s".into(),
        host.into(),
        "shell".into(),
        "sh".into(),
        "-c".into(),
        format!(
            "echo {v} > {}; echo {v} > {}",
            crate::lamp::LAMP_EN_PATH,
            crate::lamp::LAMP_POWER_PATH
        ),
    ]
}

/// GPIO Lcd_power: true when the lamp rails are up.
pub fn picture_lamp(adb: &impl AdbDevice) -> Result<bool, AdbError> {
    crate::lamp::parse_lamp(&adb.lamp_read()?).map_err(AdbError::new)
}

pub fn set_picture_lamp(adb: &impl AdbDevice, on: bool) -> Result<bool, AdbError> {
    adb.lamp_write(on)?;
    Ok(on)
}

/// `on` unless Android's power-off confirm has focus. SoC/ADB may still be up.
pub fn picture_power(adb: &impl AdbDevice) -> Result<&'static str, AdbError> {
    Ok(crate::power::from_window_dump(&adb.window_dump()?))
}

pub fn picture_brightness(adb: &impl AdbDevice) -> Result<i32, AdbError> {
    let raw = adb.getprop(crate::brightness::BRIGHTNESS_PROP)?;
    crate::brightness::parse_brightness(&raw).map_err(AdbError::new)
}

pub fn nudge_picture_brightness(adb: &impl AdbDevice, delta: i32) -> Result<i32, AdbError> {
    let next = crate::brightness::nudge_brightness(picture_brightness(adb)?, delta);
    adb.setprop(crate::brightness::BRIGHTNESS_PROP, &next.to_string())?;
    adb.testgamma_brightness(next)?;
    Ok(next)
}

pub struct CliAdb {
    pub adb_bin: String,
    pub host: String,
}

impl CliAdb {
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            adb_bin: "adb".into(),
            host: host.into(),
        }
    }

    fn run(&self, args: &[String]) -> Result<Vec<u8>, AdbError> {
        run::run_limited(&self.adb_bin, args, None)
    }
}

impl AdbDevice for CliAdb {
    fn connect(&mut self) -> Result<(), AdbError> {
        self.run(&connect_args(&self.host)).map(|_| ())
    }

    fn keyevent(&self, code: i32) -> Result<(), AdbError> {
        self.run(&keyevent_args(&self.host, code)).map(|_| ())
    }

    fn tap(&self, x: i32, y: i32) -> Result<(), AdbError> {
        self.run(&tap_args(&self.host, x, y)).map(|_| ())
    }

    fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<(), AdbError> {
        self.run(&swipe_args(&self.host, x1, y1, x2, y2, duration_ms))
            .map(|_| ())
    }

    fn mouse(&self, action: &str, x: i32, y: i32) -> Result<(), AdbError> {
        let parsed = crate::mouse::parse_mouse_action(action)
            .ok_or_else(|| AdbError::new(crate::mouse::invalid_mouse_action(action)))?;
        self.run(&mouse_args(&self.host, parsed, x, y)).map(|_| ())
    }

    fn screencap_png(&self) -> Result<Vec<u8>, AdbError> {
        let raw = run::run_limited(
            &self.adb_bin,
            &screencap_args(&self.host),
            Some(SCREENCAP_TIMEOUT),
        )?;
        raw::to_png(&raw::parse(&raw)?)
    }

    fn getprop(&self, name: &str) -> Result<String, AdbError> {
        let out = self.run(&getprop_args(&self.host, name))?;
        Ok(String::from_utf8_lossy(&out).into_owned())
    }

    fn setprop(&self, name: &str, value: &str) -> Result<(), AdbError> {
        self.run(&setprop_args(&self.host, name, value)).map(|_| ())
    }

    fn testgamma_brightness(&self, value: i32) -> Result<(), AdbError> {
        self.run(&testgamma_args(&self.host, value)).map(|_| ())
    }

    fn window_dump(&self) -> Result<String, AdbError> {
        let out = self.run(&window_dump_args(&self.host))?;
        Ok(String::from_utf8_lossy(&out).into_owned())
    }

    fn lamp_read(&self) -> Result<String, AdbError> {
        let out = self.run(&lamp_read_args(&self.host))?;
        Ok(String::from_utf8_lossy(&out).into_owned())
    }

    fn lamp_write(&self, on: bool) -> Result<(), AdbError> {
        self.run(&lamp_write_args(&self.host, on)).map(|_| ())
    }
}

#[derive(Default)]
pub struct FakeAdb {
    pub host: String,
    pub connected: bool,
    pub keys: RefCell<Vec<i32>>,
    pub taps: RefCell<Vec<(i32, i32)>>,
    pub swipes: RefCell<Vec<(i32, i32, i32, i32, u32)>>,
    pub mouse: RefCell<Vec<(String, i32, i32)>>,
    pub props: RefCell<HashMap<String, String>>,
    pub testgamma: RefCell<Vec<i32>>,
    pub window: RefCell<String>,
    pub lamp: RefCell<i32>,
    pub png: Vec<u8>,
    pub fail: Option<String>,
}

impl AdbDevice for FakeAdb {
    fn connect(&mut self) -> Result<(), AdbError> {
        if let Some(msg) = &self.fail {
            return Err(AdbError::new(msg.clone()));
        }
        self.connected = true;
        Ok(())
    }

    fn keyevent(&self, code: i32) -> Result<(), AdbError> {
        if let Some(msg) = &self.fail {
            return Err(AdbError::new(msg.clone()));
        }
        self.keys.borrow_mut().push(code);
        Ok(())
    }

    fn tap(&self, x: i32, y: i32) -> Result<(), AdbError> {
        if let Some(msg) = &self.fail {
            return Err(AdbError::new(msg.clone()));
        }
        self.taps.borrow_mut().push((x, y));
        Ok(())
    }

    fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<(), AdbError> {
        if let Some(msg) = &self.fail {
            return Err(AdbError::new(msg.clone()));
        }
        self.swipes.borrow_mut().push((x1, y1, x2, y2, duration_ms));
        Ok(())
    }

    fn mouse(&self, action: &str, x: i32, y: i32) -> Result<(), AdbError> {
        if let Some(msg) = &self.fail {
            return Err(AdbError::new(msg.clone()));
        }
        let parsed = crate::mouse::parse_mouse_action(action)
            .ok_or_else(|| AdbError::new(crate::mouse::invalid_mouse_action(action)))?;
        self.mouse.borrow_mut().push((parsed.to_string(), x, y));
        Ok(())
    }

    fn screencap_png(&self) -> Result<Vec<u8>, AdbError> {
        if let Some(msg) = &self.fail {
            return Err(AdbError::new(msg.clone()));
        }
        Ok(self.png.clone())
    }

    fn getprop(&self, name: &str) -> Result<String, AdbError> {
        if let Some(msg) = &self.fail {
            return Err(AdbError::new(msg.clone()));
        }
        Ok(self.props.borrow().get(name).cloned().unwrap_or_default())
    }

    fn setprop(&self, name: &str, value: &str) -> Result<(), AdbError> {
        if let Some(msg) = &self.fail {
            return Err(AdbError::new(msg.clone()));
        }
        self.props
            .borrow_mut()
            .insert(name.to_string(), value.to_string());
        Ok(())
    }

    fn testgamma_brightness(&self, value: i32) -> Result<(), AdbError> {
        if let Some(msg) = &self.fail {
            return Err(AdbError::new(msg.clone()));
        }
        self.testgamma.borrow_mut().push(value);
        Ok(())
    }

    fn window_dump(&self) -> Result<String, AdbError> {
        if let Some(msg) = &self.fail {
            return Err(AdbError::new(msg.clone()));
        }
        Ok(self.window.borrow().clone())
    }

    fn lamp_read(&self) -> Result<String, AdbError> {
        if let Some(msg) = &self.fail {
            return Err(AdbError::new(msg.clone()));
        }
        Ok(self.lamp.borrow().to_string())
    }

    fn lamp_write(&self, on: bool) -> Result<(), AdbError> {
        if let Some(msg) = &self.fail {
            return Err(AdbError::new(msg.clone()));
        }
        *self.lamp.borrow_mut() = crate::lamp::write_value(on);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{from_pad, keycode};
    use fake::Fake;
    use std::fs;

    #[test]
    fn argv_uses_serial_and_input() {
        let host: String = format!("192.168.15.{}:3268", (2u8..250u8).fake::<u8>());
        assert_eq!(
            connect_args(&host),
            vec!["connect".to_string(), host.clone()]
        );
        let code = keycode(from_pad("ok").unwrap());
        assert_eq!(
            keyevent_args(&host, code),
            vec![
                "-s".into(),
                host.clone(),
                "shell".into(),
                "input".into(),
                "keyevent".into(),
                code.to_string()
            ]
        );
        let x: i32 = (0i32..1280i32).fake();
        let y: i32 = (0i32..720i32).fake();
        let tap = tap_args(&host, x, y);
        assert_eq!(tap[4], "tap");
        assert_eq!(tap[5], x.to_string());
        assert_eq!(tap[6], y.to_string());
        let action = crate::mouse::parse_mouse_action("MOVE").unwrap();
        let mouse = mouse_args(&host, action, x, y);
        assert_eq!(mouse[4], "mouse");
        assert_eq!(mouse[5], "motionevent");
        assert_eq!(mouse[6], "MOVE");
        assert_eq!(mouse[7], x.to_string());
        assert_eq!(mouse[8], y.to_string());
        assert_eq!(
            swipe_args(&host, 640, 360, 640, 240, 200),
            vec![
                "-s".into(),
                host.clone(),
                "shell".into(),
                "input".into(),
                "swipe".into(),
                "640".into(),
                "360".into(),
                "640".into(),
                "240".into(),
                "200".into()
            ]
        );
        assert_eq!(
            screencap_args(&host),
            vec![
                "-s".into(),
                host.clone(),
                "exec-out".into(),
                "screencap".into()
            ]
        );
        assert!(
            !screencap_args(&host).iter().any(|a| a == "-p"),
            "PNG on-device (-p) estoura 4s no vídeo; raw RGBA encodes on the Mac"
        );
        assert_eq!(
            getprop_args(&host, crate::brightness::BRIGHTNESS_PROP),
            vec![
                "-s".into(),
                host.clone(),
                "shell".into(),
                "getprop".into(),
                crate::brightness::BRIGHTNESS_PROP.into(),
            ]
        );
        let level: i32 = (0i32..=100i32).fake();
        assert_eq!(
            setprop_args(
                &host,
                crate::brightness::BRIGHTNESS_PROP,
                &level.to_string()
            ),
            vec![
                "-s".into(),
                host.clone(),
                "shell".into(),
                "setprop".into(),
                crate::brightness::BRIGHTNESS_PROP.into(),
                level.to_string(),
            ]
        );
        let gamma = testgamma_args(&host, level);
        assert_eq!(gamma[0], "-s");
        assert_eq!(gamma[1], host);
        assert_eq!(gamma[2], "shell");
        assert_eq!(gamma[3], "testgamma");
        assert!(gamma
            .windows(2)
            .any(|w| w[0] == "--brightness" && w[1] == level.to_string()));
        assert!(gamma.windows(2).any(|w| w[0] == "--wb_g" && w[1] == "0.96"));
        assert_eq!(
            window_dump_args(&host),
            vec![
                "-s".into(),
                host.clone(),
                "shell".into(),
                "dumpsys".into(),
                "window".into(),
                "displays".into(),
            ]
        );
        assert_eq!(
            lamp_read_args(&host),
            vec![
                "-s".into(),
                host.clone(),
                "shell".into(),
                "cat".into(),
                crate::lamp::LAMP_POWER_PATH.into(),
            ]
        );
        let on = (0u8..2u8).fake::<u8>() == 1;
        let write = lamp_write_args(&host, on);
        assert_eq!(write[3], "sh");
        assert_eq!(write[4], "-c");
        let script = &write[5];
        assert!(script.contains(crate::lamp::LAMP_EN_PATH), "{script}");
        assert!(script.contains(crate::lamp::LAMP_POWER_PATH), "{script}");
        assert!(
            script.contains(&crate::lamp::write_value(on).to_string()),
            "{script}"
        );
        assert!(gamma
            .windows(2)
            .any(|w| w[0] == "--target_gamma" && w[1] == "1.8"));
    }

    fn fake_raw_frame(width: u32, height: u32, pixel: [u8; 4]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&width.to_le_bytes());
        buf.extend_from_slice(&height.to_le_bytes());
        buf.extend_from_slice(&1u32.to_le_bytes());
        buf.extend_from_slice(&1u32.to_le_bytes());
        for _ in 0..(width * height) {
            buf.extend_from_slice(&pixel);
        }
        buf
    }

    #[test]
    fn parse_raw_reads_android12_rgba_header() {
        let width: u32 = (1u32..8u32).fake();
        let height: u32 = (1u32..8u32).fake();
        let pixel = [0xff, 0x00, 0x40, 0xff];
        let buf = fake_raw_frame(width, height, pixel);
        let frame = raw::parse(&buf).unwrap();
        assert_eq!(frame.width, width);
        assert_eq!(frame.height, height);
        assert_eq!(frame.rgba.len(), (width * height * 4) as usize);
        assert_eq!(&frame.rgba[0..4], &pixel);
    }

    #[test]
    fn parse_raw_rejects_short_and_wrong_format() {
        let err = raw::parse(&[1, 2, 3]).unwrap_err();
        assert!(err.message.contains("3"), "{}", err.message);
        assert!(err.message.contains("16"), "{}", err.message);
        let mut bad = fake_raw_frame(1, 1, [1, 2, 3, 4]);
        bad[8..12].copy_from_slice(&5u32.to_le_bytes());
        let err = raw::parse(&bad).unwrap_err();
        assert!(err.message.contains("format 5"), "{}", err.message);
        assert!(err.message.contains("RGBA"), "{}", err.message);
    }

    #[test]
    fn raw_frame_encodes_png_ihdr() {
        let width: u32 = (1u32..6u32).fake();
        let height: u32 = (1u32..6u32).fake();
        let frame = raw::parse(&fake_raw_frame(width, height, [10, 20, 30, 255])).unwrap();
        let png = raw::to_png(&frame).unwrap();
        assert_eq!(
            crate::preview::png_size(&png),
            Some((width, height)),
            "png len {}",
            png.len()
        );
        assert_eq!(&png[0..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn fake_records_key_and_tap() {
        let fake = FakeAdb {
            png: vec![0x89, b'P', b'N', b'G'],
            ..FakeAdb::default()
        };
        fake.keyevent(23).unwrap();
        fake.tap(10, 20).unwrap();
        fake.swipe(640, 360, 640, 240, 200).unwrap();
        fake.mouse("DOWN", 10, 20).unwrap();
        assert_eq!(*fake.keys.borrow(), vec![23]);
        assert_eq!(*fake.taps.borrow(), vec![(10, 20)]);
        assert_eq!(*fake.swipes.borrow(), vec![(640, 360, 640, 240, 200)]);
        assert_eq!(*fake.mouse.borrow(), vec![("DOWN".into(), 10, 20)]);
        assert_eq!(fake.screencap_png().unwrap()[1], b'P');
    }

    #[test]
    fn fake_nudge_picture_brightness_sets_prop_and_testgamma() {
        let fake = FakeAdb::default();
        fake.setprop(crate::brightness::BRIGHTNESS_PROP, "40")
            .unwrap();
        let next = nudge_picture_brightness(&fake, 5).unwrap();
        assert_eq!(next, 45);
        assert_eq!(
            fake.getprop(crate::brightness::BRIGHTNESS_PROP).unwrap(),
            "45"
        );
        assert_eq!(*fake.testgamma.borrow(), vec![45]);
    }

    #[test]
    fn empty_prop_defaults_to_fifty_then_nudge() {
        let fake = FakeAdb::default();
        assert_eq!(picture_brightness(&fake).unwrap(), 50);
        let next = nudge_picture_brightness(&fake, -5).unwrap();
        assert_eq!(next, 45);
        assert_eq!(*fake.testgamma.borrow(), vec![45]);
    }

    #[test]
    fn fake_picture_power_follows_window_focus() {
        let fake = FakeAdb::default();
        assert_eq!(picture_power(&fake).unwrap(), "on");
        *fake.window.borrow_mut() =
            "  mCurrentFocus=Window{c85f1b1 u0 LegacyGlobalActions}\n".into();
        assert_eq!(picture_power(&fake).unwrap(), "off");
    }

    #[test]
    fn fake_set_picture_lamp_writes_gpio() {
        let fake = FakeAdb::default();
        assert_eq!(picture_lamp(&fake).unwrap(), false);
        assert_eq!(set_picture_lamp(&fake, true).unwrap(), true);
        assert_eq!(*fake.lamp.borrow(), 255);
        assert_eq!(picture_lamp(&fake).unwrap(), true);
        assert_eq!(set_picture_lamp(&fake, false).unwrap(), false);
        assert_eq!(*fake.lamp.borrow(), 0);
    }

    #[test]
    fn fake_failure_includes_message() {
        let mut fake = FakeAdb {
            fail: Some("adb: no devices/emulators found".into()),
            ..FakeAdb::default()
        };
        let err = fake.connect().unwrap_err();
        assert!(err.message.contains("no devices"), "{}", err.message);
        let err = fake.swipe(640, 360, 640, 240, 200).unwrap_err();
        assert!(err.message.contains("no devices"), "{}", err.message);
        assert!(fake.swipes.borrow().is_empty());
        let err = fake.mouse("MOVE", 1, 1).unwrap_err();
        assert!(err.message.contains("no devices"), "{}", err.message);
        assert!(fake.mouse.borrow().is_empty());
    }

    #[test]
    fn mouse_rejects_invalid_action_and_cites_value() {
        let fake = FakeAdb::default();
        let junk = format!("tap{}", (1u32..99u32).fake::<u32>());
        let err = fake.mouse(&junk, 1, 1).unwrap_err();
        assert!(err.message.contains(&junk), "{}", err.message);
        assert!(err.message.contains("MOVE|DOWN|UP"), "{}", err.message);
        assert!(fake.mouse.borrow().is_empty());
    }

    #[test]
    fn error_from_missing_binary_names_argv() {
        let adb = CliAdb {
            adb_bin: "/tmp/hy300-adb-nao-existe".into(),
            host: "192.168.15.4:3268".into(),
        };
        let err = adb.keyevent(23).unwrap_err();
        assert!(err.message.contains("keyevent"), "{}", err.message);
        assert!(err.message.contains("23"), "{}", err.message);
    }

    /// Write+chmod via a sibling then rename so Linux does not ETXTBSY on exec.
    fn write_fake_adb(dir: &std::path::Path, script: &str) -> std::path::PathBuf {
        fs::create_dir_all(dir).unwrap();
        let bin = dir.join("adb");
        let tmp = dir.join("adb.tmp");
        fs::write(&tmp, script).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perm = fs::metadata(&tmp).unwrap().permissions();
            perm.set_mode(0o755);
            fs::set_permissions(&tmp, perm).unwrap();
        }
        fs::rename(&tmp, &bin).unwrap();
        bin
    }

    #[test]
    fn screencap_png_encodes_raw_rgba_from_cli() {
        let dir = std::env::temp_dir().join(format!(
            "hy300-raw-adb-{}",
            (1u32..1_000_000u32).fake::<u32>()
        ));
        // 1x1 Android 12 header + opaque red RGBA. printf octal keeps the script ASCII.
        let bin = write_fake_adb(
            &dir,
            "#!/bin/sh\nprintf '\\1\\0\\0\\0\\1\\0\\0\\0\\1\\0\\0\\0\\1\\0\\0\\0\\377\\0\\0\\377'\n",
        );
        let adb = CliAdb {
            adb_bin: bin.to_string_lossy().into_owned(),
            host: "192.168.15.4:3268".into(),
        };
        let png = adb.screencap_png().unwrap();
        assert_eq!(&png[0..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(crate::preview::png_size(&png), Some((1, 1)));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn screencap_timeout_kills_slow_binary() {
        let dir = std::env::temp_dir().join(format!(
            "hy300-slow-adb-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let bin = write_fake_adb(&dir, "#!/bin/sh\nsleep 30\nprintf 'too-late'\n");
        let adb = CliAdb {
            adb_bin: bin.to_string_lossy().into_owned(),
            host: "192.168.15.4:3268".into(),
        };
        let started = std::time::Instant::now();
        let err = adb.screencap_png().unwrap_err();
        assert!(
            started.elapsed() < std::time::Duration::from_secs(8),
            "timeout should kill the 30s sleep"
        );
        assert!(err.message.contains("estourou"), "{}", err.message);
        assert!(err.message.contains("screencap"), "{}", err.message);
        let _ = fs::remove_dir_all(&dir);
    }
}
