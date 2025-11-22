use crate::domain::collection::CollectionElement;
use crate::domain::Id;
use async_trait::async_trait;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Screenshot {
    pub id: i32,
    pub game_id: i32,
    pub filename: String,
    pub order_index: i32,
    pub created_at: String,
}

#[async_trait]
pub trait ScreenshotRepository {
    async fn get_by_game_id(
        &self,
        game_id: &Id<CollectionElement>,
    ) -> anyhow::Result<Vec<Screenshot>>;
    async fn get_by_id(&self, id: i32) -> anyhow::Result<Option<Screenshot>>;
    async fn insert(&self, game_id: &Id<CollectionElement>, filename: &str) -> anyhow::Result<()>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
    async fn update_order(&self, id: i32, order_index: i32) -> anyhow::Result<()>;
}
