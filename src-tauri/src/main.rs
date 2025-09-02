// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod domain;
mod infrastructure;
mod interface;
mod usecase;

use std::sync::Arc;

use interface::{
    command,
    module::{Modules, ModulesExt},
};
use tauri::{
    tray::{TrayIconEvent, TrayIconBuilder},
    AppHandle, Emitter, Listener, Manager, PhysicalPosition, Wry,
};
use tauri_plugin_autostart::{ManagerExt, MacosLauncher};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_log::{Target, TargetKind};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            app.emit("single-instance", ()).unwrap();
        }))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let app_handle = app.clone();
                    let shortcut = shortcut.clone();
                    tauri::async_runtime::spawn(async move {
                        let modules = app_handle.state::<Arc<Modules>>();
                        if let Ok(Some(shortcut_key)) = modules
                            .collection_use_case()
                            .get_app_setting("shortcut_key".to_string())
                            .await
                        {
                            if let Ok(shortcut_from_setting) = shortcut_key.parse::<Shortcut>() {
                                if shortcut == shortcut_from_setting {
                                    // launch_shortcut_game コマンドのロジックをここに直接展開
                                    if let Ok(Some(game_id_str)) = modules
                                        .collection_use_case()
                                        .get_app_setting("shortcut_game_id".to_string())
                                        .await
                                    {
                                        if let Ok(game_id) = game_id_str.parse::<i32>() {
                                            if let Err(e) = modules
                                                .collection_use_case()
                                                .play_game_and_track(
                                                    app_handle.clone().into(),
                                                    game_id,
                                                )
                                                .await
                                            {
                                                eprintln!("Error playing game: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
                })
                .build(),
        )
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_window_state::Builder::default().with_label_ignores(&["tray"]).build())
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                if window.label() == "main" {
                    window.hide().unwrap();
                    api.prevent_close();
                }
            }
            tauri::WindowEvent::Focused(false) => {
                if window.label() == "tray" {
                    window.hide().unwrap();
                }
            }
            _ => {}
        })
        .setup(|app| {
            #[cfg(desktop)]
            {
                let autostart_manager = app.autolaunch();
                if !autostart_manager.is_enabled().unwrap() {
                    let _ = autostart_manager.enable();
                }
            }

            let handle = app.handle().clone();
            if let Err(e) = handle.global_shortcut().unregister_all() {
                eprintln!("Failed to unregister all shortcuts on startup: {}", e);
            }

            // Modulesの初期化を先に行う
            let modules = Arc::new(tauri::async_runtime::block_on(Modules::new(
                &app.handle(),
            )));
            app.manage(modules);

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Launcherg")
                .on_tray_icon_event(|tray, event| {
                    match event {
                        TrayIconEvent::Click { button, position, .. } => {
                            let app = tray.app_handle();
                            if button == tauri::tray::MouseButton::Right {
                                if let Some(window) = app.get_webview_window("tray") {
                                    if window.is_visible().unwrap() {
                                        window.hide().unwrap();
                                    } else {
                                        let scale_factor = window.scale_factor().unwrap_or(1.0);
                                        let window_size = window.outer_size().unwrap().to_physical::<u32>(scale_factor);
                                        let physical_pos = PhysicalPosition {
                                            x: position.x as i32 - window_size.width as i32,
                                            y: position.y as i32 - window_size.height as i32,
                                        };
                                        window.set_position(physical_pos).unwrap();
                                        window.show().unwrap();
                                    }
                                }
                            } else if button == tauri::tray::MouseButton::Left {
                                if let Some(window) = app.get_webview_window("main") {
                                    if !window.is_visible().unwrap() {
                                        window.show().unwrap();
                                        window.set_focus().unwrap();
                                    }
                                }
                            }
                        }
                    _ => {}
                }
                })
                .build(app)?;

            let app_handle = app.handle().clone();
            app.listen("single-instance", move |_event| {
                if let Some(window) = app_handle.get_webview_window("main") {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            });

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let modules = handle.state::<Arc<Modules>>();
                if let Ok(Some(shortcut_key)) = modules
                    .collection_use_case()
                    .get_app_setting("shortcut_key".to_string())
                    .await
                {
                    if !shortcut_key.is_empty() {
                        if let Ok(shortcut) = shortcut_key.parse::<Shortcut>() {
                            if !handle.global_shortcut().is_registered(shortcut.clone()) {
                                if let Err(e) = handle.global_shortcut().register(shortcut) {
                                    eprintln!("Failed to register shortcut on startup: {}", e);
                                }
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            command::create_elements_in_pc,
            command::get_nearest_key_and_distance,
            command::upload_image,
            command::upsert_collection_element,
            command::update_collection_element_icon,
            command::get_default_import_dirs,
            command::play_game,
            command::get_play_time_minutes,
            command::get_collection_element,
            command::delete_collection_element,
            command::get_not_registered_detail_element_ids,
            command::create_element_details,
            command::get_all_elements,
            command::update_element_like,
            command::update_element_play_status, // 追加
            command::open_folder,
            command::get_all_game_cache_last_updated,
            command::update_all_game_cache,
            command::get_game_candidates,
            command::get_exe_path_by_lnk,
            command::get_game_cache_by_id,
            command::save_screenshot_by_pid,
            command::update_collection_element_thumbnails,
            command::update_game_image,
            command::get_app_setting,
            command::set_app_setting,
            command::launch_shortcut_game,
            command::update_shortcut_registration,
            quit_app,
            hide_tray_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn quit_app(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
async fn hide_tray_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("tray") {
        window.hide().unwrap();
    }
    Ok(())
}
