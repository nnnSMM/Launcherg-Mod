use crate::domain::{
    collection::{CollectionElement, NewCollectionElement, NewCollectionElementDetail},
    Id,
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Local, NaiveDate};

#[derive(Debug, Clone)]
pub struct VndbScreenshotCache {
    pub collection_element_id: i32,
    pub vndb_id: Option<String>,
    pub matched_title: Option<String>,
    pub screenshots_json: String,
    pub fetched_at: String,
    pub status: String,
}

#[async_trait]
pub trait CollectionRepository {
    async fn get_all_elements(&self) -> Result<Vec<CollectionElement>>;
    async fn get_element_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElement>>;
    async fn upsert_collection_element(&self, new_elements: &NewCollectionElement) -> Result<()>;
    async fn upsert_collection_element_thumbnail_size(
        &self,
        id: &Id<CollectionElement>,
        width: i32,
        height: i32,
    ) -> Result<()>;
    async fn get_null_thumbnail_size_element_ids(&self) -> Result<Vec<Id<CollectionElement>>>;
    #[allow(dead_code)]
    async fn remove_conflict_maps(&self) -> Result<()>;
    #[allow(dead_code)]
    async fn delete_collection_element(&self, element_id: &Id<CollectionElement>) -> Result<()>;

    #[allow(dead_code)]
    async fn get_not_registered_detail_element_ids(&self) -> Result<Vec<Id<CollectionElement>>>;
    #[allow(dead_code)]
    async fn create_element_details(&self, details: Vec<NewCollectionElementDetail>) -> Result<()>;
    #[allow(dead_code)]
    async fn get_brandname_and_rubies(&self) -> Result<Vec<(String, String)>>;

    #[allow(dead_code)]
    async fn get_element_ids_by_is_nukige(
        &self,
        is_nukige: bool,
    ) -> Result<Vec<Id<CollectionElement>>>;
    #[allow(dead_code)]
    async fn get_element_ids_by_install_at_not_null(&self) -> Result<Vec<Id<CollectionElement>>>;
    #[allow(dead_code)]
    async fn get_element_ids_by_brandnames(
        &self,
        brandnames: &Vec<String>,
    ) -> Result<Vec<Id<CollectionElement>>>;
    #[allow(dead_code)]
    async fn get_element_ids_by_sellday(
        &self,
        since: &str,
        until: &str,
    ) -> Result<Vec<Id<CollectionElement>>>;

    async fn update_element_last_play_at_by_id(
        &self,
        id: &Id<CollectionElement>,
        last_play_at: DateTime<Local>,
    ) -> Result<()>;
    async fn update_element_first_play_at_if_null_by_id(
        &self,
        id: &Id<CollectionElement>,
        first_play_at: DateTime<Local>,
    ) -> Result<()>;
    async fn update_element_like_at_by_id(
        &self,
        id: &Id<CollectionElement>,
        like_at: Option<DateTime<Local>>,
    ) -> Result<()>;
    async fn update_element_play_status_by_id(
        &self,
        id: &Id<CollectionElement>,
        play_status: i32,
    ) -> Result<()>;
    async fn add_play_time_seconds(&self, id: &Id<CollectionElement>, seconds: i32) -> Result<()>;
    async fn add_daily_play_time_seconds(
        &self,
        id: &Id<CollectionElement>,
        play_date: NaiveDate,
        seconds: i32,
    ) -> Result<()>;
    async fn subtract_daily_play_time_seconds_from_latest(
        &self,
        id: &Id<CollectionElement>,
        seconds: i32,
    ) -> Result<()>;

    #[allow(dead_code)]
    async fn delete_element_by_id(&self, id: &Id<CollectionElement>) -> Result<()>;

    async fn touch(&self, id: &Id<CollectionElement>) -> Result<()>;

    async fn get_app_setting(&self, key: String) -> Result<Option<String>>;
    async fn set_app_setting(&self, key: String, value: Option<String>) -> Result<()>;

    async fn get_vndb_screenshot_cache(
        &self,
        collection_element_id: i32,
    ) -> Result<Option<VndbScreenshotCache>>;
    async fn upsert_vndb_screenshot_cache(&self, cache: VndbScreenshotCache) -> Result<()>;

    async fn update_collection_element_path(
        &self,
        id: &Id<CollectionElement>,
        exe_path: Option<String>,
        lnk_path: Option<String>,
    ) -> Result<()>;
}
