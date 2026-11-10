//! ADB CLI. Testes cobrem argv; o binário não roda na CI.

mod run;

use std::cell::RefCell;
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
    fn screencap_png(&self) -> Result<Vec<u8>, AdbError>;
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

pub fn screencap_args(host: &str) -> Vec<String> {
    vec![
        "-s".into(),
        host.into(),
        "exec-out".into(),
        "screencap".into(),
        "-p".into(),
    ]
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

    fn screencap_png(&self) -> Result<Vec<u8>, AdbError> {
        run::run_limited(
            &self.adb_bin,
            &screencap_args(&self.host),
            Some(SCREENCAP_TIMEOUT),
        )
    }
}

#[derive(Default)]
pub struct FakeAdb {
    pub host: String,
    pub connected: bool,
    pub keys: RefCell<Vec<i32>>,
    pub taps: RefCell<Vec<(i32, i32)>>,
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

    fn screencap_png(&self) -> Result<Vec<u8>, AdbError> {
        if let Some(msg) = &self.fail {
            return Err(AdbError::new(msg.clone()));
        }
        Ok(self.png.clone())
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
        assert_eq!(screencap_args(&host)[3], "screencap");
        assert_eq!(screencap_args(&host)[4], "-p");
    }

    #[test]
    fn fake_records_key_and_tap() {
        let fake = FakeAdb {
            png: vec![0x89, b'P', b'N', b'G'],
            ..FakeAdb::default()
        };
        fake.keyevent(23).unwrap();
        fake.tap(10, 20).unwrap();
        assert_eq!(*fake.keys.borrow(), vec![23]);
        assert_eq!(*fake.taps.borrow(), vec![(10, 20)]);
        assert_eq!(fake.screencap_png().unwrap()[1], b'P');
    }

    #[test]
    fn fake_failure_includes_message() {
        let mut fake = FakeAdb {
            fail: Some("adb: no devices/emulators found".into()),
            ..FakeAdb::default()
        };
        let err = fake.connect().unwrap_err();
        assert!(err.message.contains("no devices"), "{}", err.message);
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

    #[test]
    fn screencap_timeout_kills_slow_binary() {
        let dir = std::env::temp_dir().join(format!(
            "hy300-slow-adb-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let bin = dir.join("adb");
        fs::write(&bin, "#!/bin/sh\nsleep 30\nprintf 'too-late'\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perm = fs::metadata(&bin).unwrap().permissions();
            perm.set_mode(0o755);
            fs::set_permissions(&bin, perm).unwrap();
        }
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
