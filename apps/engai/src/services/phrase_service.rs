use chrono::NaiveDateTime;

use crate::db::{ExampleRepository, PhraseRepository};
use crate::error::{AppError, Result};
use crate::models::{Example, Phrase};
use crate::services::AiService;

#[derive(Clone)]
pub struct PhraseService {
    phrase_repo: PhraseRepository,
    example_repo: ExampleRepository,
    ai: AiService,
}

impl PhraseService {
    pub fn new(
        phrase_repo: PhraseRepository,
        example_repo: ExampleRepository,
        ai: AiService,
    ) -> Self {
        Self {
            phrase_repo,
            example_repo,
            ai,
        }
    }

    pub async fn list_phrases(
        &self,
        search: Option<&str>,
        familiarity_gte: Option<i32>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Phrase>> {
        Ok(self
            .phrase_repo
            .list_phrases(search, familiarity_gte, limit, offset)
            .await?)
    }

    pub async fn add_phrase(&self, phrase: &str, meaning: Option<&str>) -> Result<Phrase> {
        if phrase.trim().is_empty() {
            return Err(AppError::ValidationError("phrase cannot be empty".into()));
        }
        Ok(self.phrase_repo.add_phrase(phrase, meaning).await?)
    }

    pub async fn get_phrase_by_id(&self, id: i64) -> Result<Phrase> {
        self.phrase_repo
            .get_phrase_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("phrase {} not found", id)))
    }

    pub async fn find_phrase(&self, phrase: &str) -> Result<Option<Phrase>> {
        Ok(self.phrase_repo.get_phrase(phrase).await?)
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
    ) -> Result<Phrase> {
        self.phrase_repo
            .update_phrase(id, phrase, meaning, familiarity, next_review, interval, ease_factor)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("phrase {} not found", id)))
    }

    pub async fn delete_phrase(&self, id: i64) -> Result<()> {
        self.phrase_repo.delete_phrase(id).await?;
        Ok(())
    }

    pub async fn get_examples(&self, id: i64) -> Result<Vec<Example>> {
        Ok(self.example_repo.get_examples("phrase", id).await?)
    }

    #[allow(dead_code)]
    pub async fn get_today_review_phrases(&self) -> Result<Vec<Phrase>> {
        Ok(self.phrase_repo.get_today_review_phrases().await?)
    }

    #[allow(dead_code)]
    pub async fn phrase_count(&self) -> Result<i64> {
        Ok(self.phrase_repo.phrase_count().await?)
    }

    pub async fn explain_phrase(&self, phrase: &str) -> Result<String> {
        self.ai
            .explain_phrase(phrase)
            .await
            .map_err(|e| AppError::AiError(e.to_string()))
    }
}
