use std::{fs, sync::Arc};

use chrono::Local;
use derive_new::new;
use sysinfo::{ProcessExt, System, SystemExt};
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tokio::time::{interval, Duration, Instant};

use super::error::UseCaseError;
use crate::{
    domain::{
        collection::{CollectionElement, NewCollectionElement, NewCollectionElementDetail},
        file::{
            get_exe_path_from_lnk, get_icon_path, get_lnk_metadatas, get_thumbnail_path,
            save_icon_to_png, save_thumbnail,
        },
        repository::collection::CollectionRepository,
        Id,
    },
    infrastructure::repositoryimpl::repository::RepositoriesExt,
};

#[derive(new)]
pub struct CollectionUseCase<R: RepositoriesExt> {
    repositories: Arc<R>,
}

impl<R: RepositoriesExt + Send + Sync + 'static> CollectionUseCase<R> {
    pub async fn play_game_and_track(
        &self,
        handle: Arc<AppHandle>,
        element_id: i32,
    ) -> anyhow::Result<()> {
        let element = self.get_element_by_element_id(&Id::new(element_id)).await?;

        self.update_element_last_play_at(&Id::new(element_id))
            .await?;

        let path_str = match (element.exe_path, element.lnk_path) {
            (Some(p), _) => p,
            (None, Some(p)) => p,
            (None, None) => {
                return Err(anyhow::anyhow!(
                    "実行ファイルまたはショートカットが見つかりません"
                ))
            }
        };

        let mut system_before = System::new_all();
        system_before.refresh_processes();
        let pids_before: std::collections::HashSet<_> =
            system_before.processes().keys().cloned().collect();

        handle
            .shell()
            .open(&path_str, None)
            .map_err(anyhow::Error::from)?;
        println!("[INFO] Opening path with shell: {}", &path_str);

        let game_name = element.gamename.clone();
        let path_str_clone = path_str.clone();
        let repositories = self.repositories.clone();

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
                    println!(
                        "[WARN] Game process search timed out. Play time may not be recorded."
                    );
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

                    if system_folders
                        .iter()
                        .any(|folder| path_lower.starts_with(folder))
                    {
                        continue; // 除外
                    }

                    // スコア付け
                    let mut score = 0;
                    // 最優先: 起動パスと完全一致
                    let final_exe_path_str = if path_str_clone.to_lowercase().ends_with(".lnk") {
                        get_exe_path_from_lnk(&path_str_clone)
                            .await
                            .unwrap_or(path_str_clone.clone())
                    } else {
                        path_str_clone.clone()
                    };
                    let final_exe_path = std::path::Path::new(&final_exe_path_str);

                    if exe_path == final_exe_path {
                        score = 3;
                    }
                    // 次点: 有名なゲームフォルダ
                    else if game_folders
                        .iter()
                        .any(|folder| path_lower.contains(folder))
                    {
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
                    let max_score = candidates
                        .iter()
                        .map(|(_, score)| *score)
                        .max()
                        .unwrap_or(0);
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
                println!(
                    "Start monitoring process (PID: {}) for game {}",
                    pid_to_monitor, game_name
                );
                let start_time = Instant::now();
                let mut interval = interval(Duration::from_secs(10));
                let mut system = System::new_all();

                loop {
                    interval.tick().await;
                    system.refresh_processes();
                    if system.process(pid_to_monitor).is_none() {
                        let duration = start_time.elapsed().as_secs() as i32;
                        if duration > 0 {
                            println!(
                                "Game {} (PID: {}) finished. Play time: {} seconds.",
                                game_name, pid_to_monitor, duration
                            );
                            let _ = repositories
                                .collection_repository()
                                .add_play_time_seconds(&Id::new(element_id), duration)
                                .await;
                        }

                        println!("Updating last_play_at to the session end time.");
                        let _ = repositories
                            .collection_repository()
                            .update_element_last_play_at_by_id(&Id::new(element_id), Local::now())
                            .await;

                        break;
                    }
                }
            }
        });

        Ok(())
    }
    pub async fn upsert_collection_element(
        &self,
        source: &NewCollectionElement,
    ) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .upsert_collection_element(source)
            .await?;
        Ok(())
    }
    pub async fn upsert_collection_element_thumbnail_size(
        &self,
        handle: &Arc<AppHandle>,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        let thumbnail_path = get_thumbnail_path(handle, id);
        match image::image_dimensions(thumbnail_path) {
            Ok((width, height)) => {
                self.repositories
                    .collection_repository()
                    .upsert_collection_element_thumbnail_size(id, width as i32, height as i32)
                    .await?;
            }
            Err(e) => {
                eprintln!(
                    "[upsert_collection_element_thumbnail_size] {}",
                    e.to_string()
                );
            }
        }
        Ok(())
    }
    pub async fn concurency_upsert_collection_element_thumbnail_size(
        &self,
        handle: &Arc<AppHandle>,
        ids: Vec<Id<CollectionElement>>,
    ) -> anyhow::Result<()> {
        use futures::StreamExt as _;

        futures::stream::iter(ids.into_iter())
            .map(move |id| {
                let id = id.clone();
                let handle_cloned = handle.clone();
                async move {
                    self.upsert_collection_element_thumbnail_size(&handle_cloned, &id)
                        .await
                }
            })
            .buffered(50)
            .for_each(|v| async move {
                match v {
                    Err(e) => eprintln!(
                        "[concurency_upsert_collection_element_thumbnail_size] {}",
                        e.to_string()
                    ),
                    _ => {}
                }
            })
            .await;
        Ok(())
    }
    pub async fn upsert_collection_elements(
        &self,
        source: &Vec<NewCollectionElement>,
    ) -> anyhow::Result<()> {
        for v in source.into_iter() {
            self.repositories
                .collection_repository()
                .upsert_collection_element(v)
                .await?
        }
        Ok(())
    }

    pub async fn get_element_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<CollectionElement> {
        Ok(self
            .repositories
            .collection_repository()
            .get_element_by_element_id(id)
            .await?
            .ok_or(UseCaseError::CollectionElementIsNotFound)?)
    }

    pub async fn update_collection_element_icon(
        &self,
        handle: &Arc<AppHandle>,
        id: &Id<CollectionElement>,
        path: String,
    ) -> anyhow::Result<()> {
        let save_icon_path = get_icon_path(handle, id);
        fs::copy(path, save_icon_path)?;
        Ok(())
    }

    pub async fn save_element_icon(
        &self,
        handle: &Arc<AppHandle>,
        element: &NewCollectionElement,
    ) -> anyhow::Result<()> {
        let id = &element.id;
        let icon_path;
        if let Some(lnk_path) = element.lnk_path.clone() {
            let metadatas = get_lnk_metadatas(vec![lnk_path.as_str()])?;
            let metadata = metadatas
                .get(lnk_path.as_str())
                .ok_or(anyhow::anyhow!("metadata cannot get"))?;
            if metadata.icon.to_lowercase().ends_with("ico") {
                println!("icon is ico");
                icon_path = metadata.icon.clone();
            } else {
                icon_path = metadata.path.clone();
            }
        } else if let Some(exe_path) = element.exe_path.clone() {
            icon_path = exe_path;
        } else {
            eprintln!("lnk_path and exe_path are None");
            return Ok(());
        }
        Ok(save_icon_to_png(handle, &icon_path, id)?.await??)
    }

    pub async fn save_element_thumbnail(
        &self,
        handle: &Arc<AppHandle>,
        id: &Id<CollectionElement>,
        src_url: String,
    ) -> anyhow::Result<()> {
        Ok(save_thumbnail(handle, id, src_url).await??)
    }

    pub async fn concurency_save_thumbnails(
        &self,
        handle: &Arc<AppHandle>,
        args: Vec<(Id<CollectionElement>, String)>,
    ) -> anyhow::Result<()> {
        use futures::StreamExt as _;

        futures::stream::iter(args.into_iter())
            .map(|(id, url)| save_thumbnail(handle, &id, url))
            .buffered(50)
            .map(|v| v?)
            .for_each(|v| async move {
                match v {
                    Err(e) => eprintln!("[concurency_save_thumbnails] {}", e.to_string()),
                    _ => {}
                }
            })
            .await;
        Ok(())
    }

    pub async fn delete_collection_element_by_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        let existed = self
            .repositories
            .collection_repository()
            .get_element_by_element_id(id)
            .await?;
        if existed.is_none() {
            return Err(UseCaseError::CollectionElementIsNotFound.into());
        }
        self.repositories
            .collection_repository()
            .delete_collection_element(id)
            .await
    }

    pub async fn get_not_registered_detail_element_ids(
        &self,
    ) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        self.repositories
            .collection_repository()
            .get_not_registered_detail_element_ids()
            .await
    }

    pub async fn create_element_details(
        &self,
        details: Vec<NewCollectionElementDetail>,
    ) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .create_element_details(details)
            .await
    }

    pub async fn update_element_last_play_at(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .update_element_last_play_at_by_id(id, Local::now())
            .await?;
        Ok(())
    }
    pub async fn update_element_like_at(
        &self,
        id: &Id<CollectionElement>,
        is_like: bool,
    ) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .update_element_like_at_by_id(id, is_like.then_some(Local::now()))
            .await?;
        Ok(())
    }
    pub async fn update_element_play_status(
        // 追加
        &self,
        id: &Id<CollectionElement>,
        play_status: i32,
    ) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .update_element_play_status_by_id(id, play_status)
            .await?;
        Ok(())
    }
    pub async fn add_play_time_seconds(
        &self,
        id: &Id<CollectionElement>,
        seconds: i32,
    ) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .add_play_time_seconds(id, seconds)
            .await
    }
    pub async fn delete_element(&self, id: &Id<CollectionElement>) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .delete_collection_element(id) // delete_element_by_id から delete_collection_element に変更
            .await?;
        Ok(())
    }
    pub async fn get_all_elements(
        &self,
        handle: &Arc<AppHandle>,
    ) -> anyhow::Result<Vec<CollectionElement>> {
        let null_size_ids = self
            .repositories
            .collection_repository()
            .get_null_thumbnail_size_element_ids()
            .await?;
        self.concurency_upsert_collection_element_thumbnail_size(handle, null_size_ids)
            .await?;

        self.repositories
            .collection_repository()
            .get_all_elements()
            .await
    }

    pub async fn touch_element(&self, id: &Id<CollectionElement>) -> anyhow::Result<()> {
        self.repositories.collection_repository().touch(id).await
    }

    pub async fn get_app_setting(&self, key: String) -> anyhow::Result<Option<String>> {
        self.repositories
            .collection_repository()
            .get_app_setting(key)
            .await
    }

    pub async fn set_app_setting(&self, key: String, value: Option<String>) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .set_app_setting(key, value)
            .await
    }
}
