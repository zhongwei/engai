use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;

use crate::db::{ExampleRepository, NoteRepository, PhraseRepository, ReviewRepository, WordRepository};
use esync::models::{PhraseSummary, ReviewInfo, WordSummary};
use esync::SyncDb;

pub struct SyncDbAdapter {
    word_repo: WordRepository,
    phrase_repo: PhraseRepository,
    example_repo: ExampleRepository,
    note_repo: NoteRepository,
    review_repo: ReviewRepository,
}

impl SyncDbAdapter {
    pub fn new(
        word_repo: WordRepository,
        phrase_repo: PhraseRepository,
        example_repo: ExampleRepository,
        note_repo: NoteRepository,
        review_repo: ReviewRepository,
    ) -> Self {
        Self {
            word_repo,
            phrase_repo,
            example_repo,
            note_repo,
            review_repo,
        }
    }
}

#[async_trait]
impl SyncDb for SyncDbAdapter {
    async fn list_words(&self) -> Result<Vec<WordSummary>> {
        let words = self.word_repo.list_words(None, None, 10000, 0).await?;
        Ok(words
            .into_iter()
            .map(|w| WordSummary {
                id: w.id,
                word: w.word,
                phonetic: w.phonetic,
                familiarity: w.familiarity,
                interval: w.interval,
                next_review: w.next_review,
                meaning: w.meaning,
                updated_at: w.updated_at,
            })
            .collect())
    }

    async fn get_word(&self, word: &str) -> Result<Option<WordSummary>> {
        let w = self.word_repo.get_word(word).await?;
        Ok(w.map(|w| WordSummary {
            id: w.id,
            word: w.word,
            phonetic: w.phonetic,
            familiarity: w.familiarity,
            interval: w.interval,
            next_review: w.next_review,
            meaning: w.meaning,
            updated_at: w.updated_at,
        }))
    }

    async fn add_word(&self, word: &str, phonetic: Option<&str>, meaning: Option<&str>) -> Result<WordSummary> {
        let w = self.word_repo.add_word(word, phonetic, meaning).await?;
        Ok(WordSummary {
            id: w.id,
            word: w.word,
            phonetic: w.phonetic,
            familiarity: w.familiarity,
            interval: w.interval,
            next_review: w.next_review,
            meaning: w.meaning,
            updated_at: w.updated_at,
        })
    }

    async fn update_word(
        &self,
        id: i64,
        phonetic: Option<&str>,
        meaning: Option<&str>,
        familiarity: Option<i32>,
        next_review: Option<NaiveDateTime>,
        interval: Option<i32>,
    ) -> Result<()> {
        self.word_repo
            .update_word(id, None, phonetic, meaning, familiarity, next_review, interval, None)
            .await?;
        Ok(())
    }

    async fn list_phrases(&self) -> Result<Vec<PhraseSummary>> {
        let phrases = self.phrase_repo.list_phrases(None, None, 10000, 0).await?;
        Ok(phrases
            .into_iter()
            .map(|p| PhraseSummary {
                id: p.id,
                phrase: p.phrase,
                familiarity: p.familiarity,
                interval: p.interval,
                next_review: p.next_review,
                meaning: p.meaning,
                updated_at: p.updated_at,
            })
            .collect())
    }

    async fn get_phrase(&self, phrase: &str) -> Result<Option<PhraseSummary>> {
        let p = self.phrase_repo.get_phrase(phrase).await?;
        Ok(p.map(|p| PhraseSummary {
            id: p.id,
            phrase: p.phrase,
            familiarity: p.familiarity,
            interval: p.interval,
            next_review: p.next_review,
            meaning: p.meaning,
            updated_at: p.updated_at,
        }))
    }

    async fn add_phrase(&self, phrase: &str, meaning: Option<&str>) -> Result<PhraseSummary> {
        let p = self.phrase_repo.add_phrase(phrase, meaning).await?;
        Ok(PhraseSummary {
            id: p.id,
            phrase: p.phrase,
            familiarity: p.familiarity,
            interval: p.interval,
            next_review: p.next_review,
            meaning: p.meaning,
            updated_at: p.updated_at,
        })
    }

    async fn update_phrase(
        &self,
        id: i64,
        meaning: Option<&str>,
        familiarity: Option<i32>,
        next_review: Option<NaiveDateTime>,
        interval: Option<i32>,
    ) -> Result<()> {
        self.phrase_repo
            .update_phrase(id, None, meaning, familiarity, next_review, interval, None)
            .await?;
        Ok(())
    }

    async fn get_examples(&self, target_type: &str, target_id: i64) -> Result<Vec<String>> {
        let examples = self.example_repo.get_examples(target_type, target_id).await?;
        Ok(examples.into_iter().map(|e| e.sentence).collect())
    }

    async fn add_example(&self, target_type: &str, target_id: i64, sentence: &str) -> Result<()> {
        self.example_repo.add_example(target_type, target_id, sentence, None).await?;
        Ok(())
    }

    async fn get_notes(&self, target_type: &str, target_id: i64) -> Result<Vec<String>> {
        let notes = self.note_repo.get_notes(target_type, target_id).await?;
        Ok(notes.into_iter().map(|n| n.content).collect())
    }

    async fn get_reviews(&self, target_type: &str, target_id: i64) -> Result<Vec<ReviewInfo>> {
        let reviews = self.review_repo.get_reviews(target_type, target_id).await?;
        Ok(reviews
            .into_iter()
            .map(|r| ReviewInfo {
                reviewed_at: r.reviewed_at,
                quality: r.quality,
            })
            .collect())
    }
}
