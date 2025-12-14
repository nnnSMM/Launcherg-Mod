use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::Shortcut;

use super::module::{Modules, ModulesExt};

pub async fn handle_shortcut(app_handle: AppHandle, shortcut: Shortcut) {
    let modules = app_handle.state::<Arc<Modules>>().inner().clone();

    // Launch shortcut handling
    if let Ok(Some(shortcut_key)) = modules
        .collection_use_case()
        .get_app_setting("shortcut_key".to_string())
        .await
    {
        if let Ok(shortcut_from_setting) = shortcut_key.parse::<Shortcut>() {
            if shortcut == shortcut_from_setting {
                if let Ok(Some(game_id_str)) = modules
                    .collection_use_case()
                    .get_app_setting("shortcut_game_id".to_string())
                    .await
                {
                    if let Ok(game_id) = game_id_str.parse::<i32>() {
                        if let Err(e) = modules
                            .collection_use_case()
                            .play_game_and_track(app_handle.clone().into(), game_id)
                            .await
                        {
                            eprintln!("Error playing game: {}", e);
                        }
                    }
                }
            }
        }
    }

    // Pause shortcut handling
    if let Ok(Some(pause_shortcut_key)) = modules
        .collection_use_case()
        .get_app_setting("pause_shortcut_key".to_string())
        .await
    {
        if let Ok(pause_shortcut) = pause_shortcut_key.parse::<Shortcut>() {
            if shortcut == pause_shortcut {
                // Check if tracking before allowing pause
                if let Ok(is_paused) = modules.pause_manager().toggle() {
                    if let Some(window) = app_handle.get_webview_window("overlay") {
                        if is_paused {
                            let _ = window.show();
                            let _ = window.set_focus();
                        } else {
                            let _ = window.hide();
                        }
                    }
                    let _ = app_handle.emit("pause-toggled", is_paused);
                }
            }
        }
    }

    // Scaling shortcut handling
    if let Ok(Some(scaling_shortcut_key)) = modules
        .collection_use_case()
        .get_app_setting("scaling_shortcut_key".to_string())
        .await
    {
        if let Ok(scaling_shortcut) = scaling_shortcut_key.parse::<Shortcut>() {
            if shortcut == scaling_shortcut {
                let shader_name = match modules
                    .collection_use_case()
                    .get_app_setting("scaling_shader".to_string())
                    .await
                {
                    Ok(Some(s)) if !s.is_empty() => s,
                    _ => "Bicubic".to_string(),
                };

                if let Err(e) = modules.scaling_use_case().toggle_scaling(shader_name) {
                    eprintln!("Error toggling scaling: {}", e);
                }
            }
        }
    }
}
