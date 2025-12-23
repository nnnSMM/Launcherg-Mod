use std::{fs, sync::Arc};

use chrono::Local;
use derive_new::new;
use tauri::AppHandle;

use tokio::time::Duration;

use super::error::UseCaseError;
use super::game_tracker::{
    is_path_related_error, launch_game, GameProcessMonitor, ProcessSearchConfig,
};
use super::pause_manager::PauseManager;
use crate::{
    domain::{
        collection::{CollectionElement, NewCollectionElement, NewCollectionElementDetail},
        file::{
            get_icon_path, get_lnk_metadatas, get_thumbnail_path, save_icon_to_png, save_thumbnail,
        },
        repository::collection::CollectionRepository,
        repository::screenshot::{Screenshot, ScreenshotRepository},
        Id,
    },
    infrastructure::{repositoryimpl::repository::RepositoriesExt, util::get_save_root_abs_dir},
};

#[derive(new)]
pub struct CollectionUseCase<R: RepositoriesExt> {
    repositories: Arc<R>,
    pause_manager: Arc<PauseManager>,
    screenshot_watcher: Arc<crate::usecase::screenshot_watcher::ScreenshotWatcher<R>>,
}

impl<R: RepositoriesExt + Send + Sync + 'static> CollectionUseCase<R> {
    /// ゲームを起動し、プレイ時間を追跡する
    ///
    /// この関数は以下の処理を行います:
    /// 1. ゲームの起動
    /// 2. ゲームプロセスの検索
    /// 3. プレイ時間の記録
    /// 4. スクリーンショットの監視
    pub async fn play_game_and_track(
        &self,
        handle: Arc<AppHandle>,
        element_id: i32,
    ) -> anyhow::Result<()> {
        let element = self.get_element_by_element_id(&Id::new(element_id)).await?;

        self.update_element_last_play_at(&Id::new(element_id))
            .await?;

        // ゲームを起動
        let launch_result = match launch_game(&element) {
            Ok(result) => result,
            Err(e) => {
                // パス関連のエラーの場合、未インストール状態にする
                if let Some(io_error) = e.downcast_ref::<std::io::Error>() {
                    if is_path_related_error(io_error) {
                        self.delete_collection_element_logical(&Id::new(element_id))
                            .await?;
                    }
                }
                return Err(e);
            }
        };

        let game_name = element.gamename.clone();
        let path_str = launch_result.path_str.clone();
        let spawned_pid = launch_result.spawned_pid;
        let repositories = self.repositories.clone();
        let pause_manager = self.pause_manager.clone();
        let screenshot_watcher = self.screenshot_watcher.clone();

        tauri::async_runtime::spawn(async move {
            // 追跡状態を開始
            pause_manager.set_tracking(true);

            let monitor = GameProcessMonitor::new(
                handle.clone(),
                repositories,
                pause_manager,
                screenshot_watcher,
                element_id,
            );

            // ポーズショートカットを登録
            let registered_shortcut = monitor.register_pause_shortcut().await;

            // ランチャーがゲーム本体を起動するまで1秒待つ
            tokio::time::sleep(Duration::from_secs(1)).await;

            // ゲームプロセスを検索
            let config = ProcessSearchConfig::default();
            if let Some(pid) = monitor
                .find_game_process(&path_str, spawned_pid, &game_name, &config)
                .await
            {
                // プロセスを監視
                monitor.monitor_process(pid, registered_shortcut).await;
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
    pub async fn concurrency_upsert_collection_element_thumbnail_size(
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

    pub async fn concurrency_save_thumbnails(
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
        handle: &Arc<AppHandle>,
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

        // Delete icon
        let icon_path = get_icon_path(handle, id);
        if std::path::Path::new(&icon_path).exists() {
            let _ = std::fs::remove_file(icon_path);
        }

        // Delete thumbnail
        let thumbnail_path = get_thumbnail_path(handle, id);
        if std::path::Path::new(&thumbnail_path).exists() {
            let _ = std::fs::remove_file(thumbnail_path);
        }

        // Delete play history
        let play_history_path = crate::domain::file::get_play_history_path(handle, id);
        if std::path::Path::new(&play_history_path).exists() {
            let _ = std::fs::remove_file(play_history_path);
        }

        // Delete screenshots directory
        let root_dir = get_save_root_abs_dir(handle);
        let game_screenshot_dir = std::path::Path::new(&root_dir)
            .join("game-memos")
            .join(id.value.to_string());
        if game_screenshot_dir.exists() {
            let _ = std::fs::remove_dir_all(game_screenshot_dir);
        }

        // Delete screenshots from DB
        self.repositories
            .screenshot_repository()
            .delete_by_game_id(id)
            .await?;

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

    pub async fn get_all_elements(
        &self,
        handle: &Arc<AppHandle>,
    ) -> anyhow::Result<Vec<CollectionElement>> {
        let null_size_ids = self
            .repositories
            .collection_repository()
            .get_null_thumbnail_size_element_ids()
            .await?;
        self.concurrency_upsert_collection_element_thumbnail_size(handle, null_size_ids)
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

    pub async fn get_game_screenshots(
        &self,
        handle: &Arc<AppHandle>,
        game_id: i32,
    ) -> anyhow::Result<Vec<Screenshot>> {
        let screenshots = self
            .repositories
            .screenshot_repository()
            .get_by_game_id(&Id::new(game_id))
            .await?;

        let root_dir = get_save_root_abs_dir(handle);
        let game_dir = std::path::Path::new(&root_dir)
            .join("game-memos")
            .join(game_id.to_string());
        Ok(screenshots
            .into_iter()
            .map(|mut s| {
                s.filename = game_dir.join(&s.filename).to_string_lossy().to_string();
                s
            })
            .collect())
    }

    pub async fn import_screenshot(
        &self,
        handle: &Arc<AppHandle>,
        game_id: i32,
        file_path: String,
    ) -> anyhow::Result<()> {
        let path = std::path::Path::new(&file_path);
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found"));
        }

        let filename = path
            .file_name()
            .ok_or(anyhow::anyhow!("Invalid filename"))?
            .to_string_lossy()
            .to_string();

        let dest_dir = std::path::Path::new(&get_save_root_abs_dir(handle))
            .join("game-memos")
            .join(game_id.to_string());

        if !dest_dir.exists() {
            std::fs::create_dir_all(&dest_dir)?;
        }

        let dest_path = dest_dir.join(&filename);
        std::fs::copy(path, &dest_path)?;

        self.repositories
            .screenshot_repository()
            .insert(&Id::new(game_id), &filename)
            .await?;

        Ok(())
    }

    pub async fn delete_screenshot(
        &self,
        handle: &Arc<AppHandle>,
        screenshot_id: i32,
    ) -> anyhow::Result<()> {
        // Get screenshot details
        let screenshot = self
            .repositories
            .screenshot_repository()
            .get_by_id(screenshot_id)
            .await?
            .ok_or(anyhow::anyhow!("Screenshot not found"))?;

        // Delete from DB
        self.repositories
            .screenshot_repository()
            .delete(screenshot_id)
            .await?;

        // Delete file
        let file_path = std::path::Path::new(&get_save_root_abs_dir(handle))
            .join("game-memos")
            .join(screenshot.game_id.to_string())
            .join(screenshot.filename);

        if file_path.exists() {
            std::fs::remove_file(file_path)?;
        }

        Ok(())
    }

    pub async fn update_screenshots_order(
        &self,
        updates: Vec<(i32, i32)>, // Vec<(id, order_index)>
    ) -> anyhow::Result<()> {
        for (id, order_index) in updates {
            self.repositories
                .screenshot_repository()
                .update_order(id, order_index)
                .await?;
        }
        Ok(())
    }

    pub async fn update_collection_element_path(
        &self,
        id: &Id<CollectionElement>,
        path: String,
    ) -> anyhow::Result<()> {
        let is_lnk = path.to_lowercase().ends_with(".lnk");
        let (exe_path, lnk_path) = if is_lnk {
            (None, Some(path))
        } else {
            (Some(path), None)
        };
        self.repositories
            .collection_repository()
            .update_collection_element_path(id, exe_path, lnk_path)
            .await
    }

    pub async fn delete_collection_element_logical(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .update_collection_element_path(id, None, None)
            .await
    }
}
