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
        distance::get_comparable_distance,
        file::{get_file_created_at_sync, get_icon_path, get_lnk_metadatas, get_thumbnail_path, normalize},
        Id,
    },
    usecase::models::collection::CreateCollectionElementDetail,
};
use std::{
    io::{BufWriter, Write},
    num::NonZeroU32,
    sync::{Arc, Mutex},
};
use image::{ColorType, ImageEncoder};
use image::codecs::png::PngEncoder;
use fast_image_resize as fr;
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use tauri::{AppHandle, Emitter, State};
use tokio::time::{interval, Duration, Instant};
use chrono::Local;

#[tauri::command]
pub async fn create_elements_in_pc(
    modules: State<'_, Arc<Modules>>,
    handle: AppHandle,
    explore_dir_paths: Vec<String>,
    use_cache: bool,
) -> Result<Vec<String>, CommandError> {
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
        .concurency_get_file_paths(explore_dir_paths)
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
        .concurency_save_thumbnails(
            &handle,
            new_elements_game_caches
                .into_iter()
                .map(|v| (Id::new(v.id), v.thumbnail_url))
                .collect(),
        )
        .await?;

    modules
        .collection_use_case()
        .upsert_collection_elements(&new_elements)
        .await?;

    let new_element_ids = new_elements.iter().map(|v| v.id.clone()).collect::<Vec<Id<_>>>();
    modules
        .collection_use_case()
        .concurency_upsert_collection_element_thumbnail_size(&handle, new_element_ids)
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

    let mut max_distance = 0.0;
    let mut max_distance_value = None;
    for (comp_key, comp_value) in normalized_kv.into_iter() {
        let distance = get_comparable_distance(&key, &comp_key);
        if max_distance < distance {
            max_distance = distance;
            max_distance_value = Some(comp_value);
        }
    }

    match max_distance_value {
        Some(value) => Ok((value, max_distance)),
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
    let install_at;
    if let Some(path) = exe_path.clone() {
        install_at = get_file_created_at_sync(&path);
    } else if let Some(path) = lnk_path.clone() {
        let metadatas = get_lnk_metadatas(vec![path.as_str()])?;
        let metadata = metadatas
            .get(path.as_str())
            .ok_or(anyhow::anyhow!("metadata cannot get"))?;
        println!(
            "metadata.path: {}, metadata.icon: {}",
            metadata.path, metadata.icon
        );
        install_at = get_file_created_at_sync(&metadata.path);
    } else {
        install_at = None;
    }
    let new_element = NewCollectionElement::new(
        Id::new(game_cache.id),
        game_cache.gamename,
        exe_path,
        lnk_path,
        install_at,
    );
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
        .save_element_thumbnail(&handle, &new_element.id, game_cache.thumbnail_url)
        .await?;
    Ok(modules
        .collection_use_case()
        .upsert_collection_element_thumbnail_size(&handle, &new_element.id)
        .await?)
}

#[tauri::command]
pub async fn update_collection_element_thumbnails(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    ids: Vec<i32>,
) -> Result<(), CommandError> {
    let all_game_cache = modules.all_game_cache_use_case().get_by_ids(ids.clone()).await?;
    let handle = Arc::new(handle);
    modules
        .collection_use_case()
        .concurency_save_thumbnails(
            &handle,
            all_game_cache
                .into_iter()
                .map(|v| (Id::new(v.id), v.thumbnail_url))
                .collect(),
        )
        .await?;
    Ok(modules
        .collection_use_case()
        .concurency_upsert_collection_element_thumbnail_size(
            &handle,
            ids.into_iter().map(|v| Id::new(v)).collect(),
        )
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

use tauri_plugin_shell::ShellExt;

#[tauri::command]
pub async fn play_game(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    element_id: i32,
    _is_admin: Option<bool>,
) -> Result<(), CommandError> {
    let element = modules
        .collection_use_case()
        .get_element_by_element_id(&Id::new(element_id))
        .await?;

    modules
        .collection_use_case()
        .update_element_last_play_at(&Id::new(element_id))
        .await?;

    let path_str = match (element.exe_path, element.lnk_path) {
        (Some(p), _) => p,
        (None, Some(p)) => p,
        (None, None) => {
            return Err(CommandError::Anyhow(anyhow::anyhow!(
                "実行ファイルまたはショートカットが見つかりません"
            )))
        }
    };

    let mut system_before = System::new_all();
    system_before.refresh_processes();
    let pids_before: std::collections::HashSet<_> = system_before.processes().keys().cloned().collect();

    handle.shell().open(&path_str, None).map_err(anyhow::Error::from)?;
    println!("[INFO] Opening path with shell: {}", &path_str);

    let modules_clone = modules.inner().clone();
    let game_name = element.gamename.clone();
    let path_str_clone = path_str.clone();

    tauri::async_runtime::spawn(async move {
        // ランチャーがゲーム本体を起動するまで5秒待つ
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        let search_timeout = Duration::from_secs(45);
        let search_start_time = Instant::now();
        let mut target_pid: Option<sysinfo::Pid> = None;

        println!("Searching for the new game process...");
        
        // ▼▼▼ 修正: 優先度付けを行う新しい特定ロジック ▼▼▼
        loop {
            if search_start_time.elapsed() > search_timeout {
                println!("[WARN] Game process search timed out. Play time may not be recorded.");
                break;
            }
            tokio::time::sleep(Duration::from_secs(2)).await;

            let mut system_after = System::new_all();
            system_after.refresh_processes();

            let new_processes: Vec<_> = system_after
                .processes()
                .values()
                .filter(|p| !pids_before.contains(&p.pid()))
                .collect();

            if new_processes.is_empty() {
                continue;
            }

            let mut candidates: Vec<(&sysinfo::Process, i32)> = Vec::new();
            let system_folders = ["c:\\windows"];
            let game_folders = ["VisualNovel", "steamapps", "dmmgameplayer"];

            for process in new_processes {
                let exe_path = process.exe();
                let path_lower = exe_path.to_string_lossy().to_lowercase();
                
                if system_folders.iter().any(|folder| path_lower.starts_with(folder)) {
                    continue; // 除外
                }
                
                // スコア付け
                let mut score = 0;
                // 最優先: 起動パスと完全一致
                let final_exe_path_str = if path_str_clone.to_lowercase().ends_with(".lnk") {
                    get_exe_path_by_lnk(path_str_clone.clone()).await.unwrap_or(path_str_clone.clone())
                } else {
                    path_str_clone.clone()
                };
                let final_exe_path = std::path::Path::new(&final_exe_path_str);
                
                if exe_path == final_exe_path {
                    score = 3;
                }
                // 次点: 有名なゲームフォルダ
                else if game_folders.iter().any(|folder| path_lower.contains(folder)) {
                    score = 2;
                }
                // それ以外
                else {
                    score = 1;
                }
                candidates.push((process, score));
            }

            // 最もスコアの高い候補の中から、ゲーム名に一番近いものを探す
            if !candidates.is_empty() {
                let max_score = candidates.iter().map(|(_, score)| *score).max().unwrap_or(0);
                if let Some(best_match) = candidates
                    .iter()
                    .filter(|(_, score)| *score == max_score)
                    .min_by_key(|(p, _)| {
                        // 編集距離の代わりに簡易的な文字数差で最終判断
                        (p.name().len() as i32 - game_name.len() as i32).abs()
                    })
                {
                    target_pid = Some(best_match.0.pid());
                }
            }
            
            if target_pid.is_some() {
                println!("Game process identified (PID: {:?}).", target_pid.unwrap());
                break;
            }
        }

        if let Some(pid_to_monitor) = target_pid {
            println!("Start monitoring process (PID: {}) for game {}", pid_to_monitor, game_name);
            let start_time = Instant::now();
            let mut interval = interval(Duration::from_secs(10));
            let mut system = System::new_all();

            loop {
                interval.tick().await;
                system.refresh_processes();
                if system.process(pid_to_monitor).is_none() {
                    let duration = start_time.elapsed().as_secs() as i32;
                    if duration > 0 {
                        println!("Game {} (PID: {}) finished. Play time: {} seconds.", game_name, pid_to_monitor, duration);
                        let _ = modules_clone.collection_use_case().add_play_time_seconds(&Id::new(element_id), duration).await;
                    }
                    
                    println!("Updating last_play_at to the session end time.");
                    let _ = modules_clone
                        .collection_use_case()
                        .update_element_last_play_at(&Id::new(element_id))
                        .await;

                    break;
                }
            }
        }
    });

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
    Ok(modules
        .collection_use_case()
        .get_element_by_element_id(&Id::new(collection_element_id))
        .await
        .and_then(|v| Ok(CollectionElement::from_domain(&Arc::new(handle), v)))?)
}

#[tauri::command]
pub async fn delete_collection_element(
    modules: State<'_, Arc<Modules>>,
    collection_element_id: i32,
) -> Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .delete_collection_element_by_id(&Id::new(collection_element_id))
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
        .get_all_elements(&handle)
        .await?
        .into_iter()
        .map(|v| CollectionElement::from_domain(&handle, v))
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
    let last_updated = modules.all_game_cache_use_case().get_cache_last_updated().await?;
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
    let all_game_cache = modules.all_game_cache_use_case().get_all_game_cache().await?;

    Ok(modules
        .file_use_case()
        .get_game_candidates(all_game_cache, filepath)
        .await?
        .into_iter()
        .map(|c| (c.id, c.gamename))
        .collect())
}

#[tauri::command]
pub async fn get_exe_path_by_lnk(filepath: String) -> Result<String, CommandError> {
    if !filepath.to_lowercase().ends_with("lnk") {
        return Err(CommandError::Anyhow(anyhow::anyhow!(
            "filepath is not ends with lnk"
        )));
    }

    let p: &str = &filepath;
    let metadatas = get_lnk_metadatas(vec![p])?;
    if let Some(meta) = metadatas.get(p) {
        return Ok(meta.path.clone());
    } else {
        return Err(CommandError::Anyhow(anyhow::anyhow!(
            "cannot get lnk metadata"
        )));
    }
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
        .and_then(|v| Some(v.into())))
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

        let width = NonZeroU32::new(img.width()).ok_or(anyhow::anyhow!("failed NonZeroU32::new"))?;
        let height = NonZeroU32::new(img.height()).ok_or(anyhow::anyhow!("failed NonZeroU32::new"))?;
        let mut src_image = fr::Image::from_vec_u8(
            width,
            height,
            img.to_rgba8().into_raw(),
            fr::PixelType::U8x4,
        )?;

        let alpha_mul_div = fr::MulDiv::default();
        alpha_mul_div.multiply_alpha_inplace(&mut src_image.view_mut())?;

        let dst_width_px = 400;
        let dst_width =
            NonZeroU32::new(dst_width_px).ok_or(anyhow::anyhow!("failed NonZeroU32::new"))?;
        let dst_height =
            NonZeroU32::new((height.get() as f32 / width.get() as f32 * dst_width_px as f32) as u32)
                .ok_or(anyhow::anyhow!("failed NonZeroU32::new"))?;

        let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());

        let mut dst_view = dst_image.view_mut();

        let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Box));
        resizer.resize(&src_image.view(), &mut dst_view)?;

        alpha_mul_div.divide_alpha_inplace(&mut dst_view)?;

        let mut result_buf = BufWriter::new(std::fs::File::create(&dest_path)?);

        PngEncoder::new(&mut result_buf).write_image(
            dst_image.buffer(),
            dst_width.get(),
            dst_height.get(),
            ColorType::Rgba8,
        )?;
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

    modules
        .collection_use_case()
        .touch_element(id)
        .await?;

    Ok(())
}