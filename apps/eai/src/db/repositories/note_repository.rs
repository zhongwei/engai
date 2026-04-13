use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::models::Note;

#[derive(Clone)]
pub struct NoteRepository {
    pool: SqlitePool,
}

impl NoteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn add_note(
        &self,
        target_type: &str,
        target_id: i64,
        content: &str,
    ) -> Result<Note> {
        let row = sqlx::query_as::<_, Note>(
            "INSERT INTO notes (target_type, target_id, content) VALUES (?, ?, ?) RETURNING *",
        )
        .bind(target_type)
        .bind(target_id)
        .bind(content)
        .fetch_one(&self.pool)
        .await
        .context("Failed to add note")?;
        Ok(row)
    }

    pub async fn get_notes(&self, target_type: &str, target_id: i64) -> Result<Vec<Note>> {
        let rows = sqlx::query_as::<_, Note>(
            "SELECT * FROM notes WHERE target_type = ? AND target_id = ? ORDER BY created_at DESC",
        )
        .bind(target_type)
        .bind(target_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn delete_note(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM notes WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
