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
        collection::{
            CollectionElement, NewCollectionElement, NewCollectionElementInfo, ScannedGameElement,
        },
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
            tokio::time::sleep(Duration::from_secs(5)).await;

            let search_timeout = Duration::from_secs(45);
            let search_start_time = Instant::now();
            let mut target_pid: Option<sysinfo::Pid> = None;

            println!("Searching for the new game process...");

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
                        continue;
                    }

                    let score;
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
                    else if game_folders
                        .iter()
                        .any(|folder| path_lower.contains(folder))
                    {
                        score = 2;
                    }
                    else {
                        score = 1;
                    }
                    candidates.push((process, score));
                }

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

    pub async fn create_element_details(
        &self,
        details: Vec<NewCollectionElementInfo>,
    ) -> anyhow::Result<()> {
        for v in details.into_iter() {
            self.upsert_collection_element_info(&v).await?;
        }
        Ok(())
    }

    pub async fn upsert_collection_element_info(
        &self,
        info: &crate::domain::collection::NewCollectionElementInfo,
    ) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .upsert_collection_element_info(info)
            .await?;
        Ok(())
    }

    pub async fn create_collection_element(
        &self,
        element: &ScannedGameElement,
    ) -> anyhow::Result<()> {
        use crate::domain::collection::{
            NewCollectionElement, NewCollectionElementInstall, NewCollectionElementPaths,
        };

        let new_element = NewCollectionElement::new(element.id.clone(), element.gamename.clone());
        self.upsert_collection_element(&new_element).await?;

        if element.exe_path.is_some() || element.lnk_path.is_some() {
            let new_paths = NewCollectionElementPaths::new(
                element.id.clone(),
                element.exe_path.clone(),
                element.lnk_path.clone(),
            );
            self.repositories
                .collection_repository()
                .upsert_collection_element_paths(&new_paths)
                .await?;
        }

        if let Some(install_time) = element.install_at {
            let new_install = NewCollectionElementInstall::new(element.id.clone(), install_time);
            self.repositories
                .collection_repository()
                .upsert_collection_element_install(&new_install)
                .await?;
        }

        Ok(())
    }
    pub async fn upsert_collection_element_thumbnail_size(
        &self,
        handle: &Arc<AppHandle>,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        let thumbnail_path = get_thumbnail_path(handle, id);
        if !std::path::Path::new(&thumbnail_path).exists() {
            return Ok(());
        }
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
        source: &Vec<ScannedGameElement>,
    ) -> anyhow::Result<()> {
        for element in source.iter() {
            self.create_collection_element(element).await?;
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

        let paths = self
            .repositories
            .collection_repository()
            .get_element_paths_by_element_id(id)
            .await?;

        if let Some(paths) = paths {
            let icon_path = if let Some(lnk_path) = paths.lnk_path {
                use crate::domain::file::get_lnk_metadatas;
                let metadatas = get_lnk_metadatas(vec![lnk_path.as_str()])?;
                let metadata = metadatas
                    .get(lnk_path.as_str())
                    .ok_or(anyhow::anyhow!("metadata cannot get"))?;
                if metadata.icon.to_lowercase().ends_with("ico") {
                    metadata.icon.clone()
                } else {
                    metadata.path.clone()
                }
            } else if let Some(exe_path) = paths.exe_path {
                exe_path
            } else {
                eprintln!("lnk_path and exe_path are None");
                return Ok(());
            };
            use crate::domain::file::save_icon_to_png;
            save_icon_to_png(handle, &icon_path, id).await??;
        } else {
            eprintln!("No paths found for element {}", id.value);
        }
        Ok(())
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

    pub async fn get_not_registered_info_element_ids(
        &self,
    ) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        self.repositories
            .collection_repository()
            .get_not_registered_info_element_ids()
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
            .delete_collection_element(id)
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
