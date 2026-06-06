use super::{
    error::CommandError,
    models::{
        all_game_cache::AllGameCacheOne,
        collection::{CollectionElement, ProgressLivePayload, ProgressPayload},
    },
    module::{Modules, ModulesExt},
};
use crate::{
    domain::{
        collection::NewCollectionElement,
        distance::find_nearest,
        file::{
            get_exe_path_from_lnk, get_file_created_at_sync, get_icon_path, get_lnk_metadatas,
            get_thumbnail_candidate_urls, get_thumbnail_path, normalize,
        },
        repository::collection::{
            DailyPlayTime as DomainDailyPlayTime, GameScreenshotCache as DomainGameScreenshotCache,
        },
        Id,
    },
    usecase::error::UseCaseError,
    usecase::models::collection::CreateCollectionElementDetail,
};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Listener, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowScreenshot {
    pub id: i32,
    pub game_id: i32,
    pub filename: String,
    pub thumbnail_filename: Option<String>,
    pub order_index: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameScreenshotCache {
    pub collection_element_id: i32,
    pub matched_title: Option<String>,
    pub screenshots_json: String,
    pub fetched_at: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionElementDailyPlayTime {
    pub collection_element_id: i32,
    pub play_date: String,
    pub play_time_seconds: i32,
}

impl From<DomainDailyPlayTime> for CollectionElementDailyPlayTime {
    fn from(value: DomainDailyPlayTime) -> Self {
        Self {
            collection_element_id: value.collection_element_id,
            play_date: value.play_date,
            play_time_seconds: value.play_time_seconds,
        }
    }
}

impl From<DomainGameScreenshotCache> for GameScreenshotCache {
    fn from(value: DomainGameScreenshotCache) -> Self {
        Self {
            collection_element_id: value.collection_element_id,
            matched_title: value.matched_title,
            screenshots_json: value.screenshots_json,
            fetched_at: value.fetched_at,
            status: value.status,
        }
    }
}

impl From<GameScreenshotCache> for DomainGameScreenshotCache {
    fn from(value: GameScreenshotCache) -> Self {
        Self {
            collection_element_id: value.collection_element_id,
            matched_title: value.matched_title,
            screenshots_json: value.screenshots_json,
            fetched_at: value.fetched_at,
            status: value.status,
        }
    }
}

fn normalize_shortcut_key(shortcut_key: Option<String>) -> Option<String> {
    shortcut_key
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
}

fn parse_shortcut_key(shortcut_key: Option<&str>) -> anyhow::Result<Option<Shortcut>> {
    let Some(shortcut_key) = shortcut_key.map(str::trim).filter(|key| !key.is_empty()) else {
        return Ok(None);
    };

    shortcut_key
        .parse::<Shortcut>()
        .map(Some)
        .map_err(|e| anyhow::anyhow!("invalid shortcut key `{}`: {}", shortcut_key, e))
}

fn unregister_shortcut_if_needed(
    handle: &AppHandle,
    shortcut: Shortcut,
) -> Result<bool, CommandError> {
    let was_registered = handle.global_shortcut().is_registered(shortcut);
    if was_registered {
        handle
            .global_shortcut()
            .unregister(shortcut)
            .map_err(anyhow::Error::from)?;
    }

    Ok(was_registered)
}

fn register_new_shortcut(
    handle: &AppHandle,
    shortcut: Shortcut,
    shortcut_key: Option<&str>,
) -> Result<(), CommandError> {
    if handle.global_shortcut().is_registered(shortcut) {
        return Err(anyhow::anyhow!(
            "shortcut key `{}` is already registered",
            shortcut_key.unwrap_or("")
        )
        .into());
    }

    handle
        .global_shortcut()
        .register(shortcut)
        .map_err(anyhow::Error::from)?;

    Ok(())
}

fn ensure_shortcut_not_reserved(
    new_shortcut: Option<Shortcut>,
    reserved_shortcut: Option<Shortcut>,
    shortcut_key: Option<&str>,
    reserved_name: &str,
) -> Result<(), CommandError> {
    if new_shortcut.is_some() && new_shortcut == reserved_shortcut {
        return Err(anyhow::anyhow!(
            "shortcut key `{}` is already registered by {}",
            shortcut_key.unwrap_or(""),
            reserved_name
        )
        .into());
    }

    Ok(())
}

fn ensure_pause_shortcut_change_allowed(
    is_tracking: bool,
    new_shortcut: Option<Shortcut>,
    old_shortcut: Option<Shortcut>,
) -> Result<(), CommandError> {
    if is_tracking && new_shortcut != old_shortcut {
        return Err(
            anyhow::anyhow!("pause shortcut cannot be changed while tracking a game").into(),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_shortcut_key_trims_and_drops_empty_values() {
        assert_eq!(
            normalize_shortcut_key(Some(" Ctrl+Shift+L ".to_string())),
            Some("Ctrl+Shift+L".to_string())
        );
        assert_eq!(normalize_shortcut_key(Some("   ".to_string())), None);
    }

    #[test]
    fn ensure_shortcut_not_reserved_rejects_cross_setting_conflicts() {
        let shortcut = parse_shortcut_key(Some("Ctrl+Shift+L")).unwrap();

        let error = ensure_shortcut_not_reserved(
            shortcut,
            shortcut,
            Some("Ctrl+Shift+L"),
            "pause shortcut",
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("already registered by pause shortcut"));
    }

    #[test]
    fn ensure_shortcut_not_reserved_allows_empty_or_distinct_shortcuts() {
        let launch_shortcut = parse_shortcut_key(Some("Ctrl+Shift+L")).unwrap();
        let pause_shortcut = parse_shortcut_key(Some("Ctrl+Shift+P")).unwrap();

        assert!(
            ensure_shortcut_not_reserved(None, launch_shortcut, None, "pause shortcut").is_ok()
        );
        assert!(ensure_shortcut_not_reserved(
            launch_shortcut,
            pause_shortcut,
            Some("Ctrl+Shift+L"),
            "pause shortcut"
        )
        .is_ok());
    }

    #[test]
    fn ensure_pause_shortcut_change_allowed_rejects_changes_while_tracking() {
        let old_shortcut = parse_shortcut_key(Some("Ctrl+Shift+P")).unwrap();
        let new_shortcut = parse_shortcut_key(Some("Ctrl+Alt+P")).unwrap();

        let error =
            ensure_pause_shortcut_change_allowed(true, new_shortcut, old_shortcut).unwrap_err();

        assert!(error
            .to_string()
            .contains("cannot be changed while tracking"));
    }

    #[test]
    fn ensure_pause_shortcut_change_allowed_allows_noop_while_tracking() {
        let shortcut = parse_shortcut_key(Some("Ctrl+Shift+P")).unwrap();

        assert!(ensure_pause_shortcut_change_allowed(true, shortcut, shortcut).is_ok());
    }
}

#[tauri::command]
pub async fn open_screenshot_window(
    handle: AppHandle,
    game_id: Option<i32>,
    initial_screenshot_id: Option<i32>,
    initial_screenshot: Option<WindowScreenshot>,
) -> Result<(), CommandError> {
    println!(
        "[open_screenshot_window] Called with game_id={:?}, initial_screenshot_id={:?}, has_initial_screenshot={}",
        game_id,
        initial_screenshot_id,
        initial_screenshot.is_some()
    );

    #[derive(serde::Serialize, Clone)]
    struct WindowArgs {
        game_id: Option<i32>,
        initial_screenshot_id: Option<i32>,
        initial_screenshot: Option<WindowScreenshot>,
    }
    let args = WindowArgs {
        game_id,
        initial_screenshot_id,
        initial_screenshot,
    };

    if let Some(window) = handle.get_webview_window("screenshot_window") {
        println!("[open_screenshot_window] Window exists, applying args before showing");
        let window_for_ready = window.clone();
        window.once("screenshot-window-args-applied", move |_| {
            let _ = window_for_ready.show();
            let _ = window_for_ready.set_focus();
        });
        let _ = window.hide();
        let emit_result = window.emit("screenshot-window-args", args.clone());
        println!("[open_screenshot_window] Emit result: {:?}", emit_result);
        emit_result.map_err(anyhow::Error::from)?;
    } else {
        println!("[open_screenshot_window] Creating new window");
        let json_args = serde_json::to_string(&args).unwrap_or_default();
        let init_script = format!("window.__INITIAL_SCREENSHOT_ARGS__ = {};", json_args);

        let window = WebviewWindowBuilder::new(
            &handle,
            "screenshot_window",
            WebviewUrl::App("index.html".into()),
        )
        .title("Screenshots")
        .inner_size(1200.0, 800.0)
        .visible(false) // Create hidden to avoid flicker
        .center() // Center on screen
        .decorations(false)
        .initialization_script(&init_script)
        .build()
        .map_err(anyhow::Error::from)?;

        // Show the window only after frontend initial layout is ready.
        let window_for_ready = window.clone();
        window.once("screenshot-window-ready", move |_| {
            let _ = window_for_ready.show();
            let _ = window_for_ready.set_focus();
        });
    }
    Ok(())
}

#[tauri::command]
pub async fn app_log(level: String, message: String) -> Result<(), CommandError> {
    match level.as_str() {
        "debug" => log::debug!("{}", message),
        "info" => log::info!("{}", message),
        "warn" => log::warn!("{}", message),
        "error" => log::error!("{}", message),
        _ => log::error!("{}", message),
    }

    Ok(())
}

#[tauri::command]
pub async fn update_shortcut_registration(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    new_shortcut_key: Option<String>,
) -> Result<(), CommandError> {
    let normalized_shortcut_key = normalize_shortcut_key(new_shortcut_key);
    let new_shortcut = parse_shortcut_key(normalized_shortcut_key.as_deref())?;
    let old_shortcut = modules
        .collection_use_case()
        .get_app_setting("shortcut_key".to_string())
        .await
        .ok()
        .flatten()
        .and_then(|key| parse_shortcut_key(Some(&key)).ok().flatten());
    let pause_shortcut = modules
        .collection_use_case()
        .get_app_setting("pause_shortcut_key".to_string())
        .await
        .ok()
        .flatten()
        .and_then(|key| parse_shortcut_key(Some(&key)).ok().flatten());

    ensure_shortcut_not_reserved(
        new_shortcut,
        pause_shortcut,
        normalized_shortcut_key.as_deref(),
        "pause shortcut",
    )?;

    if new_shortcut == old_shortcut {
        modules
            .collection_use_case()
            .set_app_setting("shortcut_key".to_string(), normalized_shortcut_key)
            .await?;
        return Ok(());
    }

    let old_shortcut_was_registered = if let Some(old_shortcut) = old_shortcut {
        unregister_shortcut_if_needed(&handle, old_shortcut)?
    } else {
        false
    };
    let mut new_shortcut_was_registered = false;

    if let Some(new_shortcut) = new_shortcut {
        if let Err(e) =
            register_new_shortcut(&handle, new_shortcut, normalized_shortcut_key.as_deref())
        {
            if old_shortcut_was_registered {
                if let Some(old_shortcut) = old_shortcut {
                    let _ = register_new_shortcut(&handle, old_shortcut, None);
                }
            }
            return Err(e);
        }
        new_shortcut_was_registered = true;
    }

    if let Err(e) = modules
        .collection_use_case()
        .set_app_setting("shortcut_key".to_string(), normalized_shortcut_key)
        .await
    {
        if new_shortcut_was_registered {
            if let Some(new_shortcut) = new_shortcut {
                let _ = unregister_shortcut_if_needed(&handle, new_shortcut);
            }
        }
        if old_shortcut_was_registered {
            if let Some(old_shortcut) = old_shortcut {
                let _ = register_new_shortcut(&handle, old_shortcut, None);
            }
        }
        return Err(e.into());
    }

    Ok(())
}

#[tauri::command]
pub async fn update_pause_shortcut_registration(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    new_shortcut_key: Option<String>,
) -> Result<(), CommandError> {
    let normalized_shortcut_key = normalize_shortcut_key(new_shortcut_key);
    let new_shortcut = parse_shortcut_key(normalized_shortcut_key.as_deref())?;
    let old_shortcut = modules
        .collection_use_case()
        .get_app_setting("pause_shortcut_key".to_string())
        .await
        .ok()
        .flatten()
        .and_then(|key| parse_shortcut_key(Some(&key)).ok().flatten());
    let launch_shortcut = modules
        .collection_use_case()
        .get_app_setting("shortcut_key".to_string())
        .await
        .ok()
        .flatten()
        .and_then(|key| parse_shortcut_key(Some(&key)).ok().flatten());

    ensure_shortcut_not_reserved(
        new_shortcut,
        launch_shortcut,
        normalized_shortcut_key.as_deref(),
        "launch shortcut",
    )?;

    if new_shortcut == old_shortcut {
        modules
            .collection_use_case()
            .set_app_setting("pause_shortcut_key".to_string(), normalized_shortcut_key)
            .await?;
        return Ok(());
    }

    ensure_pause_shortcut_change_allowed(
        modules.pause_manager().is_tracking(),
        new_shortcut,
        old_shortcut,
    )?;

    let old_shortcut_was_registered = if let Some(old_shortcut) = old_shortcut {
        unregister_shortcut_if_needed(&handle, old_shortcut)?
    } else {
        false
    };

    if let Some(new_shortcut) = new_shortcut {
        if handle.global_shortcut().is_registered(new_shortcut) {
            if old_shortcut_was_registered {
                if let Some(old_shortcut) = old_shortcut {
                    let _ = register_new_shortcut(&handle, old_shortcut, None);
                }
            }
            return Err(anyhow::anyhow!(
                "shortcut key `{}` is already registered",
                normalized_shortcut_key.as_deref().unwrap_or("")
            )
            .into());
        }
    }

    if let Err(e) = modules
        .collection_use_case()
        .set_app_setting("pause_shortcut_key".to_string(), normalized_shortcut_key)
        .await
    {
        if old_shortcut_was_registered {
            if let Some(old_shortcut) = old_shortcut {
                let _ = register_new_shortcut(&handle, old_shortcut, None);
            }
        }
        return Err(e.into());
    }

    Ok(())
}

#[tauri::command]
pub async fn launch_shortcut_game(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
) -> Result<(), CommandError> {
    if let Ok(Some(game_id_str)) = modules
        .collection_use_case()
        .get_app_setting("shortcut_game_id".to_string())
        .await
    {
        if let Ok(game_id) = game_id_str.parse::<i32>() {
            modules
                .collection_use_case()
                .play_game_and_track(handle.into(), game_id)
                .await?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn create_elements_in_pc(
    modules: State<'_, Arc<Modules>>,
    handle: AppHandle,
    explore_dir_paths: Vec<String>,
    use_cache: bool,
) -> Result<Vec<String>, CommandError> {
    for path_str in &explore_dir_paths {
        if !std::path::Path::new(path_str).is_dir() {
            return Err(CommandError::Anyhow(anyhow::anyhow!(
                "指定されたパスはフォルダではありません: {}",
                path_str
            )));
        }
    }

    let handle = Arc::new(handle);
    let emit_progress = Arc::new(|message| {
        if let Err(e) = handle.emit("progress", ProgressPayload::new(message)) {
            return Err(anyhow::anyhow!(e.to_string()));
        }
        Ok(())
    });
    let cloned_handle = handle.clone();
    let process_each_game_file_callback = Arc::new(Mutex::new(move || {
        if let Err(e) = cloned_handle.emit("progresslive", ProgressLivePayload::new(None)) {
            return Err(anyhow::anyhow!(e.to_string()));
        }
        Ok(())
    }));

    let explored_caches = modules.explored_cache_use_case().get_cache().await?;
    let explore_files: Vec<String> = modules
        .file_use_case()
        .concurrency_get_file_paths(explore_dir_paths)
        .await?
        .into_iter()
        .filter_map(|v| match use_cache && explored_caches.contains(&v) {
            true => None,
            false => Some(v),
        })
        .collect();

    emit_progress(format!(
        "指定したフォルダの .lnk .exe ファイルを取得しました。ファイル数: {}",
        explore_files.len()
    ))?;
    if let Err(e) = handle.emit(
        "progresslive",
        ProgressLivePayload::new(Some(explore_files.len() as i32)),
    ) {
        return Err(CommandError::Anyhow(anyhow::anyhow!(e.to_string())));
    }

    let all_game_cache = modules
        .all_game_cache_use_case()
        .get_all_game_cache()
        .await?;

    let new_elements = modules
        .file_use_case()
        .filter_files_to_collection_elements(
            &handle,
            explore_files.clone(),
            all_game_cache,
            emit_progress,
            process_each_game_file_callback,
        )
        .await?;

    let new_elements_game_caches = modules
        .all_game_cache_use_case()
        .get_by_ids(new_elements.iter().map(|v| v.id.value).collect())
        .await?;
    modules
        .collection_use_case()
        .concurrency_save_thumbnails_from_candidates(
            &handle,
            new_elements_game_caches
                .into_iter()
                .map(|v| {
                    let urls = new_elements
                        .iter()
                        .find(|element| element.id.value == v.id)
                        .map(|element| {
                            get_thumbnail_candidate_urls(element, v.thumbnail_url.clone())
                        })
                        .unwrap_or_else(|| vec![v.thumbnail_url.clone()]);
                    (Id::new(v.id), urls)
                })
                .collect(),
        )
        .await?;

    modules
        .collection_use_case()
        .upsert_collection_elements(&new_elements)
        .await?;

    let new_element_ids = new_elements
        .iter()
        .map(|v| v.id.clone())
        .collect::<Vec<Id<_>>>();
    modules
        .collection_use_case()
        .concurrency_upsert_collection_element_thumbnail_size(&handle, new_element_ids)
        .await?;

    modules
        .explored_cache_use_case()
        .add_cache(explore_files)
        .await?;

    Ok(new_elements.into_iter().map(|v| v.gamename).collect())
}

#[tauri::command]
pub async fn get_nearest_key_and_distance(
    key: String,
    calculate_distance_kv: Vec<(String, String)>,
) -> Result<(String, f32), CommandError> {
    let key = normalize(&key);
    let normalized_kv = calculate_distance_kv
        .into_iter()
        .map(|v| (normalize(&v.0), normalize(&v.1)))
        .collect::<Vec<(String, String)>>();

    for (comp_key, comp_value) in normalized_kv.iter() {
        if key == *comp_key {
            return Ok((comp_value.to_string(), 1.0));
        }
    }

    let (max_distance_value, max_distance) = find_nearest(&key, &normalized_kv);

    match max_distance_value {
        Some(value) => Ok((value.to_string(), max_distance)),
        _ => Err(CommandError::Anyhow(anyhow::anyhow!(
            "maybe calculate_distance_kv is empty."
        ))),
    }
}

#[tauri::command]
pub async fn upload_image(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    id: i32,
    base64_image: String,
) -> Result<String, CommandError> {
    Ok(modules
        .file_use_case()
        .upload_image(&Arc::new(handle), id, base64_image)
        .await?)
}

#[tauri::command]
pub async fn upsert_collection_element(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    exe_path: Option<String>,
    lnk_path: Option<String>,
    game_cache: AllGameCacheOne,
) -> Result<(), CommandError> {
    if let Some(path) = &exe_path {
        if !std::path::Path::new(path).is_file() {
            return Err(CommandError::Anyhow(anyhow::anyhow!(
                "指定されたパスはファイルではありません: {}",
                path
            )));
        }
    }
    if let Some(path) = &lnk_path {
        if !std::path::Path::new(path).is_file() {
            return Err(CommandError::Anyhow(anyhow::anyhow!(
                "指定されたパスはファイルではありません: {}",
                path
            )));
        }
    }

    let install_at;
    if let Some(path) = exe_path.clone() {
        install_at = get_file_created_at_sync(&path);
    } else if let Some(path) = lnk_path.clone() {
        let metadatas = get_lnk_metadatas(vec![path.as_str()])?;
        let metadata = metadatas
            .get(path.as_str())
            .ok_or(anyhow::anyhow!("metadata cannot get"))?;

        install_at = get_file_created_at_sync(&metadata.path);
    } else {
        install_at = None;
    }
    let thumbnail_url = game_cache.thumbnail_url;
    let new_element = NewCollectionElement::new(
        Id::new(game_cache.id),
        game_cache.gamename,
        exe_path,
        lnk_path,
        install_at,
    );
    let thumbnail_urls = get_thumbnail_candidate_urls(&new_element, thumbnail_url);
    let handle = Arc::new(handle);
    modules
        .collection_use_case()
        .upsert_collection_element(&new_element)
        .await?;
    modules
        .collection_use_case()
        .save_element_icon(&handle, &new_element)
        .await?;
    modules
        .collection_use_case()
        .save_element_thumbnail_from_candidates(&handle, &new_element.id, thumbnail_urls)
        .await?;
    Ok(modules
        .collection_use_case()
        .upsert_collection_element_thumbnail_size(&handle, &new_element.id)
        .await?)
}

#[tauri::command]
pub async fn update_collection_element_icon(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    id: i32,
    path: String,
) -> Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .update_collection_element_icon(&Arc::new(handle), &Id::new(id), path)
        .await?)
}

#[tauri::command]
pub async fn get_default_import_dirs() -> Result<Vec<String>, CommandError> {
    let user_menu = dirs::home_dir()
        .ok_or(anyhow::anyhow!("cannot got home dir"))?
        .join("AppData")
        .join("Roaming")
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .to_string_lossy()
        .to_string();

    let system_menu = "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs";

    Ok(vec![user_menu, system_menu.to_string()])
}

#[tauri::command]
pub async fn play_game(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    element_id: i32,
    _is_admin: Option<bool>,
) -> Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .play_game_and_track(handle.into(), element_id)
        .await?)
}

#[tauri::command]
pub async fn get_app_setting(
    modules: State<'_, Arc<Modules>>,
    key: String,
) -> Result<Option<String>, CommandError> {
    Ok(modules.collection_use_case().get_app_setting(key).await?)
}

#[tauri::command]
pub async fn get_game_screenshot_cache(
    modules: State<'_, Arc<Modules>>,
    collection_element_id: i32,
) -> Result<Option<GameScreenshotCache>, CommandError> {
    Ok(modules
        .collection_use_case()
        .get_game_screenshot_cache(collection_element_id)
        .await?
        .map(Into::into))
}

#[tauri::command]
pub async fn upsert_game_screenshot_cache(
    modules: State<'_, Arc<Modules>>,
    cache: GameScreenshotCache,
) -> Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .upsert_game_screenshot_cache(cache.into())
        .await?)
}

#[tauri::command]
pub async fn set_app_setting(
    app_handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    key: String,
    value: Option<String>,
) -> Result<(), CommandError> {
    modules
        .collection_use_case()
        .set_app_setting(key.clone(), value)
        .await?;

    if key == "shortcut_game_id" {
        app_handle
            .emit("shortcut-game-changed", ())
            .map_err(anyhow::Error::from)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_play_time_minutes(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    collection_element_id: i32,
) -> Result<f32, CommandError> {
    Ok(modules
        .file_use_case()
        .get_play_time_minutes(&Arc::new(handle), &Id::new(collection_element_id))?)
}

#[tauri::command]
pub async fn get_collection_element(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    collection_element_id: i32,
) -> Result<CollectionElement, CommandError> {
    match modules
        .collection_use_case()
        .get_element_by_element_id(&Id::new(collection_element_id))
        .await
        .map(|v| CollectionElement::from_domain(&Arc::new(handle), v))
    {
        Ok(v) => Ok(v),
        Err(e) => {
            if let Some(UseCaseError::CollectionElementIsNotFound) =
                e.downcast_ref::<UseCaseError>()
            {
                return Err(CommandError::NotFound);
            }
            Err(CommandError::Anyhow(e))
        }
    }
}

#[tauri::command]
pub async fn delete_collection_element(
    handle: tauri::AppHandle,
    modules: State<'_, Arc<Modules>>,
    collection_element_id: i32,
) -> Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .delete_collection_element_by_id(&Arc::new(handle), &Id::new(collection_element_id))
        .await?)
}

#[tauri::command]
pub async fn get_not_registered_detail_element_ids(
    modules: State<'_, Arc<Modules>>,
) -> Result<Vec<i32>, CommandError> {
    Ok(modules
        .collection_use_case()
        .get_not_registered_detail_element_ids()
        .await?
        .into_iter()
        .map(|v| v.value)
        .collect())
}

#[tauri::command]
pub async fn create_element_details(
    modules: State<'_, Arc<Modules>>,
    details: Vec<CreateCollectionElementDetail>,
) -> Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .create_element_details(details.into_iter().map(|v| v.into()).collect())
        .await?)
}

#[tauri::command]
pub async fn get_all_elements(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
) -> Result<Vec<CollectionElement>, CommandError> {
    let handle = &Arc::new(handle);
    Ok(modules
        .collection_use_case()
        .get_all_elements(handle)
        .await?
        .into_iter()
        .map(|v| CollectionElement::from_domain(handle, v))
        .collect())
}

#[tauri::command]
pub async fn update_element_like(
    modules: State<'_, Arc<Modules>>,
    id: i32,
    is_like: bool,
) -> Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .update_element_like_at(&Id::new(id), is_like)
        .await?)
}

#[tauri::command]
pub async fn update_element_play_status(
    modules: State<'_, Arc<Modules>>,
    id: i32,
    play_status: i32,
) -> Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .update_element_play_status(&Id::new(id), play_status)
        .await?)
}

#[tauri::command]
pub async fn adjust_untracked_play_time_seconds(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    id: i32,
    seconds: i32,
) -> Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .adjust_untracked_play_time_seconds(&Arc::new(handle), &Id::new(id), seconds)
        .await?)
}

#[tauri::command]
pub async fn get_collection_element_daily_play_times(
    modules: State<'_, Arc<Modules>>,
    collection_element_id: i32,
) -> Result<Vec<CollectionElementDailyPlayTime>, CommandError> {
    Ok(modules
        .collection_use_case()
        .get_collection_element_daily_play_times(&Id::new(collection_element_id))
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), CommandError> {
    let p = std::path::Path::new(&path);
    let path = match p.is_file() {
        true => p
            .parent()
            .ok_or(anyhow::anyhow!("parent not found"))?
            .to_string_lossy()
            .to_string(),
        false => path,
    };
    let err_msg = anyhow::anyhow!("Failed to open folder at path: {}", path);
    std::process::Command::new("explorer")
        .arg(path)
        .output()
        .map_err(|_| err_msg)?;

    Ok(())
}

#[tauri::command]
pub async fn get_all_game_cache_last_updated(
    modules: State<'_, Arc<Modules>>,
) -> Result<(i32, String), CommandError> {
    let last_updated = modules
        .all_game_cache_use_case()
        .get_cache_last_updated()
        .await?;
    Ok((last_updated.0, last_updated.1.to_rfc3339()))
}

#[tauri::command]
pub async fn update_all_game_cache(
    modules: State<'_, Arc<Modules>>,
    game_caches: Vec<AllGameCacheOne>,
) -> Result<(), CommandError> {
    modules
        .all_game_cache_use_case()
        .update_all_game_cache(game_caches.into_iter().map(|v| v.into()).collect())
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_game_candidates(
    modules: State<'_, Arc<Modules>>,
    filepath: String,
) -> Result<Vec<(i32, String)>, CommandError> {
    let all_game_cache = modules
        .all_game_cache_use_case()
        .get_all_game_cache()
        .await?;

    Ok(modules
        .file_use_case()
        .get_game_candidates(all_game_cache, filepath)
        .await?
        .into_iter()
        .map(|c| (c.id, c.gamename))
        .collect())
}

#[tauri::command]
pub async fn search_all_game_cache(
    modules: State<'_, Arc<Modules>>,
    query: String,
    limit: i64,
    offset: i64,
) -> Result<Vec<AllGameCacheOne>, CommandError> {
    let limit = limit.clamp(1, 240);
    let offset = offset.max(0);
    Ok(modules
        .all_game_cache_use_case()
        .search(query, limit, offset)
        .await?
        .into_iter()
        .map(|v| v.into())
        .collect())
}

#[tauri::command]
pub async fn get_exe_path_by_lnk(filepath: String) -> Result<String, CommandError> {
    Ok(get_exe_path_from_lnk(&filepath).await?)
}

#[tauri::command]
pub async fn get_game_cache_by_id(
    modules: State<'_, Arc<Modules>>,
    id: i32,
) -> Result<Option<AllGameCacheOne>, CommandError> {
    Ok(modules
        .all_game_cache_use_case()
        .get(id)
        .await?
        .map(|v| v.into()))
}

#[tauri::command]
pub async fn save_screenshot_by_pid(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    work_id: i32,
    process_id: u32,
) -> Result<String, CommandError> {
    let upload_path = modules
        .file_use_case()
        .get_new_upload_image_path(&Arc::new(handle), work_id)?;
    modules
        .process_use_case()
        .save_screenshot_by_pid(process_id, &upload_path)
        .await?;
    Ok(upload_path)
}

#[tauri::command]
pub async fn save_fullscreen_screenshot(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    work_id: i32,
) -> Result<String, CommandError> {
    let handle = Arc::new(handle);
    let upload_path = modules
        .file_use_case()
        .get_new_upload_image_path(&handle, work_id)?;
    modules
        .process_use_case()
        .save_fullscreen_screenshot(&upload_path)
        .await?;
    modules
        .collection_use_case()
        .register_screenshot_file(&handle, work_id, upload_path.clone())
        .await?;
    Ok(upload_path)
}

#[cfg(target_os = "windows")]
fn mouse_input(flags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[tauri::command]
pub async fn send_right_click() -> Result<(), CommandError> {
    #[cfg(target_os = "windows")]
    {
        let inputs = [
            mouse_input(MOUSEEVENTF_RIGHTDOWN),
            mouse_input(MOUSEEVENTF_RIGHTUP),
        ];
        let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
        if sent != inputs.len() as u32 {
            return Err(anyhow::anyhow!("右クリック入力の送信に失敗しました").into());
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err(anyhow::anyhow!("右クリック入力はWindowsでのみ利用できます").into())
    }
}

#[tauri::command]
pub async fn update_game_image(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    element_id: i32,
    image_type: String, // "icon" or "thumbnail"
    new_image_path: String,
) -> Result<(), CommandError> {
    let id = &Id::new(element_id);
    let handle = Arc::new(handle);

    if image_type == "thumbnail" {
        let dest_path = get_thumbnail_path(&handle, id);
        let img = image::open(&new_image_path).map_err(anyhow::Error::from)?;
        img.save(dest_path).map_err(anyhow::Error::from)?;
    } else if image_type == "icon" {
        let dest_path = get_icon_path(&handle, id);
        let img = image::open(&new_image_path).map_err(anyhow::Error::from)?;
        let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
        // RGBA8形式に変換
        let image = img.to_rgba8();
        let icon_image =
            ico::IconImage::from_rgba_data(image.width(), image.height(), image.into_raw());
        icon_dir.add_entry(ico::IconDirEntry::encode(&icon_image).map_err(anyhow::Error::from)?);
        let file = std::fs::File::create(dest_path).map_err(anyhow::Error::from)?;
        icon_dir.write(file).map_err(anyhow::Error::from)?;
    }

    modules.collection_use_case().touch_element(id).await?;

    modules
        .collection_use_case()
        .upsert_collection_element_thumbnail_size(&handle, id)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn toggle_pause_tracking(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
) -> Result<bool, CommandError> {
    super::logic::toggle_pause_and_notify(&handle, &modules).map_err(CommandError::Anyhow)
}

#[tauri::command]
pub async fn get_pause_state(modules: State<'_, Arc<Modules>>) -> Result<bool, CommandError> {
    Ok(modules.pause_manager().is_paused())
}
use crate::domain::repository::screenshot::Screenshot;

#[tauri::command]
pub async fn get_game_screenshots(
    modules: State<'_, Arc<Modules>>,
    app_handle: tauri::AppHandle,
    game_id: i32,
) -> Result<Vec<Screenshot>, CommandError> {
    Ok(modules
        .collection_use_case()
        .get_game_screenshots(&Arc::new(app_handle), game_id)
        .await?)
}

#[tauri::command]
pub async fn get_all_screenshots(
    modules: State<'_, Arc<Modules>>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<Screenshot>, CommandError> {
    Ok(modules
        .collection_use_case()
        .get_all_screenshots(&Arc::new(app_handle))
        .await?)
}

#[tauri::command]
pub async fn import_screenshot(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    game_id: i32,
    file_path: String,
) -> Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .import_screenshot(&Arc::new(handle), game_id, file_path)
        .await?)
}

#[tauri::command]
pub async fn delete_screenshot(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    screenshot_id: i32,
) -> Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .delete_screenshot(&Arc::new(handle), screenshot_id)
        .await?)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotOrderUpdate {
    pub id: i32,
    pub order_index: i32,
}

#[tauri::command]
pub async fn update_screenshots_order(
    modules: State<'_, Arc<Modules>>,
    updates: Vec<ScreenshotOrderUpdate>,
) -> Result<(), CommandError> {
    let updates_vec: Vec<(i32, i32)> = updates.into_iter().map(|u| (u.id, u.order_index)).collect();

    Ok(modules
        .collection_use_case()
        .update_screenshots_order(updates_vec)
        .await?)
}

#[tauri::command]
pub async fn update_collection_element_path(
    modules: State<'_, Arc<Modules>>,
    id: i32,
    path: String,
) -> Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .update_collection_element_path(&Id::new(id), path)
        .await?)
}

#[tauri::command]
pub async fn delete_collection_element_logical(
    modules: State<'_, Arc<Modules>>,
    id: i32,
) -> Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .delete_collection_element_logical(&Id::new(id))
        .await?)
}

#[tauri::command]
pub fn show_main_window(handle: AppHandle) -> Result<(), CommandError> {
    if let Some(window) = handle.get_webview_window("main") {
        let _ = window.unminimize();
        window.show().map_err(anyhow::Error::from)?;
        window.set_focus().map_err(anyhow::Error::from)?;
    }

    if let Some(window) = handle.get_webview_window("tray_menu") {
        let _ = window.hide();
    }

    Ok(())
}

fn window_state_flags() -> StateFlags {
    StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED | StateFlags::FULLSCREEN
}

fn save_current_window_state(handle: &AppHandle) -> anyhow::Result<()> {
    if let Some(window) = handle.get_webview_window("main") {
        if window.is_minimized().unwrap_or(false) {
            let _ = window.unminimize();
        }
    }

    handle
        .save_window_state(window_state_flags())
        .map_err(anyhow::Error::from)
}

#[tauri::command]
pub fn save_main_window_state(handle: AppHandle) -> Result<(), CommandError> {
    save_current_window_state(&handle).map_err(CommandError::from)
}

#[tauri::command]
pub fn hide_tray_menu(handle: AppHandle) -> Result<(), CommandError> {
    if let Some(window) = handle.get_webview_window("tray_menu") {
        window.hide().map_err(anyhow::Error::from)?;
    }

    Ok(())
}

#[tauri::command]
pub fn quit_app(handle: AppHandle) {
    let _ = save_current_window_state(&handle);
    handle.exit(0);
}
