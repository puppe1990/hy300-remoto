#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Mutex;

use hy300_remoto::adb::{AdbDevice, CliAdb};
use hy300_remoto::host::{load_config, save_config, HostConfig};
use hy300_remoto::preview::{png_data_url, png_size};
use hy300_remoto::tap::HY300_SCREEN;
use serde::Serialize;
use tauri::{Manager, State};

struct Remote {
    host: Mutex<String>,
    config_dir: PathBuf,
}

impl Remote {
    fn current_host(&self) -> Result<String, String> {
        self.host
            .lock()
            .map(|h| h.clone())
            .map_err(|e| e.to_string())
    }

    fn set_current(&self, host: String) -> Result<(), String> {
        *self.host.lock().map_err(|e| e.to_string())? = host;
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
fn get_host(remote: State<Remote>) -> Result<HostConfig, String> {
    Ok(HostConfig {
        host: remote.current_host()?,
    })
}

#[tauri::command]
fn set_host(host: String, remote: State<Remote>) -> Result<HostConfig, String> {
    let cfg = save_config(&remote.config_dir, &host)?;
    remote.set_current(cfg.host.clone())?;
    Ok(cfg)
}

#[tauri::command]
async fn connect_host(remote: State<'_, Remote>) -> Result<(), String> {
    let mut adb = remote.adb()?;
    tauri::async_runtime::spawn_blocking(move || adb.connect().map_err(|e| e.message))
        .await
        .map_err(|e| format!("adb: {e}"))?
}

#[tauri::command]
async fn send_key(code: i32, remote: State<'_, Remote>) -> Result<(), String> {
    let adb = remote.adb()?;
    tauri::async_runtime::spawn_blocking(move || adb.keyevent(code).map_err(|e| e.message))
        .await
        .map_err(|e| format!("adb: {e}"))?
}

#[tauri::command]
async fn send_tap(x: i32, y: i32, remote: State<'_, Remote>) -> Result<(), String> {
    let adb = remote.adb()?;
    tauri::async_runtime::spawn_blocking(move || adb.tap(x, y).map_err(|e| e.message))
        .await
        .map_err(|e| format!("adb: {e}"))?
}

#[tauri::command]
async fn preview_frame(remote: State<'_, Remote>) -> Result<PreviewFrame, String> {
    let adb = remote.adb()?;
    tauri::async_runtime::spawn_blocking(move || {
        let png = adb.screencap_png().map_err(|e| e.message)?;
        let (width, height) = png_size(&png).unwrap_or((HY300_SCREEN.width, HY300_SCREEN.height));
        Ok(PreviewFrame {
            data_url: png_data_url(&png),
            width,
            height,
        })
    })
    .await
    .map_err(|e| format!("preview: {e}"))?
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app
                .path()
                .app_config_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            let cfg = load_config(&dir);
            app.manage(Remote {
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
