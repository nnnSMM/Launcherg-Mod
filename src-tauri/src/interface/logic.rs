use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter, Manager};

#[cfg(target_os = "windows")]
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{GetForegroundWindow, SetForegroundWindow},
};

use super::module::{Modules, ModulesExt};

static PAUSE_FOREGROUND_WINDOW: OnceLock<Mutex<Option<isize>>> = OnceLock::new();

fn foreground_window_slot() -> &'static Mutex<Option<isize>> {
    PAUSE_FOREGROUND_WINDOW.get_or_init(|| Mutex::new(None))
}

#[cfg(target_os = "windows")]
fn remember_foreground_window() {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0 == 0 {
        return;
    }
    if let Ok(mut slot) = foreground_window_slot().lock() {
        *slot = Some(hwnd.0);
    }
}

#[cfg(not(target_os = "windows"))]
fn remember_foreground_window() {}

#[cfg(target_os = "windows")]
fn restore_foreground_window() {
    let hwnd = foreground_window_slot()
        .lock()
        .ok()
        .and_then(|mut slot| slot.take());
    if let Some(hwnd) = hwnd {
        if hwnd != 0 {
            let _ = unsafe { SetForegroundWindow(HWND(hwnd)) };
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn restore_foreground_window() {}

pub fn toggle_pause_and_notify(app: &AppHandle, modules: &Modules) -> anyhow::Result<bool> {
    let is_paused = modules
        .pause_manager()
        .toggle()
        .map_err(|e| anyhow::anyhow!(e))?;

    if let Some(window) = app.get_webview_window("overlay") {
        if is_paused {
            remember_foreground_window();
            window.show()?;
            window.set_focus()?;
        } else {
            window.hide()?;
            restore_foreground_window();
        }
    }

    app.emit("pause-toggled", is_paused)?;
    Ok(is_paused)
}
