use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::models::Reading;

#[derive(Clone)]
pub struct ReadingRepository {
    pool: SqlitePool,
}

impl ReadingRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn add_reading(
        &self,
        title: Option<&str>,
        content: &str,
        source: Option<&str>,
    ) -> Result<Reading> {
        let row = sqlx::query_as::<_, Reading>(
            "INSERT INTO readings (title, content, source) VALUES (?, ?, ?) RETURNING *",
        )
        .bind(title)
        .bind(content)
        .bind(source)
        .fetch_one(&self.pool)
        .await
        .context("Failed to add reading")?;
        Ok(row)
    }

    pub async fn get_reading(&self, id: i64) -> Result<Option<Reading>> {
        let row = sqlx::query_as::<_, Reading>("SELECT * FROM readings WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    pub async fn list_readings(&self, limit: i64, offset: i64) -> Result<Vec<Reading>> {
        let rows = sqlx::query_as::<_, Reading>(
            "SELECT * FROM readings ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn delete_reading(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM readings WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
