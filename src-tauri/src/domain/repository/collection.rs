use crate::domain::{
    collection::{CollectionElement, NewCollectionElement, NewCollectionElementDetail},
    Id,
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Local};

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
    async fn remove_conflict_maps(&self) -> Result<()>;
    async fn delete_collection_element(&self, element_id: &Id<CollectionElement>) -> Result<()>;

    async fn get_not_registered_detail_element_ids(&self) -> Result<Vec<Id<CollectionElement>>>;
    async fn create_element_details(&self, details: Vec<NewCollectionElementDetail>) -> Result<()>;
    async fn get_brandname_and_rubies(&self) -> Result<Vec<(String, String)>>;

    async fn get_element_ids_by_is_nukige(
        &self,
        is_nukige: bool,
    ) -> Result<Vec<Id<CollectionElement>>>;
    async fn get_element_ids_by_install_at_not_null(&self) -> Result<Vec<Id<CollectionElement>>>;
    async fn get_element_ids_by_brandnames(
        &self,
        brandnames: &Vec<String>,
    ) -> Result<Vec<Id<CollectionElement>>>;
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
    async fn add_play_time_seconds(
        &self,
        id: &Id<CollectionElement>,
        seconds: i32,
    ) -> Result<()>;

    async fn delete_element_by_id(&self, id: &Id<CollectionElement>) -> Result<()>;
}
