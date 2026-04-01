use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::models::Example;

#[derive(Clone)]
pub struct ExampleRepository {
    pool: SqlitePool,
}

impl ExampleRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn add_example(
        &self,
        target_type: &str,
        target_id: i64,
        sentence: &str,
        source: Option<&str>,
    ) -> Result<Example> {
        let row = sqlx::query_as::<_, Example>(
            "INSERT INTO examples (target_type, target_id, sentence, source) VALUES (?, ?, ?, ?) RETURNING *",
        )
        .bind(target_type)
        .bind(target_id)
        .bind(sentence)
        .bind(source)
        .fetch_one(&self.pool)
        .await
        .context("Failed to add example")?;
        Ok(row)
    }

    pub async fn get_examples(&self, target_type: &str, target_id: i64) -> Result<Vec<Example>> {
        let rows = sqlx::query_as::<_, Example>(
            "SELECT * FROM examples WHERE target_type = ? AND target_id = ?",
        )
        .bind(target_type)
        .bind(target_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn delete_examples(&self, target_type: &str, target_id: i64) -> Result<u64> {
        let result = sqlx::query("DELETE FROM examples WHERE target_type = ? AND target_id = ?")
            .bind(target_type)
            .bind(target_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}
