use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::models::ChatEntry;

#[derive(Clone)]
pub struct ChatRepository {
    pool: SqlitePool,
}

impl ChatRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn add_chat_message(&self, role: &str, content: &str) -> Result<ChatEntry> {
        let row = sqlx::query_as::<_, ChatEntry>(
            "INSERT INTO chat_history (role, content) VALUES (?, ?) RETURNING *",
        )
        .bind(role)
        .bind(content)
        .fetch_one(&self.pool)
        .await
        .context("Failed to add chat message")?;
        Ok(row)
    }

    pub async fn get_recent_chat(&self, limit: i64) -> Result<Vec<ChatEntry>> {
        let rows = sqlx::query_as::<_, ChatEntry>(
            "SELECT * FROM chat_history ORDER BY created_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn clear_chat(&self) -> Result<u64> {
        let result = sqlx::query("DELETE FROM chat_history")
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}
