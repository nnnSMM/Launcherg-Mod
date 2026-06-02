use std::{path::Path, sync::Arc};

use anyhow::{Context, Result};
use refinery::config::{Config, ConfigDbType};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};
use tauri::AppHandle;

use crate::infrastructure::util::get_save_root_abs_dir;

#[derive(Clone)]
pub struct Db(pub(crate) Arc<Pool<Sqlite>>);

const DB_FILE: &str = "launcherg_sqlite.db3";

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./src/migrations");
}

impl Db {
    pub async fn new(handle: &AppHandle) -> Result<Db> {
        let root = get_save_root_abs_dir(handle);
        let db_path = Path::new(&root).join(Path::new(DB_FILE));
        let db_filename = db_path.to_string_lossy().to_string();
        let pool = SqlitePoolOptions::new()
            .max_connections(256)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(&db_path)
                    .create_if_missing(true)
                    .foreign_keys(true),
            )
            .await
            .with_context(|| format!("Failed to connect SQLite database: {db_filename}"))?;

        // migrate
        let mut conf = Config::new(ConfigDbType::Sqlite).set_db_path(&db_filename);
        embedded::migrations::runner()
            .set_abort_divergent(false)
            .set_abort_missing(false)
            .run(&mut conf)
            .with_context(|| format!("Failed to migrate SQLite database: {db_filename}"))?;

        Ok(Db(Arc::new(pool)))
    }
}
