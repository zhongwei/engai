use anyhow::{Context, Result};
use chrono::Local;
use sqlx::SqlitePool;

use crate::models::Review;

#[derive(Clone)]
pub struct ReviewRepository {
    pool: SqlitePool,
}

impl ReviewRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn add_review(
        &self,
        target_type: &str,
        target_id: i64,
        quality: i32,
    ) -> Result<Review> {
        let row = sqlx::query_as::<_, Review>(
            "INSERT INTO reviews (target_type, target_id, quality) VALUES (?, ?, ?) RETURNING *",
        )
        .bind(target_type)
        .bind(target_id)
        .bind(quality)
        .fetch_one(&self.pool)
        .await
        .context("Failed to add review")?;
        Ok(row)
    }

    pub async fn get_reviews(&self, target_type: &str, target_id: i64) -> Result<Vec<Review>> {
        let rows = sqlx::query_as::<_, Review>(
            "SELECT * FROM reviews WHERE target_type = ? AND target_id = ? ORDER BY reviewed_at DESC",
        )
        .bind(target_type)
        .bind(target_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn review_count_today(&self) -> Result<i64> {
        let start_of_day = Local::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM reviews WHERE reviewed_at >= ?",
        )
        .bind(start_of_day)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }

    pub async fn pending_review_count(&self) -> Result<i64> {
        let now = chrono::Local::now().naive_local();
        let row: (i64,) = sqlx::query_as(
            "SELECT (SELECT COUNT(*) FROM words WHERE next_review IS NOT NULL AND next_review <= ?) + (SELECT COUNT(*) FROM phrases WHERE next_review IS NOT NULL AND next_review <= ?)",
        )
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }
}
