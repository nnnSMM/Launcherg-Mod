use super::{
    all_game_cache::AllGameCacheRepository, collection::CollectionRepository,
    explored_cache::ExploredCacheRepository, screenshot::ScreenshotRepository,
};

pub trait RepositoriesExt {
    type CollectionRepo: CollectionRepository;
    type ExploredCacheRepo: ExploredCacheRepository;
    type AllGameCacheRepo: AllGameCacheRepository;
    type ScreenshotRepo: ScreenshotRepository;

    fn collection_repository(&self) -> &Self::CollectionRepo;
    fn explored_cache_repository(&self) -> &Self::ExploredCacheRepo;
    fn all_game_cache_repository(&self) -> &Self::AllGameCacheRepo;
    fn screenshot_repository(&self) -> &Self::ScreenshotRepo;
}
