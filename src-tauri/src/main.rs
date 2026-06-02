// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod domain;
mod infrastructure;
mod interface;
mod usecase;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use interface::{
    command,
    module::{Modules, ModulesExt},
};
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Listener, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder,
};
#[cfg(all(desktop, not(debug_assertions)))]
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

const TRAY_MENU_WIDTH: f64 = 312.0;
const TRAY_MENU_HEIGHT: f64 = 448.0;
const APP_AUTOSTART_NAME: &str = "Launcherg";
const LEFT_CLICK_TRAY_MENU_DELAY: Duration = Duration::from_millis(280);
/// If no trailing `Click` arrives after `DoubleClick`, clear suppress so a later single-click still works.
const SUPPRESS_LEFT_MENU_RESET_AFTER_DOUBLE_CLICK: Duration = Duration::from_millis(400);

/// Left single-click opens the tray menu after a delay; double-click bumps this to cancel it.
struct TrayLeftClickMenuToken {
    generation: AtomicU64,
    /// After double-click, Windows may emit a trailing left `Click` (Up) that would open the menu;
    /// consume it here instead of scheduling.
    suppress_next_left_menu_open: AtomicBool,
}

impl TrayLeftClickMenuToken {
    fn new() -> Self {
        Self {
            generation: AtomicU64::new(0),
            suppress_next_left_menu_open: AtomicBool::new(false),
        }
    }

    fn begin_left_click_schedule(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn invalidate_scheduled_menu(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
    }

    fn generation_matches(&self, expected: u64) -> bool {
        self.generation.load(Ordering::SeqCst) == expected
    }

    fn arm_suppress_next_left_menu_open(&self) {
        self.suppress_next_left_menu_open
            .store(true, Ordering::SeqCst);
    }

    fn take_suppress_next_left_menu_open(&self) -> bool {
        self.suppress_next_left_menu_open
            .swap(false, Ordering::SeqCst)
    }

    fn clear_suppress_next_left_menu_open(&self) {
        self.suppress_next_left_menu_open
            .store(false, Ordering::SeqCst);
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .app_name(APP_AUTOSTART_NAME)
                .arg("--autostart")
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Err(e) = app.emit("single-instance", ()) {
                eprintln!("Failed to emit single-instance event: {}", e);
            }
        }))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let app_handle = app.clone();
                    let shortcut = *shortcut;
                    tauri::async_runtime::spawn(async move {
                        interface::shortcut::handle_shortcut(app_handle, shortcut).await;
                    });
                })
                .build(),
        )
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(window_state_flags())
                .build(),
        )
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                // For screenshot_window, destroy it
                // For other windows (main, overlay, tray_menu), just hide them
                if window.label() == "screenshot_window" {
                    println!("[main] screenshot_window close requested, destroying");
                    let _ = window.destroy();
                } else {
                    if window.label() == "main" {
                        save_current_window_state(window.app_handle());
                    }
                    // Hide other windows instead of closing
                    if let Err(e) = window.hide() {
                        eprintln!("Failed to hide window {}: {}", window.label(), e);
                    }
                    api.prevent_close();
                }
            }
            tauri::WindowEvent::Focused(false) if window.label() == "tray_menu" => {
                let _ = window.hide();
            }
            _ => {}
        })
        .setup(|app| -> std::result::Result<(), Box<dyn std::error::Error>> {
            // デバッグビルドでは current_exe() が target/debug を返すため、
            // autostart 登録を実行するとリリース版のRunエントリを開発用パスで上書きしてしまう。
            // リリースビルドのみで登録・補修を行う。
            #[cfg(all(desktop, not(debug_assertions)))]
            {
                let autostart_manager = app.autolaunch();
                // 常に enable() を呼び、インストーラーが登録した引数なしエントリを
                // --autostart 付きのエントリで上書きする（次回起動以降に有効）
                if let Err(e) = autostart_manager.enable() {
                    eprintln!("Failed to enable autostart: {}", e);
                }
                if let Err(e) = ensure_windows_autostart_entry(app) {
                    eprintln!("Failed to repair Windows autostart entry: {}", e);
                }
            }

            let handle = app.handle().clone();
            if let Err(e) = handle.global_shortcut().unregister_all() {
                eprintln!("Failed to unregister all shortcuts on startup: {}", e);
            }

            // Ensure overlay is hidden on startup
            if let Some(window) = handle.get_webview_window("overlay") {
                let _ = window.hide();
            }

            // Hide main window decorations (official title bar) here instead of tauri.conf.json
            // to fix window size saving issues on Windows.
            if let Some(window) = handle.get_webview_window("main") {
                let _ = window.set_decorations(false);
            }

            // Modulesの初期化を先に行う
            let modules = Arc::new(tauri::async_runtime::block_on(Modules::new(app.handle()))?);
            app.manage(modules);

            app.manage(TrayLeftClickMenuToken::new());

            create_tray_menu_window(app.handle())?;

            let mut tray_builder = TrayIconBuilder::with_id("main-tray")
                .tooltip("Launcherg")
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    let app = tray.app_handle();
                    match event {
                        TrayIconEvent::DoubleClick { button, .. } => {
                            if matches!(button, MouseButton::Left) {
                                let token_state = app.state::<TrayLeftClickMenuToken>();
                                token_state.invalidate_scheduled_menu();
                                token_state.arm_suppress_next_left_menu_open();
                                    if let Err(e) = show_main_window(app) {
                                    eprintln!(
                                        "Failed to show main window from tray double-click: {}",
                                        e
                                    );
                                }
                                let app_reset = app.clone();
                                tauri::async_runtime::spawn(async move {
                                    tokio::time::sleep(SUPPRESS_LEFT_MENU_RESET_AFTER_DOUBLE_CLICK)
                                        .await;
                                    app_reset
                                        .state::<TrayLeftClickMenuToken>()
                                        .clear_suppress_next_left_menu_open();
                                });
                            }
                        }
                        TrayIconEvent::Click {
                            position,
                            button,
                            button_state,
                            ..
                        } => {
                            if button_state != MouseButtonState::Up {
                                return;
                            }
                            match button {
                                MouseButton::Right => {
                                    if let Err(e) = toggle_tray_menu_window(app, position) {
                                        eprintln!("Failed to toggle tray menu: {}", e);
                                    }
                                }
                                MouseButton::Left => {
                                    let token_state = app.state::<TrayLeftClickMenuToken>();
                                    if token_state.take_suppress_next_left_menu_open() {
                                        return;
                                    }
                                    let generation = token_state.begin_left_click_schedule();
                                    let app_clone = app.clone();
                                    tauri::async_runtime::spawn(async move {
                                        tokio::time::sleep(LEFT_CLICK_TRAY_MENU_DELAY).await;
                                        let token_state =
                                            app_clone.state::<TrayLeftClickMenuToken>();
                                        if !token_state.generation_matches(generation) {
                                            return;
                                        }
                                        if let Err(e) =
                                            toggle_tray_menu_window(&app_clone, position)
                                        {
                                            eprintln!("Failed to toggle tray menu: {}", e);
                                        }
                                    });
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            } else {
                eprintln!("Default tray icon is not available; building tray without an icon.");
            }
            let _tray = tray_builder.build(app)?;

            let app_handle = app.handle().clone();
            app.listen("single-instance", move |_event| {
                if let Err(e) = show_main_window(&app_handle) {
                    eprintln!("Failed to show main window for single instance: {}", e);
                }
            });

            // スタートアップ起動（PC起動時の自動起動）の場合はウィンドウを表示しない
            let is_autostart = std::env::args().any(|arg| arg == "--autostart");
            if !is_autostart {
                if let Err(e) = show_main_window(app.handle()) {
                    eprintln!("Failed to show main window on startup: {}", e);
                }
            }

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let modules = handle.state::<Arc<Modules>>();

                // Register launch shortcut
                if let Ok(Some(shortcut_key)) = modules
                    .collection_use_case()
                    .get_app_setting("shortcut_key".to_string())
                    .await
                {
                    if !shortcut_key.is_empty() {
                        if let Ok(shortcut) = shortcut_key.parse::<Shortcut>() {
                            if !handle.global_shortcut().is_registered(shortcut) {
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
            command::adjust_untracked_play_time_seconds,
            command::get_collection_element_daily_play_times,
            command::open_folder,
            command::get_all_game_cache_last_updated,
            command::update_all_game_cache,
            command::get_game_candidates,
            command::search_all_game_cache,
            command::get_exe_path_by_lnk,
            command::get_game_cache_by_id,
            command::save_screenshot_by_pid,
            command::update_game_image,
            command::get_app_setting,
            command::set_app_setting,
            command::get_game_screenshot_cache,
            command::upsert_game_screenshot_cache,
            command::launch_shortcut_game,
            command::update_shortcut_registration,
            command::update_pause_shortcut_registration,
            command::toggle_pause_tracking,
            command::get_pause_state,
            command::get_game_screenshots,
            command::get_all_screenshots,
            command::open_screenshot_window,
            command::import_screenshot,
            command::delete_screenshot,
            command::update_screenshots_order,
            command::update_collection_element_path,
            command::delete_collection_element_logical,
            command::show_main_window,
            command::save_main_window_state,
            command::hide_tray_menu,
            command::quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_main_window(app_handle: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.unminimize();
        window.show()?;
        window.set_focus()?;
    }

    if let Some(window) = app_handle.get_webview_window("tray_menu") {
        let _ = window.hide();
    }

    Ok(())
}

fn window_state_flags() -> StateFlags {
    StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED | StateFlags::FULLSCREEN
}

fn save_current_window_state(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        if window.is_minimized().unwrap_or(false) {
            let _ = window.unminimize();
        }
    }

    let _ = app_handle.save_window_state(window_state_flags());
}

#[cfg(all(windows, desktop, not(debug_assertions)))]
fn ensure_windows_autostart_entry<R: tauri::Runtime>(app: &tauri::App<R>) -> anyhow::Result<()> {
    use winreg::enums::{RegType::REG_BINARY, HKEY_CURRENT_USER, KEY_SET_VALUE};
    use winreg::{RegKey, RegValue};

    const RUN_KEY: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run";
    const STARTUP_APPROVED_RUN_KEY: &str =
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
    const STARTUP_APPROVED_ENABLED: [u8; 12] = [
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let exe_path = std::env::current_exe()?;
    let command = format!("\"{}\" --autostart", exe_path.display());
    let legacy_app_name = app.package_info().name.as_str();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let run_key = hkcu.open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE)?;
    run_key.set_value(APP_AUTOSTART_NAME, &command)?;
    if !legacy_app_name.eq_ignore_ascii_case(APP_AUTOSTART_NAME) {
        let _ = run_key.delete_value(legacy_app_name);
    }

    if let Ok(startup_approved_key) =
        hkcu.open_subkey_with_flags(STARTUP_APPROVED_RUN_KEY, KEY_SET_VALUE)
    {
        startup_approved_key.set_raw_value(
            APP_AUTOSTART_NAME,
            &RegValue {
                vtype: REG_BINARY,
                bytes: STARTUP_APPROVED_ENABLED.to_vec(),
            },
        )?;
        if !legacy_app_name.eq_ignore_ascii_case(APP_AUTOSTART_NAME) {
            let _ = startup_approved_key.delete_value(legacy_app_name);
        }
    }

    Ok(())
}

#[cfg(all(not(windows), desktop, not(debug_assertions)))]
fn ensure_windows_autostart_entry<R: tauri::Runtime>(_app: &tauri::App<R>) -> anyhow::Result<()> {
    Ok(())
}

fn create_tray_menu_window(app_handle: &AppHandle) -> tauri::Result<()> {
    if app_handle.get_webview_window("tray_menu").is_some() {
        return Ok(());
    }

    WebviewWindowBuilder::new(
        app_handle,
        "tray_menu",
        WebviewUrl::App("index.html".into()),
    )
    .title("Launcherg Menu")
    .inner_size(TRAY_MENU_WIDTH, TRAY_MENU_HEIGHT)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(false)
    .build()?;

    Ok(())
}

fn toggle_tray_menu_window(
    app_handle: &AppHandle,
    position: PhysicalPosition<f64>,
) -> tauri::Result<()> {
    let Some(window) = app_handle.get_webview_window("tray_menu") else {
        return Ok(());
    };

    if window.is_visible()? {
        window.hide()?;
        return Ok(());
    }

    let (x, y) = tray_menu_position(app_handle, position)?;
    window.set_position(PhysicalPosition::new(x, y))?;
    window.show()?;
    window.set_focus()?;

    Ok(())
}

fn tray_menu_position(
    app_handle: &AppHandle,
    position: PhysicalPosition<f64>,
) -> tauri::Result<(i32, i32)> {
    let menu_width = TRAY_MENU_WIDTH as i32;
    let menu_height = TRAY_MENU_HEIGHT as i32;
    let px = position.x.round() as i32;
    let py = position.y.round() as i32;

    let monitor = app_handle
        .available_monitors()?
        .into_iter()
        .find(|monitor| {
            let origin = monitor.position();
            let size = monitor.size();
            px >= origin.x
                && px <= origin.x + size.width as i32
                && py >= origin.y
                && py <= origin.y + size.height as i32
        })
        .or(app_handle.primary_monitor()?);

    if let Some(monitor) = monitor {
        let origin = monitor.position();
        let size = monitor.size();
        let left = origin.x;
        let top = origin.y;
        let right = origin.x + size.width as i32;
        let bottom = origin.y + size.height as i32;

        let x = clamp_position(px - menu_width + 24, left + 8, right - menu_width - 8);
        let preferred_y = py - menu_height - 12;
        let fallback_y = py + 12;
        let y = if preferred_y < top + 8 {
            fallback_y
        } else {
            preferred_y
        };
        let y = clamp_position(y, top + 8, bottom - menu_height - 8);

        Ok((x, y))
    } else {
        Ok((px - menu_width + 24, py - menu_height - 12))
    }
}

fn clamp_position(value: i32, min: i32, max: i32) -> i32 {
    if min > max {
        min
    } else {
        value.clamp(min, max)
    }
}
