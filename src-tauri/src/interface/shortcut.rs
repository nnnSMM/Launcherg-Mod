use std::sync::Arc;
use tauri::{AppHandle, Manager};
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
                if let Err(e) = super::logic::toggle_pause_and_notify(&app_handle, &modules) {
                    eprintln!("Error toggling pause: {}", e);
                }
            }
        }
    }

    // Screenshot shortcut handling
    let mut screenshot_shortcut_str = "F1".to_string();
    if let Ok(Some(key)) = modules
        .collection_use_case()
        .get_app_setting("screenshot_shortcut_key".to_string())
        .await
    {
        if !key.is_empty() && key != "F12" {
            screenshot_shortcut_str = key;
        }
    }

    if let Ok(screenshot_shortcut) = screenshot_shortcut_str.parse::<Shortcut>() {
        if shortcut == screenshot_shortcut {
            if let Some(session) = modules.pause_manager().tracking_session() {
                let work_id = session.game_id;
                let app_handle_arc = Arc::new(app_handle.clone());

                match modules.file_use_case().get_new_upload_image_path(&app_handle_arc, work_id) {
                    Ok(upload_path) => {
                        if let Err(e) = modules.process_use_case().save_fullscreen_screenshot(&upload_path).await {
                            eprintln!("Error saving screenshot via shortcut: {}", e);
                        } else {
                            let path = std::path::Path::new(&upload_path);
                            let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            crate::interface::command::spawn_screenshot_notification(&app_handle, work_id, &filename, &upload_path);
                            if let Err(e) = modules.collection_use_case().register_screenshot_file(&app_handle_arc, work_id, upload_path.clone()).await {
                                eprintln!("Error registering screenshot via shortcut: {}", e);
                            }
                        }
                    }
                    Err(e) => eprintln!("Error getting upload path for screenshot via shortcut: {}", e),
                }
            }
        }
    }
}
