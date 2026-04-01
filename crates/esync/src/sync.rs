use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use crate::markdown::{MarkdownPhrase, MarkdownWord};
use crate::models::{PhraseSummary, ReviewInfo, WordSummary};

#[async_trait]
pub trait SyncDb: Send + Sync {
    async fn list_words(&self) -> Result<Vec<WordSummary>>;
    async fn get_word(&self, word: &str) -> Result<Option<WordSummary>>;
    async fn add_word(&self, word: &str, phonetic: Option<&str>, meaning: Option<&str>) -> Result<WordSummary>;
    async fn update_word(
        &self,
        id: i64,
        phonetic: Option<&str>,
        meaning: Option<&str>,
        familiarity: Option<i32>,
        next_review: Option<chrono::NaiveDateTime>,
        interval: Option<i32>,
    ) -> Result<()>;

    async fn list_phrases(&self) -> Result<Vec<PhraseSummary>>;
    async fn get_phrase(&self, phrase: &str) -> Result<Option<PhraseSummary>>;
    async fn add_phrase(&self, phrase: &str, meaning: Option<&str>) -> Result<PhraseSummary>;
    async fn update_phrase(
        &self,
        id: i64,
        meaning: Option<&str>,
        familiarity: Option<i32>,
        next_review: Option<chrono::NaiveDateTime>,
        interval: Option<i32>,
    ) -> Result<()>;

    async fn get_examples(&self, target_type: &str, target_id: i64) -> Result<Vec<String>>;
    async fn add_example(&self, target_type: &str, target_id: i64, sentence: &str) -> Result<()>;

    async fn get_notes(&self, target_type: &str, target_id: i64) -> Result<Vec<String>>;

    async fn get_reviews(&self, target_type: &str, target_id: i64) -> Result<Vec<ReviewInfo>>;
}

pub struct SyncEngine<T: SyncDb> {
    db: T,
    docs_path: PathBuf,
}

impl<T: SyncDb> SyncEngine<T> {
    pub fn new(db: T, docs_path: PathBuf) -> Self {
        Self { db, docs_path }
    }

    pub async fn sync_all(&self) -> Result<()> {
        self.sync_words().await?;
        self.sync_phrases().await?;
        Ok(())
    }

    pub async fn sync_words(&self) -> Result<()> {
        let vocab_dir = self.docs_path.join("01_vocab");
        tokio::fs::create_dir_all(&vocab_dir).await?;

        let words = self.db.list_words().await?;
        for word in &words {
            let md_path = vocab_dir.join(format!("{}.md", word.word));
            let needs_write = match tokio::fs::metadata(&md_path).await {
                Ok(_) => {
                    let file_mtime = tokio::fs::metadata(&md_path).await?.modified()?;
                    let db_system_time = naive_to_system_time(word.updated_at);
                    db_system_time > file_mtime
                }
                Err(_) => true,
            };

            if needs_write {
                let examples = self.db.get_examples("word", word.id).await?;
                let notes = self.db.get_notes("word", word.id).await?;
                let reviews = self.db.get_reviews("word", word.id).await?;

                let md_word = MarkdownWord {
                    word: word.word.clone(),
                    phonetic: word.phonetic.clone(),
                    familiarity: word.familiarity,
                    interval: word.interval,
                    next_review: word.next_review,
                    meaning: word.meaning.clone(),
                    examples,
                    synonyms: Vec::new(),
                    ai_explanation: None,
                    my_notes: notes,
                    reviews: reviews
                        .iter()
                        .map(|r| {
                            format!(
                                "{}: quality {}",
                                r.reviewed_at.format("%Y-%m-%d"),
                                r.quality
                            )
                        })
                        .collect(),
                };
                md_word.save_to_file(&md_path)?;
                tracing::info!("synced word '{}' to markdown", word.word);
            }
        }

        let mut entries = tokio::fs::read_dir(&vocab_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            match MarkdownWord::parse_file(&path) {
                Ok(md_word) => {
                    let file_mtime = tokio::fs::metadata(&path).await?.modified()?;
                    let is_new = self.db.get_word(&md_word.word).await?.is_none();

                    let should_import = if is_new {
                        true
                    } else {
                        let db_word = self.db.get_word(&md_word.word).await?.unwrap();
                        let db_system_time = naive_to_system_time(db_word.updated_at);
                        file_mtime > db_system_time
                    };

                    if should_import {
                        if is_new {
                            let inserted = self
                                .db
                                .add_word(
                                    &md_word.word,
                                    md_word.phonetic.as_deref(),
                                    md_word.meaning.as_deref(),
                                )
                                .await?;
                            for ex in &md_word.examples {
                                let _ = self.db.add_example("word", inserted.id, ex).await;
                            }
                            if md_word.familiarity != 0 || md_word.interval != 0 {
                                let _ = self
                                    .db
                                    .update_word(
                                        inserted.id,
                                        None,
                                        None,
                                        Some(md_word.familiarity),
                                        md_word.next_review,
                                        Some(md_word.interval),
                                    )
                                    .await;
                            }
                            tracing::info!("imported word '{}' from markdown", md_word.word);
                        } else {
                            let db_word = self.db.get_word(&md_word.word).await?.unwrap();
                            let _ = self
                                .db
                                .update_word(
                                    db_word.id,
                                    md_word.phonetic.as_deref(),
                                    md_word.meaning.as_deref(),
                                    Some(md_word.familiarity),
                                    md_word.next_review,
                                    Some(md_word.interval),
                                )
                                .await;
                            tracing::info!("updated word '{}' from markdown", md_word.word);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("failed to parse {}: {e}", path.display());
                }
            }
        }

        Ok(())
    }

    pub async fn sync_phrases(&self) -> Result<()> {
        let phrases_dir = self.docs_path.join("02_phrases");
        tokio::fs::create_dir_all(&phrases_dir).await?;

        let phrases = self.db.list_phrases().await?;
        for phrase in &phrases {
            let filename = phrase.phrase.replace(' ', "_");
            let md_path = phrases_dir.join(format!("{}.md", filename));
            let needs_write = match tokio::fs::metadata(&md_path).await {
                Ok(_) => {
                    let file_mtime = tokio::fs::metadata(&md_path).await?.modified()?;
                    let db_system_time = naive_to_system_time(phrase.updated_at);
                    db_system_time > file_mtime
                }
                Err(_) => true,
            };

            if needs_write {
                let examples = self.db.get_examples("phrase", phrase.id).await?;
                let notes = self.db.get_notes("phrase", phrase.id).await?;
                let reviews = self.db.get_reviews("phrase", phrase.id).await?;

                let md_phrase = MarkdownPhrase {
                    phrase: phrase.phrase.clone(),
                    familiarity: phrase.familiarity,
                    interval: phrase.interval,
                    next_review: phrase.next_review,
                    meaning: phrase.meaning.clone(),
                    examples,
                    ai_explanation: None,
                    my_notes: notes,
                    reviews: reviews
                        .iter()
                        .map(|r| {
                            format!(
                                "{}: quality {}",
                                r.reviewed_at.format("%Y-%m-%d"),
                                r.quality
                            )
                        })
                        .collect(),
                };
                md_phrase.save_to_file(&md_path)?;
                tracing::info!("synced phrase '{}' to markdown", phrase.phrase);
            }
        }

        let mut entries = tokio::fs::read_dir(&phrases_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            match MarkdownPhrase::parse_file(&path) {
                Ok(md_phrase) => {
                    let file_mtime = tokio::fs::metadata(&path).await?.modified()?;
                    let is_new = self.db.get_phrase(&md_phrase.phrase).await?.is_none();

                    let should_import = if is_new {
                        true
                    } else {
                        let db_phrase = self.db.get_phrase(&md_phrase.phrase).await?.unwrap();
                        let db_system_time = naive_to_system_time(db_phrase.updated_at);
                        file_mtime > db_system_time
                    };

                    if should_import {
                        if is_new {
                            let inserted = self
                                .db
                                .add_phrase(&md_phrase.phrase, md_phrase.meaning.as_deref())
                                .await?;
                            if md_phrase.familiarity != 0 || md_phrase.interval != 0 {
                                let _ = self
                                    .db
                                    .update_phrase(
                                        inserted.id,
                                        None,
                                        Some(md_phrase.familiarity),
                                        md_phrase.next_review,
                                        Some(md_phrase.interval),
                                    )
                                    .await;
                            }
                            tracing::info!("imported phrase '{}' from markdown", md_phrase.phrase);
                        } else {
                            let db_phrase = self.db.get_phrase(&md_phrase.phrase).await?.unwrap();
                            let _ = self
                                .db
                                .update_phrase(
                                    db_phrase.id,
                                    md_phrase.meaning.as_deref(),
                                    Some(md_phrase.familiarity),
                                    md_phrase.next_review,
                                    Some(md_phrase.interval),
                                )
                                .await;
                            tracing::info!("updated phrase '{}' from markdown", md_phrase.phrase);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("failed to parse {}: {e}", path.display());
                }
            }
        }

        Ok(())
    }
}

fn naive_to_system_time(dt: chrono::NaiveDateTime) -> std::time::SystemTime {
    use chrono::{DateTime, Local};
    let local: DateTime<Local> = DateTime::from_naive_utc_and_offset(dt, *Local::now().offset());
    local.into()
}
