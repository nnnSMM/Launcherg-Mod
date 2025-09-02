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
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Listener, Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_log::{Target, TargetKind};

fn main() {
    tauri::Builder::default()
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
                    });
                })
                .build(),
        )
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .setup(|app| {
            let handle = app.handle().clone();
            if let Err(e) = handle.global_shortcut().unregister_all() {
                eprintln!("Failed to unregister all shortcuts on startup: {}", e);
            }

            let show_hide_i =
                MenuItem::with_id(app, "show_hide", "Show/Hide", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_hide_i, &quit_i])?;
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show_hide" => {
                        let window = app.get_webview_window("main").unwrap();
                        if window.is_visible().unwrap() {
                            window.hide().unwrap();
                        } else {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            let app_handle = app.handle().clone();
            app.listen("single-instance", move |_event| {
                if let Some(window) = app_handle.get_webview_window("main") {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            });

            let modules = Arc::new(tauri::async_runtime::block_on(Modules::new(
                &app.handle(),
            )));
            app.manage(modules);

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
            command::update_shortcut_registration
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
