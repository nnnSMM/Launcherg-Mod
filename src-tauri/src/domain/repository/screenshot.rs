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
    pub thumbnail_filename: Option<String>,
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
    async fn delete_by_game_id(&self, game_id: &Id<CollectionElement>) -> anyhow::Result<()>;
    async fn update_order(&self, id: i32, order_index: i32) -> anyhow::Result<()>;
    async fn get_all(&self) -> anyhow::Result<Vec<Screenshot>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenshot_creation() {
        let screenshot = Screenshot {
            id: 1,
            game_id: 100,
            filename: "screenshot_001.png".to_string(),
            thumbnail_filename: None,
            order_index: 0,
            created_at: "2023-12-25T10:00:00".to_string(),
        };

        assert_eq!(screenshot.id, 1);
        assert_eq!(screenshot.game_id, 100);
        assert_eq!(screenshot.filename, "screenshot_001.png");
        assert_eq!(screenshot.order_index, 0);
    }

    #[test]
    fn test_screenshot_serialization() {
        let screenshot = Screenshot {
            id: 42,
            game_id: 999,
            filename: "test.png".to_string(),
            thumbnail_filename: None,
            order_index: 5,
            created_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&screenshot).unwrap();
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"gameId\":999")); // camelCase
        assert!(json.contains("\"filename\":\"test.png\""));
        assert!(json.contains("\"thumbnailFilename\":null"));
        assert!(json.contains("\"orderIndex\":5")); // camelCase
    }

    #[test]
    fn test_screenshot_clone() {
        let original = Screenshot {
            id: 1,
            game_id: 2,
            filename: "clone_test.png".to_string(),
            thumbnail_filename: None,
            order_index: 3,
            created_at: "2024-01-01".to_string(),
        };
        let cloned = original.clone();

        assert_eq!(original.id, cloned.id);
        assert_eq!(original.filename, cloned.filename);
    }
}
