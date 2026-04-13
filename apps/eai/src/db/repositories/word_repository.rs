use anyhow::{Context, Result};
use chrono::{Local, NaiveDateTime};
use sqlx::SqlitePool;

use crate::models::Word;

#[derive(Clone)]
pub struct WordRepository {
    pool: SqlitePool,
}

impl WordRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn add_word(&self, word: &str, phonetic: Option<&str>, meaning: Option<&str>) -> Result<Word> {
        let row = sqlx::query_as::<_, Word>(
            "INSERT INTO words (word, phonetic, meaning) VALUES (?, ?, ?) RETURNING *",
        )
        .bind(word)
        .bind(phonetic)
        .bind(meaning)
        .fetch_one(&self.pool)
        .await
        .context("Failed to add word")?;
        Ok(row)
    }

    pub async fn get_word(&self, word: &str) -> Result<Option<Word>> {
        let row = sqlx::query_as::<_, Word>("SELECT * FROM words WHERE word = ?")
            .bind(word)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    pub async fn get_word_by_id(&self, id: i64) -> Result<Option<Word>> {
        let row = sqlx::query_as::<_, Word>("SELECT * FROM words WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    pub async fn list_words(
        &self,
        search: Option<&str>,
        familiarity_gte: Option<i32>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Word>> {
        let mut query = String::from("SELECT * FROM words WHERE 1=1");
        if search.is_some() {
            query.push_str(" AND word LIKE ?");
        }
        if familiarity_gte.is_some() {
            query.push_str(" AND familiarity >= ?");
        }
        query.push_str(" ORDER BY updated_at DESC LIMIT ? OFFSET ?");

        let mut q = sqlx::query_as::<_, Word>(&query);
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
    pub async fn update_word(
        &self,
        id: i64,
        word: Option<&str>,
        phonetic: Option<&str>,
        meaning: Option<&str>,
        familiarity: Option<i32>,
        next_review: Option<NaiveDateTime>,
        interval: Option<i32>,
        ease_factor: Option<f64>,
    ) -> Result<Option<Word>> {
        let current = self.get_word_by_id(id).await?;
        let current = match current {
            Some(w) => w,
            None => return Ok(None),
        };
        let word = word.unwrap_or(&current.word);
        let phonetic = phonetic.or(current.phonetic.as_deref());
        let meaning = meaning.or(current.meaning.as_deref());
        let familiarity = familiarity.unwrap_or(current.familiarity);
        let next_review = next_review.or(current.next_review);
        let interval = interval.unwrap_or(current.interval);
        let ease_factor = ease_factor.unwrap_or(current.ease_factor);

        let row = sqlx::query_as::<_, Word>(
            "UPDATE words SET word = ?, phonetic = ?, meaning = ?, familiarity = ?, next_review = ?, interval = ?, ease_factor = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? RETURNING *",
        )
        .bind(word)
        .bind(phonetic)
        .bind(meaning)
        .bind(familiarity)
        .bind(next_review)
        .bind(interval)
        .bind(ease_factor)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to update word")?;
        Ok(row)
    }

    pub async fn delete_word(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM words WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_today_review_words(&self) -> Result<Vec<Word>> {
        let now = Local::now().naive_local();
        let rows = sqlx::query_as::<_, Word>(
            "SELECT * FROM words WHERE next_review IS NOT NULL AND next_review <= ? ORDER BY next_review",
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn word_count(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM words")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }
}
