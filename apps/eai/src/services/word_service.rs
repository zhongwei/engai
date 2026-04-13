use chrono::NaiveDateTime;

use crate::db::{ExampleRepository, WordRepository};
use crate::error::{AppError, Result};
use crate::models::{Example, Word};
use crate::services::AiService;

#[derive(Clone)]
pub struct WordService {
    word_repo: WordRepository,
    example_repo: ExampleRepository,
    ai: AiService,
}

impl WordService {
    pub fn new(
        word_repo: WordRepository,
        example_repo: ExampleRepository,
        ai: AiService,
    ) -> Self {
        Self {
            word_repo,
            example_repo,
            ai,
        }
    }

    pub async fn list_words(
        &self,
        search: Option<&str>,
        familiarity_gte: Option<i32>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Word>> {
        Ok(self
            .word_repo
            .list_words(search, familiarity_gte, limit, offset)
            .await?)
    }

    pub async fn add_word(
        &self,
        word: &str,
        phonetic: Option<&str>,
        meaning: Option<&str>,
    ) -> Result<Word> {
        if word.trim().is_empty() {
            return Err(AppError::ValidationError("word cannot be empty".into()));
        }
        Ok(self.word_repo.add_word(word, phonetic, meaning).await?)
    }

    pub async fn get_word(&self, word: &str) -> Result<Word> {
        self.word_repo
            .get_word(word)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("word '{}' not found", word)))
    }

    #[allow(dead_code)]
    pub async fn get_word_by_id(&self, id: i64) -> Result<Word> {
        self.word_repo
            .get_word_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("word {} not found", id)))
    }

    pub async fn find_word(&self, word: &str) -> Result<Option<Word>> {
        Ok(self.word_repo.get_word(word).await?)
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
    ) -> Result<Word> {
        self.word_repo
            .update_word(id, word, phonetic, meaning, familiarity, next_review, interval, ease_factor)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("word {} not found", id)))
    }

    pub async fn delete_word(&self, word: &str) -> Result<()> {
        let w = self.get_word(word).await?;
        self.word_repo.delete_word(w.id).await?;
        Ok(())
    }

    pub async fn get_examples(&self, word: &str) -> Result<Vec<Example>> {
        let w = self.get_word(word).await?;
        Ok(self.example_repo.get_examples("word", w.id).await?)
    }

    #[allow(dead_code)]
    pub async fn get_today_review_words(&self) -> Result<Vec<Word>> {
        Ok(self.word_repo.get_today_review_words().await?)
    }

    #[allow(dead_code)]
    pub async fn word_count(&self) -> Result<i64> {
        Ok(self.word_repo.word_count().await?)
    }

    pub async fn explain_word(&self, word: &str) -> Result<String> {
        self.ai
            .explain_word(word)
            .await
            .map_err(|e| AppError::AiError(e.to_string()))
    }
}
