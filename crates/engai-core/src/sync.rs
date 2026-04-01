use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, warn};

use crate::db::{Db, ExampleRepository, NoteRepository, PhraseRepository, ReviewRepository, WordRepository};
use crate::markdown::{MarkdownPhrase, MarkdownWord};

pub struct SyncEngine {
    #[allow(dead_code)]
    db: Arc<Db>,
    word_repo: WordRepository,
    phrase_repo: PhraseRepository,
    example_repo: ExampleRepository,
    note_repo: NoteRepository,
    review_repo: ReviewRepository,
    docs_path: PathBuf,
}

impl SyncEngine {
    pub fn new(db: Arc<Db>, docs_path: &Path, _prompts_path: &Path) -> Self {
        let pool = db.pool().clone();
        Self {
            db,
            word_repo: WordRepository::new(pool.clone()),
            phrase_repo: PhraseRepository::new(pool.clone()),
            example_repo: ExampleRepository::new(pool.clone()),
            note_repo: NoteRepository::new(pool.clone()),
            review_repo: ReviewRepository::new(pool),
            docs_path: docs_path.to_path_buf(),
        }
    }

    pub async fn sync_all(&self) -> Result<()> {
        self.sync_words().await?;
        self.sync_phrases().await?;
        Ok(())
    }

    pub async fn sync_words(&self) -> Result<()> {
        let vocab_dir = self.docs_path.join("01_vocab");
        tokio::fs::create_dir_all(&vocab_dir).await?;

        let words = self.word_repo.list_words(None, None, 10000, 0).await?;
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
                let examples = self.example_repo.get_examples("word", word.id).await?;
                let notes = self.note_repo.get_notes("word", word.id).await?;
                let reviews = self.review_repo.get_reviews("word", word.id).await?;

                let md_word = MarkdownWord {
                    word: word.word.clone(),
                    phonetic: word.phonetic.clone(),
                    familiarity: word.familiarity,
                    interval: word.interval,
                    next_review: word.next_review,
                    meaning: word.meaning.clone(),
                    examples: examples.iter().map(|e| e.sentence.clone()).collect(),
                    synonyms: Vec::new(),
                    ai_explanation: None,
                    my_notes: notes.iter().map(|n| n.content.clone()).collect(),
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
                info!("synced word '{}' to markdown", word.word);
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
                    let is_new = self.word_repo.get_word(&md_word.word).await?.is_none();

                    let should_import = if is_new {
                        true
                    } else {
                        let db_word = self.word_repo.get_word(&md_word.word).await?.unwrap();
                        let db_system_time = naive_to_system_time(db_word.updated_at);
                        file_mtime > db_system_time
                    };

                    if should_import {
                        if is_new {
                            let inserted = self
                                .word_repo
                                .add_word(
                                    &md_word.word,
                                    md_word.phonetic.as_deref(),
                                    md_word.meaning.as_deref(),
                                )
                                .await?;
                            for ex in &md_word.examples {
                                let _ = self
                                    .example_repo
                                    .add_example("word", inserted.id, ex, None)
                                    .await;
                            }
                            if md_word.familiarity != 0 || md_word.interval != 0 {
                                let _ = self
                                    .word_repo
                                    .update_word(
                                        inserted.id,
                                        None,
                                        None,
                                        None,
                                        Some(md_word.familiarity),
                                        md_word.next_review,
                                        Some(md_word.interval),
                                        None,
                                    )
                                    .await;
                            }
                            info!("imported word '{}' from markdown", md_word.word);
                        } else {
                            let db_word = self.word_repo.get_word(&md_word.word).await?.unwrap();
                            let _ = self
                                .word_repo
                                .update_word(
                                    db_word.id,
                                    None,
                                    md_word.phonetic.as_deref(),
                                    md_word.meaning.as_deref(),
                                    Some(md_word.familiarity),
                                    md_word.next_review,
                                    Some(md_word.interval),
                                    None,
                                )
                                .await;
                            info!("updated word '{}' from markdown", md_word.word);
                        }
                    }
                }
                Err(e) => {
                    warn!("failed to parse {}: {e}", path.display());
                }
            }
        }

        Ok(())
    }

    pub async fn sync_phrases(&self) -> Result<()> {
        let phrases_dir = self.docs_path.join("02_phrases");
        tokio::fs::create_dir_all(&phrases_dir).await?;

        let phrases = self.phrase_repo.list_phrases(None, None, 10000, 0).await?;
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
                let examples = self.example_repo.get_examples("phrase", phrase.id).await?;
                let notes = self.note_repo.get_notes("phrase", phrase.id).await?;
                let reviews = self.review_repo.get_reviews("phrase", phrase.id).await?;

                let md_phrase = MarkdownPhrase {
                    phrase: phrase.phrase.clone(),
                    familiarity: phrase.familiarity,
                    interval: phrase.interval,
                    next_review: phrase.next_review,
                    meaning: phrase.meaning.clone(),
                    examples: examples.iter().map(|e| e.sentence.clone()).collect(),
                    ai_explanation: None,
                    my_notes: notes.iter().map(|n| n.content.clone()).collect(),
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
                info!("synced phrase '{}' to markdown", phrase.phrase);
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
                    let is_new = self.phrase_repo.get_phrase(&md_phrase.phrase).await?.is_none();

                    let should_import = if is_new {
                        true
                    } else {
                        let db_phrase = self.phrase_repo.get_phrase(&md_phrase.phrase).await?.unwrap();
                        let db_system_time = naive_to_system_time(db_phrase.updated_at);
                        file_mtime > db_system_time
                    };

                    if should_import {
                        if is_new {
                            let inserted = self
                                .phrase_repo
                                .add_phrase(&md_phrase.phrase, md_phrase.meaning.as_deref())
                                .await?;
                            if md_phrase.familiarity != 0 || md_phrase.interval != 0 {
                                let _ = self
                                    .phrase_repo
                                    .update_phrase(
                                        inserted.id,
                                        None,
                                        None,
                                        Some(md_phrase.familiarity),
                                        md_phrase.next_review,
                                        Some(md_phrase.interval),
                                        None,
                                    )
                                    .await;
                            }
                            info!("imported phrase '{}' from markdown", md_phrase.phrase);
                        } else {
                            let db_phrase = self.phrase_repo.get_phrase(&md_phrase.phrase).await?.unwrap();
                            let _ = self
                                .phrase_repo
                                .update_phrase(
                                    db_phrase.id,
                                    None,
                                    md_phrase.meaning.as_deref(),
                                    Some(md_phrase.familiarity),
                                    md_phrase.next_review,
                                    Some(md_phrase.interval),
                                    None,
                                )
                                .await;
                            info!("updated phrase '{}' from markdown", md_phrase.phrase);
                        }
                    }
                }
                Err(e) => {
                    warn!("failed to parse {}: {e}", path.display());
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
