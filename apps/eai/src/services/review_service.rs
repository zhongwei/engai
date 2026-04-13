use serde::Serialize;

use crate::db::{PhraseRepository, ReviewRepository, WordRepository};
use crate::error::{AppError, Result};
use crate::models::Word;
use crate::review::calculate_next_review;

#[derive(Debug, Clone, Serialize)]
pub struct ReviewEntry {
    pub target_type: String,
    pub id: i64,
    pub display: String,
    pub meaning: Option<String>,
    pub familiarity: i32,
    pub interval: i32,
    pub ease_factor: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewResult {
    pub next_review: chrono::NaiveDateTime,
    pub interval: i32,
    pub ease_factor: f64,
    pub familiarity: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewStats {
    pub pending_reviews: i64,
    pub reviewed_today: i64,
}

#[derive(Clone)]
pub struct ReviewService {
    word_repo: WordRepository,
    phrase_repo: PhraseRepository,
    review_repo: ReviewRepository,
}

impl ReviewService {
    pub fn new(
        word_repo: WordRepository,
        phrase_repo: PhraseRepository,
        review_repo: ReviewRepository,
    ) -> Self {
        Self {
            word_repo,
            phrase_repo,
            review_repo,
        }
    }

    pub async fn get_today_reviews(&self) -> Result<Vec<ReviewEntry>> {
        let words = self.word_repo.get_today_review_words().await?;
        let phrases = self.phrase_repo.get_today_review_phrases().await?;

        let mut items: Vec<ReviewEntry> = words
            .into_iter()
            .map(|w| ReviewEntry {
                target_type: "word".to_string(),
                id: w.id,
                display: w.word,
                meaning: w.meaning,
                familiarity: w.familiarity,
                interval: w.interval,
                ease_factor: w.ease_factor,
            })
            .collect();
        items.extend(phrases.into_iter().map(|p| ReviewEntry {
            target_type: "phrase".to_string(),
            id: p.id,
            display: p.phrase,
            meaning: p.meaning,
            familiarity: p.familiarity,
            interval: p.interval,
            ease_factor: p.ease_factor,
        }));
        Ok(items)
    }

    pub async fn submit_review(
        &self,
        target_type: &str,
        id: i64,
        quality: i32,
    ) -> Result<ReviewResult> {
        if !(0..=5).contains(&quality) {
            return Err(AppError::ValidationError(
                "quality must be between 0 and 5".into(),
            ));
        }

        let (interval, ease_factor) = match target_type {
            "word" => {
                let w = self
                    .word_repo
                    .get_word_by_id(id)
                    .await?
                    .ok_or_else(|| AppError::NotFound("word not found".into()))?;
                (w.interval, w.ease_factor)
            }
            "phrase" => {
                let p = self
                    .phrase_repo
                    .get_phrase_by_id(id)
                    .await?
                    .ok_or_else(|| AppError::NotFound("phrase not found".into()))?;
                (p.interval, p.ease_factor)
            }
            _ => {
                return Err(AppError::ValidationError(
                    "target_type must be 'word' or 'phrase'".into(),
                ))
            }
        };

        let result = calculate_next_review(quality, interval, ease_factor);

        match target_type {
            "word" => {
                self.word_repo
                    .update_word(
                        id,
                        None,
                        None,
                        None,
                        Some(result.familiarity),
                        Some(result.next_review),
                        Some(result.interval),
                        Some(result.ease_factor),
                    )
                    .await?;
            }
            "phrase" => {
                self.phrase_repo
                    .update_phrase(
                        id,
                        None,
                        None,
                        Some(result.familiarity),
                        Some(result.next_review),
                        Some(result.interval),
                        Some(result.ease_factor),
                    )
                    .await?;
            }
            _ => unreachable!(),
        }

        self.review_repo
            .add_review(target_type, id, quality)
            .await?;

        Ok(ReviewResult {
            next_review: result.next_review,
            interval: result.interval,
            ease_factor: result.ease_factor,
            familiarity: result.familiarity,
        })
    }

    pub async fn get_review_stats(&self) -> Result<ReviewStats> {
        let pending = self.review_repo.pending_review_count().await?;
        let reviewed_today = self.review_repo.review_count_today().await?;
        Ok(ReviewStats {
            pending_reviews: pending,
            reviewed_today,
        })
    }
}

impl From<Word> for ReviewEntry {
    fn from(w: Word) -> Self {
        ReviewEntry {
            target_type: "word".to_string(),
            id: w.id,
            display: w.word,
            meaning: w.meaning,
            familiarity: w.familiarity,
            interval: w.interval,
            ease_factor: w.ease_factor,
        }
    }
}
