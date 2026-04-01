use std::sync::Arc;

use anyhow::Result;

use engai_core::config::Config;
use engai_core::db::Db;
use esync::{MarkdownPhrase, MarkdownWord};

use crate::state::AppState;

#[derive(clap::Subcommand)]
pub enum AddTarget {
    Word { word: String },
    Phrase { phrase: String },
}

pub async fn run(target: AddTarget) -> Result<()> {
    let config = Config::load_global()?;
    let db = Db::new(&config.db_path()).await?;
    let state = AppState::new(Arc::new(db), config.clone());

    match target {
        AddTarget::Word { word } => {
            if state.word_service.find_word(&word).await?.is_some() {
                println!("Word '{}' already exists", word);
                return Ok(());
            }
            state.word_service.add_word(&word, None, None).await?;
            let md = MarkdownWord {
                word: word.clone(),
                phonetic: None,
                familiarity: 0,
                interval: 0,
                next_review: None,
                meaning: None,
                examples: Vec::new(),
                synonyms: Vec::new(),
                ai_explanation: None,
                my_notes: Vec::new(),
                reviews: Vec::new(),
            };
            let path = config.docs_path().join("01_vocab").join(format!("{}.md", word));
            md.save_to_file(&path)?;
            println!("Added word: {}", word);
        }
        AddTarget::Phrase { phrase } => {
            if state.phrase_service.find_phrase(&phrase).await?.is_some() {
                println!("Phrase '{}' already exists", phrase);
                return Ok(());
            }
            state.phrase_service.add_phrase(&phrase, None).await?;
            let md = MarkdownPhrase {
                phrase: phrase.clone(),
                familiarity: 0,
                interval: 0,
                next_review: None,
                meaning: None,
                examples: Vec::new(),
                ai_explanation: None,
                my_notes: Vec::new(),
                reviews: Vec::new(),
            };
            let path = config
                .docs_path()
                .join("02_phrases")
                .join(format!("{}.md", phrase));
            md.save_to_file(&path)?;
            println!("Added phrase: {}", phrase);
        }
    }

    Ok(())
}
