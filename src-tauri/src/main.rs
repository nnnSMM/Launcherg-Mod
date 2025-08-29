// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod domain;
mod infrastructure;
mod interface;
mod usecase;

use std::sync::Arc;

use infrastructure::util::get_save_root_abs_dir_with_ptr_handle;
use interface::{
    command,
    module::{Modules, ModulesExt},
};
use tauri::{async_runtime::block_on, Emitter, Listener, Manager};
use tauri_plugin_log::{Target, TargetKind};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            // folder の中身を移動して folder を削除する
            // C:\Users\ryoha\AppData\Roaming\launcherg -> C:\Users\ryoha\AppData\Roaming\ryoha.moe\launcherg

            let dst_dir = get_save_root_abs_dir_with_ptr_handle(app.handle());
            let src_dir = std::path::Path::new(&dst_dir)
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("launcherg");
            println!("src_dir: {:?}, dst_dir: {:?}", src_dir, dst_dir);
            if src_dir.exists() {
                let dst_dir = std::path::Path::new(&dst_dir);
                std::fs::create_dir_all(&dst_dir).unwrap();
                for entry in std::fs::read_dir(&src_dir).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    let file_name = path.file_name().unwrap();
                    let dst_path = dst_dir.join(file_name);
                    println!("rename {:?} -> {:?}", path, dst_path);
                    std::fs::rename(path, dst_path).unwrap();
                }
                std::fs::remove_dir_all(src_dir).unwrap();
            }

            let modules = Arc::new(block_on(Modules::new(app.handle())));
            app.manage(modules);

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let modules = app_handle.state::<Arc<Modules>>();
                if let Ok(Some(shortcut_key)) = modules
                    .collection_use_case()
                    .get_app_setting("shortcut_key".to_string())
                    .await
                {
                    if !shortcut_key.is_empty() {
                        let manager = app_handle.global_shortcut_manager();
                        let handle_clone = app_handle.clone();
                        let _ = manager.register(&shortcut_key, move || {
                            let _ = handle_clone.emit("global-shortcut-launch-game", ());
                        });
                    }
                }
            });

            let app_handle = app.handle().clone();
            app_handle.listen("global-shortcut-launch-game", move |_| {
                println!("global-shortcut-launch-game event received");
                let handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    let modules = handle.state::<Arc<Modules>>();
                    if let Ok(Some(game_id_str)) = modules
                        .collection_use_case()
                        .get_app_setting("shortcut_game_id".to_string())
                        .await
                    {
                        if let Ok(game_id) = game_id_str.parse::<i32>() {
                            println!("Launching game with id: {}", game_id);
                            let _ = modules
                                .collection_use_case()
                                .play_game_and_track(handle.into(), game_id)
                                .await;
                        }
                    }
                });
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
