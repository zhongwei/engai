use anyhow::{Context, Result};
use chrono::{Local, NaiveDateTime};
use sqlx::SqlitePool;

use crate::models::Phrase;

#[derive(Clone)]
pub struct PhraseRepository {
    pool: SqlitePool,
}

impl PhraseRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn add_phrase(&self, phrase: &str, meaning: Option<&str>) -> Result<Phrase> {
        let row = sqlx::query_as::<_, Phrase>(
            "INSERT INTO phrases (phrase, meaning) VALUES (?, ?) RETURNING *",
        )
        .bind(phrase)
        .bind(meaning)
        .fetch_one(&self.pool)
        .await
        .context("Failed to add phrase")?;
        Ok(row)
    }

    pub async fn get_phrase(&self, phrase: &str) -> Result<Option<Phrase>> {
        let row = sqlx::query_as::<_, Phrase>("SELECT * FROM phrases WHERE phrase = ?")
            .bind(phrase)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    pub async fn get_phrase_by_id(&self, id: i64) -> Result<Option<Phrase>> {
        let row = sqlx::query_as::<_, Phrase>("SELECT * FROM phrases WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    pub async fn list_phrases(
        &self,
        search: Option<&str>,
        familiarity_gte: Option<i32>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Phrase>> {
        let mut query = String::from("SELECT * FROM phrases WHERE 1=1");
        if search.is_some() {
            query.push_str(" AND phrase LIKE ?");
        }
        if familiarity_gte.is_some() {
            query.push_str(" AND familiarity >= ?");
        }
        query.push_str(" ORDER BY updated_at DESC LIMIT ? OFFSET ?");

        let mut q = sqlx::query_as::<_, Phrase>(&query);
        if let Some(s) = search {
            q = q.bind(format!("%{}%", s));
        }
        if let Some(f) = familiarity_gte {
            q = q.bind(f);
        }
        q = q.bind(limit).bind(offset);
        let result = q.fetch_all(&self.pool).await?;
        Ok(result)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_phrase(
        &self,
        id: i64,
        phrase: Option<&str>,
        meaning: Option<&str>,
        familiarity: Option<i32>,
        next_review: Option<NaiveDateTime>,
        interval: Option<i32>,
        ease_factor: Option<f64>,
    ) -> Result<Option<Phrase>> {
        let current = self.get_phrase_by_id(id).await?;
        let current = match current {
            Some(p) => p,
            None => return Ok(None),
        };
        let phrase = phrase.unwrap_or(&current.phrase);
        let meaning = meaning.or(current.meaning.as_deref());
        let familiarity = familiarity.unwrap_or(current.familiarity);
        let next_review = next_review.or(current.next_review);
        let interval = interval.unwrap_or(current.interval);
        let ease_factor = ease_factor.unwrap_or(current.ease_factor);

        let row = sqlx::query_as::<_, Phrase>(
            "UPDATE phrases SET phrase = ?, meaning = ?, familiarity = ?, next_review = ?, interval = ?, ease_factor = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? RETURNING *",
        )
        .bind(phrase)
        .bind(meaning)
        .bind(familiarity)
        .bind(next_review)
        .bind(interval)
        .bind(ease_factor)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to update phrase")?;
        Ok(row)
    }

    pub async fn delete_phrase(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM phrases WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_today_review_phrases(&self) -> Result<Vec<Phrase>> {
        let now = Local::now().naive_local();
        let rows = sqlx::query_as::<_, Phrase>(
            "SELECT * FROM phrases WHERE next_review IS NOT NULL AND next_review <= ? ORDER BY next_review",
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn phrase_count(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM phrases")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }
}
