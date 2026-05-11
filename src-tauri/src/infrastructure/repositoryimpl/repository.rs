use derive_new::new;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::domain::{
    all_game_cache::AllGameCache, collection::CollectionElement, explored_cache::ExploredCache,
};

use super::driver::Db;
use super::screenshot::ScreenshotRepositoryImpl;

#[derive(new, Clone)]
pub struct RepositoryImpl<T> {
    pub pool: Db,
    _marker: PhantomData<T>,
}

#[derive(Clone)]
pub struct Repositories {
    collection_repository: RepositoryImpl<CollectionElement>,
    explored_cache_repository: RepositoryImpl<ExploredCache>,
    all_game_cache_repository: RepositoryImpl<AllGameCache>,
    screenshot_repository: ScreenshotRepositoryImpl,
}
use crate::domain::repository::repositories::RepositoriesExt;

impl RepositoriesExt for Repositories {
    type CollectionRepo = RepositoryImpl<CollectionElement>;
    type ExploredCacheRepo = RepositoryImpl<ExploredCache>;
    type AllGameCacheRepo = RepositoryImpl<AllGameCache>;
    type ScreenshotRepo = ScreenshotRepositoryImpl;

    fn collection_repository(&self) -> &Self::CollectionRepo {
        &self.collection_repository
    }
    fn explored_cache_repository(&self) -> &Self::ExploredCacheRepo {
        &self.explored_cache_repository
    }
    fn all_game_cache_repository(&self) -> &Self::AllGameCacheRepo {
        &self.all_game_cache_repository
    }
    fn screenshot_repository(&self) -> &Self::ScreenshotRepo {
        &self.screenshot_repository
    }
}

impl Repositories {
    pub fn new(db: Db) -> Self {
        let collection_repository = RepositoryImpl::new(db.clone());
        let explored_cache_repository = RepositoryImpl::new(db.clone());
        let all_game_cache_repository = RepositoryImpl::new(db.clone());
        let screenshot_repository = ScreenshotRepositoryImpl::new(Arc::new(db.clone()));

        Self {
            collection_repository,
            explored_cache_repository,
            all_game_cache_repository,
            screenshot_repository,
        }
    }
}
