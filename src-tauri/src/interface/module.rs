use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::{
    infrastructure::{
        explorerimpl::explorer::{Explorers, ExplorersExt},
        repositoryimpl::{
            driver::Db,
            repository::{Repositories, RepositoriesExt},
        },
        util::get_shaders_abs_dir,
        windowsimpl::windows::{Windows, WindowsExt},
    },
    usecase::{
        all_game_cache::AllGameCacheUseCase, collection::CollectionUseCase,
        explored_cache::ExploredCacheUseCase, file::FileUseCase, pause_manager::PauseManager,
        process::ProcessUseCase, scaling::ScalingUseCase,
    },
};

pub struct Modules {
    collection_use_case: CollectionUseCase<Repositories>,
    explored_cache_use_case: ExploredCacheUseCase<Repositories>,

    file_use_case: FileUseCase<Explorers>,
    all_game_cache_use_case: AllGameCacheUseCase<Repositories>,
    process_use_case: ProcessUseCase<Windows>,
    pause_manager: PauseManager,
    scaling_use_case: ScalingUseCase,
}
pub trait ModulesExt {
    type Repositories: RepositoriesExt;
    type Explorers: ExplorersExt;
    type Windows: WindowsExt;

    fn collection_use_case(&self) -> &CollectionUseCase<Self::Repositories>;
    fn explored_cache_use_case(&self) -> &ExploredCacheUseCase<Self::Repositories>;
    fn all_game_cache_use_case(&self) -> &AllGameCacheUseCase<Self::Repositories>;

    fn file_use_case(&self) -> &FileUseCase<Self::Explorers>;
    fn process_use_case(&self) -> &ProcessUseCase<Self::Windows>;
    fn scaling_use_case(&self) -> &ScalingUseCase;
    fn pause_manager(&self) -> &PauseManager;
}

impl ModulesExt for Modules {
    type Repositories = Repositories;
    type Explorers = Explorers;
    type Windows = Windows;

    fn collection_use_case(&self) -> &CollectionUseCase<Self::Repositories> {
        &self.collection_use_case
    }
    fn explored_cache_use_case(&self) -> &ExploredCacheUseCase<Self::Repositories> {
        &self.explored_cache_use_case
    }
    fn all_game_cache_use_case(&self) -> &AllGameCacheUseCase<Self::Repositories> {
        &self.all_game_cache_use_case
    }

    fn file_use_case(&self) -> &FileUseCase<Self::Explorers> {
        &self.file_use_case
    }
    fn process_use_case(&self) -> &ProcessUseCase<Self::Windows> {
        &self.process_use_case
    }
    fn scaling_use_case(&self) -> &ScalingUseCase {
        &self.scaling_use_case
    }
    fn pause_manager(&self) -> &PauseManager {
        &self.pause_manager
    }
}

impl Modules {
    pub async fn new(handle: &AppHandle) -> Self {
        let db = Db::new(handle).await;

        let repositories = Arc::new(Repositories::new(db.clone()));
        let explorers = Arc::new(Explorers::new());
        let windows = Arc::new(Windows::new());

        let explored_cache_use_case = ExploredCacheUseCase::new(repositories.clone());
        let all_game_cache_use_case: AllGameCacheUseCase<Repositories> =
            AllGameCacheUseCase::new(repositories.clone());

        let file_use_case: FileUseCase<Explorers> = FileUseCase::new(explorers.clone());

        let process_use_case: ProcessUseCase<Windows> = ProcessUseCase::new(windows.clone());
        let pause_manager = PauseManager::new();
        let screenshot_watcher = crate::usecase::screenshot_watcher::ScreenshotWatcher::new(
            repositories.clone(),
            Arc::new(std::sync::Mutex::new(None)),
        );

        let collection_use_case = CollectionUseCase::new(
            repositories.clone(),
            Arc::new(pause_manager.clone()),
            Arc::new(screenshot_watcher.clone()),
        );

        let shader_dir = get_shaders_abs_dir(handle);
        let scaling_use_case = ScalingUseCase::new(shader_dir.clone());

        // Define a recursive copy helper
        fn copy_recursively(
            source: &std::path::Path,
            destination: &std::path::Path,
        ) -> std::io::Result<()> {
            if source.is_dir() {
                if !destination.exists() {
                    std::fs::create_dir_all(destination)?;
                }
                for entry in std::fs::read_dir(source)? {
                    let entry = entry?;
                    let file_type = entry.file_type()?;
                    let dest_path = destination.join(entry.file_name());
                    if file_type.is_dir() {
                        copy_recursively(&entry.path(), &dest_path)?;
                    } else if !dest_path.exists() {
                        std::fs::copy(entry.path(), &dest_path)?;
                    }
                }
            } else if !destination.exists() {
                // Should be covered by directory iteration, but handle single file case just in case
                if let Some(parent) = destination.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent)?;
                    }
                }
                std::fs::copy(source, destination)?;
            }
            Ok(())
        }

        // Resolve bundled shaders path based on environment
        let bundled_shaders_path = {
            #[cfg(debug_assertions)]
            {
                let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                path.push("src/infrastructure/windowsimpl/scaling/shaders");
                Some(path)
            }
            #[cfg(not(debug_assertions))]
            {
                handle
                    .path()
                    .resource_dir()
                    .ok()
                    .map(|dir| dir.join("src/infrastructure/windowsimpl/scaling/shaders"))
            }
        };

        if let Some(path) = bundled_shaders_path {
            if path.exists() {
                let dest_root = std::path::PathBuf::from(&shader_dir);
                let _ = copy_recursively(&path, &dest_root);
            }
        }

        Self {
            collection_use_case,
            explored_cache_use_case,
            all_game_cache_use_case,

            file_use_case,
            process_use_case,
            pause_manager,
            scaling_use_case,
        }
    }
}
