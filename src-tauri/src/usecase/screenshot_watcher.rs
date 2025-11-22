use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use derive_new::new;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::AppHandle;

use crate::domain::repository::screenshot::ScreenshotRepository;
use crate::domain::Id;
use crate::infrastructure::repositoryimpl::repository::RepositoriesExt;

#[derive(new, Clone)]
pub struct ScreenshotWatcher<R: RepositoriesExt> {
    repositories: Arc<R>,
    watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
}

impl<R: RepositoriesExt + Send + Sync + 'static> ScreenshotWatcher<R> {
    pub fn start_watching(&self, handle: Arc<AppHandle>, game_id: i32) -> anyhow::Result<()> {
        let repositories = self.repositories.clone();
        let handle = handle.clone();
        let game_id = Id::new(game_id);

        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        let user_profile = std::env::var("USERPROFILE")?;

        // Try multiple possible screenshot directories
        let screenshot_paths: Vec<PathBuf> = vec![
            // OneDrive paths (Japanese)
            Path::new(&user_profile)
                .join("OneDrive")
                .join("画像")
                .join("スクリーンショット"),
            // OneDrive paths (English)
            Path::new(&user_profile)
                .join("OneDrive")
                .join("Pictures")
                .join("Screenshots"),
            // Standard Windows paths (Japanese)
            Path::new(&user_profile)
                .join("ピクチャ")
                .join("スクリーンショット"),
            Path::new(&user_profile)
                .join("画像")
                .join("スクリーンショット"),
            // Standard Windows paths (English)
            Path::new(&user_profile)
                .join("Pictures")
                .join("Screenshots"),
        ];

        let mut screenshots_dir = None;
        for path in screenshot_paths {
            if path.exists() {
                screenshots_dir = Some(path);
                break;
            }
        }

        let screenshots_dir = match screenshots_dir {
            Some(dir) => dir,
            None => {
                // If no directory exists, create the standard one
                let default_dir = Path::new(&user_profile)
                    .join("Pictures")
                    .join("Screenshots");
                std::fs::create_dir_all(&default_dir)?;
                default_dir
            }
        };

        watcher.watch(&screenshots_dir, RecursiveMode::NonRecursive)?;

        *self.watcher.lock().unwrap() = Some(watcher);

        tauri::async_runtime::spawn(async move {
            for res in rx {
                match res {
                    Ok(event) => {
                        if let Event {
                            kind: notify::EventKind::Create(_),
                            paths,
                            ..
                        } = event
                        {
                            for path in paths {
                                if let Some(extension) = path.extension() {
                                    if extension == "png" {
                                        // Wait a bit for the file to be fully written
                                        tokio::time::sleep(Duration::from_secs(1)).await;

                                        // Copy to game folder
                                        if let Err(e) =
                                            copy_screenshot(&handle, &repositories, &game_id, &path)
                                                .await
                                        {
                                            eprintln!("[screenshot_watcher] Failed to copy screenshot: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("[screenshot_watcher] watch error: {:?}", e),
                }
            }
        });

        Ok(())
    }

    pub fn stop_watching(&self) {
        let mut watcher = self.watcher.lock().unwrap();
        *watcher = None;
    }
}

async fn copy_screenshot<R: RepositoriesExt>(
    handle: &Arc<AppHandle>,
    repositories: &Arc<R>,
    game_id: &Id<crate::domain::collection::CollectionElement>,
    src_path: &Path,
) -> anyhow::Result<()> {
    use crate::infrastructure::util::get_save_root_abs_dir;

    let filename = src_path
        .file_name()
        .ok_or(anyhow::anyhow!("No filename"))?
        .to_string_lossy()
        .to_string();

    let dest_dir = Path::new(&get_save_root_abs_dir(handle))
        .join("game-memos")
        .join(game_id.value.to_string());

    if !dest_dir.exists() {
        std::fs::create_dir_all(&dest_dir)?;
    }

    let dest_path = dest_dir.join(&filename);
    std::fs::copy(src_path, &dest_path)?;

    // Insert into DB
    repositories
        .screenshot_repository()
        .insert(game_id, &filename)
        .await?;

    Ok(())
}
