use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::{env, fs};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct DataBase {
    pub pool: SqlitePool,
}

impl DataBase {
    pub async fn new() -> Result<Self> {
        let path = env::var("DB_PATH").expect("DB_PATH missing");
        let path_obj = std::path::Path::new(&path);

        if let Some(parent) = path_obj.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        if !path_obj.exists() {
            fs::File::create(path_obj).expect("Failed to create DB file");
        }

        let database_url = format!("sqlite://{}", path);
        let pool = SqlitePool::connect(&database_url)
            .await
            .expect("Failed to connect to SQLite");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS locations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user VARCHAR(128) NOT NULL,
                host VARCHAR(16) NOT NULL,
                begin_at TEXT NOT NULL,
                end_at TEXT
            );
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                user VARCHAR(128) PRIMARY KEY,
                last_request_at INTEGER NOT NULL
            );
            "#,
        )
        .execute(&pool)
        .await?;

        debug!("DataBase successfuly open.");

        Ok(Self { pool })
    }

    pub async fn latest_begin(&self, user: &str) -> Result<Option<String>> {
        let data: Option<(String,)> = sqlx::query_as(
            "SELECT begin_at FROM locations WHERE user = ? ORDER BY begin_at DESC LIMIT 1",
        )
        .bind(user)
        .fetch_optional(&self.pool)
        .await?;

        Ok(data.map(|r| r.0))
    }

    pub async fn insert_location(
        &self,
        user: &str,
        host: &str,
        begin_at: &str,
        end_at: Option<String>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO locations (user, host, begin_at, end_at)
                 VALUES (?, ?, ?, ?)",
        )
        .bind(user)
        .bind(host)
        .bind(begin_at)
        .bind(end_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_locations(&self, user: &str) -> Result<Vec<(String, String, Option<String>)>> {
        Ok(sqlx::query_as::<_, (String, String, Option<String>)>(
            "SELECT host, begin_at, end_at FROM locations WHERE user = ?",
        )
        .bind(user)
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn can_request(&self, user: &str) -> Result<bool> {
        let now = Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            UPDATE users
            SET last_request_at = ?
            WHERE user = ?
            AND last_request_at <= ? - 30
            "#
        )
        .bind(now)
        .bind(user)
        .bind(now)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 1 {
            return Ok(true);
        }

        let insert_result = sqlx::query(
            r#"
            INSERT OR IGNORE INTO users (user, last_request_at)
            VALUES (?, ?)
            "#
        )
        .bind(user)
        .bind(now)
        .execute(&self.pool)
        .await?;

        if insert_result.rows_affected() == 1 {
            return Ok(true);
        }

        Ok(false)
    }
}
