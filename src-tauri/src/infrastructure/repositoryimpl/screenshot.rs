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
            ORDER BY order_index ASC
            "#,
        )
        .bind(game_id.value)
        .fetch_all(&*pool)
        .await?;

        Ok(screenshots
            .into_iter()
            .map(|row| {
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
            })
            .collect())
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

        Ok(screenshot.map(|row| {
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
        }))
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
}
