#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Mutex;

use hy300_remoto::adb::{AdbDevice, CliAdb};
use hy300_remoto::host::{load_config, save_config, HostConfig};
use hy300_remoto::preview::{png_data_url, png_size};
use hy300_remoto::tap::HY300_SCREEN;
use serde::Serialize;
use tauri::{Manager, State};

struct AdbSession {
    host: Mutex<String>,
    config_dir: PathBuf,
}

impl AdbSession {
    fn current_host(&self) -> Result<String, String> {
        self.host.lock().map(|h| h.clone()).map_err(|_| {
            "host mutex poisoned; expected unpoisoned Mutex<String> with host:porta".into()
        })
    }

    fn set_current(&self, host: String) -> Result<(), String> {
        *self.host.lock().map_err(|_| {
            "host mutex poisoned; expected unpoisoned Mutex<String> with host:porta".into()
        })? = host;
        Ok(())
    }

    fn adb(&self) -> Result<CliAdb, String> {
        Ok(CliAdb::new(self.current_host()?))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PreviewFrame {
    data_url: String,
    width: u32,
    height: u32,
}

#[tauri::command]
fn get_host(session: State<AdbSession>) -> Result<HostConfig, String> {
    Ok(HostConfig {
        host: session.current_host()?,
    })
}

#[tauri::command]
fn set_host(host: String, session: State<AdbSession>) -> Result<HostConfig, String> {
    let cfg = save_config(&session.config_dir, &host)?;
    session.set_current(cfg.host.clone())?;
    Ok(cfg)
}

/// ADB CLI is sync and screencap of a video frame is ~3.5s; run it off the UI thread.
async fn adb_blocking<T, F>(job: F, join_label: &'static str) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(job)
        .await
        .map_err(|e| format!("{join_label}: {e}"))?
}

#[tauri::command]
async fn connect_host(session: State<'_, AdbSession>) -> Result<(), String> {
    let mut adb = session.adb()?;
    adb_blocking(move || adb.connect().map_err(|e| e.message), "adb").await
}

#[tauri::command]
async fn send_key(code: i32, session: State<'_, AdbSession>) -> Result<(), String> {
    let adb = session.adb()?;
    adb_blocking(move || adb.keyevent(code).map_err(|e| e.message), "adb").await
}

#[tauri::command]
async fn send_tap(x: i32, y: i32, session: State<'_, AdbSession>) -> Result<(), String> {
    let adb = session.adb()?;
    adb_blocking(move || adb.tap(x, y).map_err(|e| e.message), "adb").await
}

#[tauri::command]
async fn preview_frame(session: State<'_, AdbSession>) -> Result<PreviewFrame, String> {
    let adb = session.adb()?;
    adb_blocking(
        move || {
            let png = adb.screencap_png().map_err(|e| e.message)?;
            let (width, height) =
                png_size(&png).unwrap_or((HY300_SCREEN.width, HY300_SCREEN.height));
            Ok(PreviewFrame {
                data_url: png_data_url(&png),
                width,
                height,
            })
        },
        "preview",
    )
    .await
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app
                .path()
                .app_config_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            let cfg = load_config(&dir);
            app.manage(AdbSession {
                host: Mutex::new(cfg.host),
                config_dir: dir,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_host,
            set_host,
            connect_host,
            send_key,
            send_tap,
            preview_frame
        ])
        .run(tauri::generate_context!())
        .expect("tauri");
}
