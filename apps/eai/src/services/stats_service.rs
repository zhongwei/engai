use crate::db::{PhraseRepository, ReviewRepository, WordRepository};
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsData {
    pub word_count: i64,
    pub phrase_count: i64,
    pub pending_reviews: i64,
    pub reviewed_today: i64,
}

#[derive(Clone)]
pub struct StatsService {
    word_repo: WordRepository,
    phrase_repo: PhraseRepository,
    review_repo: ReviewRepository,
}

impl StatsService {
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

    pub async fn get_stats(&self) -> Result<StatsData> {
        let word_count = self.word_repo.word_count().await?;
        let phrase_count = self.phrase_repo.phrase_count().await?;
        let pending_reviews = self.review_repo.pending_review_count().await?;
        let reviewed_today = self.review_repo.review_count_today().await?;
        Ok(StatsData {
            word_count,
            phrase_count,
            pending_reviews,
            reviewed_today,
        })
    }
}
