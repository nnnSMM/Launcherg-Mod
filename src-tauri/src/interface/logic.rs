use tauri::{AppHandle, Emitter, Manager};

use super::module::{Modules, ModulesExt};

pub fn toggle_pause_and_notify(app: &AppHandle, modules: &Modules) -> anyhow::Result<bool> {
    let is_paused = modules
        .pause_manager()
        .toggle()
        .map_err(|e| anyhow::anyhow!(e))?;

    if let Some(window) = app.get_webview_window("overlay") {
        if is_paused {
            window.show()?;
            window.set_focus()?;
        } else {
            window.hide()?;
        }
    }

    app.emit("pause-toggled", is_paused)?;
    Ok(is_paused)
}
