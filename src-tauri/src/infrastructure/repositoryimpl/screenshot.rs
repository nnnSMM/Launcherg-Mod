use async_trait::async_trait;
use chrono::{Local, NaiveDateTime};
use sqlx::{query, Row};
use std::sync::Arc;

use super::driver::Db;
use crate::domain::{
    collection::CollectionElement,
    repository::screenshot::{Screenshot, ScreenshotRepository},
    Id,
};

#[derive(Clone)]
pub struct ScreenshotRepositoryImpl {
    db: Arc<Db>,
}

impl ScreenshotRepositoryImpl {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }
    fn map_row(row: sqlx::sqlite::SqliteRow) -> Screenshot {
        use sqlx::Row;
        let created_at_str: String = row
            .get::<Option<String>, _>("created_at")
            .unwrap_or_default();

        // Convert UTC timestamp to JST
        let created_at_jst = if let Ok(naive_dt) =
            NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S")
        {
            naive_dt.and_utc().with_timezone(&Local).to_rfc3339()
        } else {
            created_at_str.replace(" ", "T")
        };

        Screenshot {
            id: row.get::<i64, _>("id") as i32,
            game_id: row.get::<i64, _>("game_id") as i32,
            filename: row.get("filename"),
            order_index: row.get::<i64, _>("order_index") as i32,
            created_at: created_at_jst,
        }
    }
}

#[async_trait]
impl ScreenshotRepository for ScreenshotRepositoryImpl {
    async fn get_by_game_id(
        &self,
        game_id: &Id<CollectionElement>,
    ) -> anyhow::Result<Vec<Screenshot>> {
        let pool = self.db.0.clone();
        let screenshots = query(
            r#"
            SELECT id, game_id, filename, order_index, created_at
            FROM screenshots
            WHERE game_id = ?
            ORDER BY order_index DESC
            "#,
        )
        .bind(game_id.value)
        .fetch_all(&*pool)
        .await?;

        Ok(screenshots.into_iter().map(Self::map_row).collect())
    }

    async fn get_by_id(&self, id: i32) -> anyhow::Result<Option<Screenshot>> {
        let pool = self.db.0.clone();
        let screenshot = query(
            r#"
            SELECT id, game_id, filename, order_index, created_at
            FROM screenshots
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&*pool)
        .await?;

        Ok(screenshot.map(Self::map_row))
    }

    async fn insert(&self, game_id: &Id<CollectionElement>, filename: &str) -> anyhow::Result<()> {
        let pool = self.db.0.clone();

        // Get the max order_index for this game
        let max_order: Option<i64> = query(
            r#"
            SELECT MAX(order_index) as max_order
            FROM screenshots
            WHERE game_id = ?
            "#,
        )
        .bind(game_id.value)
        .fetch_one(&*pool)
        .await?
        .get("max_order");

        let new_order = max_order.unwrap_or(-1) + 1;

        query(
            r#"
            INSERT INTO screenshots (game_id, filename, order_index)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(game_id.value)
        .bind(filename)
        .bind(new_order)
        .execute(&*pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        let pool = self.db.0.clone();
        query(
            r#"
            DELETE FROM screenshots
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&*pool)
        .await?;
        Ok(())
    }

    async fn delete_by_game_id(&self, game_id: &Id<CollectionElement>) -> anyhow::Result<()> {
        let pool = self.db.0.clone();
        query(
            r#"
            DELETE FROM screenshots
            WHERE game_id = ?
            "#,
        )
        .bind(game_id.value)
        .execute(&*pool)
        .await?;
        Ok(())
    }

    async fn update_order(&self, id: i32, order_index: i32) -> anyhow::Result<()> {
        let pool = self.db.0.clone();
        query(
            r#"
            UPDATE screenshots
            SET order_index = ?
            WHERE id = ?
            "#,
        )
        .bind(order_index)
        .bind(id)
        .execute(&*pool)
        .await?;
        Ok(())
    }

    async fn get_all(&self) -> anyhow::Result<Vec<Screenshot>> {
        let pool = self.db.0.clone();
        let screenshots = query(
            r#"
            SELECT id, game_id, filename, order_index, created_at
            FROM screenshots
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&*pool)
        .await?;

        Ok(screenshots.into_iter().map(Self::map_row).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_db() -> Arc<Db> {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let pool = Arc::new(pool);

        // Create tables
        sqlx::query("CREATE TABLE collection_elements (id INTEGER PRIMARY KEY AUTOINCREMENT);")
            .execute(&*pool)
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE screenshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id INTEGER NOT NULL,
            filename TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            order_index INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (game_id) REFERENCES collection_elements(id) ON DELETE CASCADE
        );",
        )
        .execute(&*pool)
        .await
        .unwrap();

        Arc::new(Db(pool))
    }

    #[tokio::test]
    async fn test_get_all() {
        let db = setup_db().await;
        let repository = ScreenshotRepositoryImpl::new(db);

        // Mock data
        let game_id = Id::new(1);
        sqlx::query("INSERT INTO collection_elements (id) VALUES (1)")
            .execute(&*repository.db.0)
            .await
            .unwrap();

        repository.insert(&game_id, "test1.png").await.unwrap();
        repository.insert(&game_id, "test2.png").await.unwrap();

        // This method does not exist yet!
        let all = repository.get_all().await.unwrap();

        assert_eq!(all.len(), 2);
    }
}
