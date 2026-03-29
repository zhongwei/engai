use anyhow::{Context, Result};
use chrono::{Local, NaiveDateTime};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;

use crate::models::{Example, Note, Phrase, Reading, Review, Word};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn new(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let options = SqliteConnectOptions::from_str(&db_url)?
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .context("Failed to connect to SQLite")?;
        MIGRATOR
            .run(&pool)
            .await
            .context("Failed to run migrations")?;
        Ok(Self { pool })
    }

    pub async fn new_in_memory() -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        MIGRATOR.run(&pool).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
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

    pub async fn word_count(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM words")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn phrase_count(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM phrases")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
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
        let now = Local::now().naive_local();
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
