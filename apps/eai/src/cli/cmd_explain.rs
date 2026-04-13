use std::sync::Arc;

use anyhow::Result;

use crate::config::Config;
use crate::db::Db;
use esync::{MarkdownPhrase, MarkdownWord};

use crate::state::AppState;

#[derive(clap::Subcommand)]
pub enum ExplainTarget {
    Word { word: String },
    Phrase { phrase: String },
}

pub async fn run(target: ExplainTarget) -> Result<()> {
    let config = Config::load_global()?;
    let db = Db::new(&config.db_path()).await?;
    let state = AppState::new(Arc::new(db), config.clone());

    match target {
        ExplainTarget::Word { word } => {
            let w = state.word_service.get_word(&word).await.map_err(|_e| {
                anyhow::anyhow!("Word '{}' not found. Add it first with `engai add word {}`", word, word)
            })?;
            let explanation = state.word_service.explain_word(&word).await?;
            state.word_service.update_word(w.id, None, None, Some(&explanation), None, None, None, None)
                .await?;

            let md_path = config.docs_path().join("01_vocab").join(format!("{}.md", word));
            if md_path.exists() {
                let mut md = MarkdownWord::parse_file(&md_path)?;
                md.ai_explanation = Some(explanation.clone());
                md.save_to_file(&md_path)?;
            } else {
                let md = MarkdownWord {
                    word: word.clone(),
                    phonetic: w.phonetic,
                    familiarity: w.familiarity,
                    interval: w.interval,
                    next_review: w.next_review,
                    meaning: Some(explanation.clone()),
                    examples: Vec::new(),
                    synonyms: Vec::new(),
                    ai_explanation: Some(explanation.clone()),
                    my_notes: Vec::new(),
                    reviews: Vec::new(),
                };
                md.save_to_file(&md_path)?;
            }
            println!("AI explanation for '{}' saved", word);
        }
        ExplainTarget::Phrase { phrase } => {
            let p = state.phrase_service.find_phrase(&phrase).await?.ok_or_else(|| {
                anyhow::anyhow!("Phrase '{}' not found. Add it first with `engai add phrase {}`", phrase, phrase)
            })?;
            let explanation = state.phrase_service.explain_phrase(&phrase).await?;
            state.phrase_service.update_phrase(p.id, None, Some(&explanation), None, None, None, None)
                .await?;

            let md_path = config
                .docs_path()
                .join("02_phrases")
                .join(format!("{}.md", phrase));
            if md_path.exists() {
                let mut md = MarkdownPhrase::parse_file(&md_path)?;
                md.ai_explanation = Some(explanation.clone());
                md.save_to_file(&md_path)?;
            } else {
                let md = MarkdownPhrase {
                    phrase: phrase.clone(),
                    familiarity: p.familiarity,
                    interval: p.interval,
                    next_review: p.next_review,
                    meaning: Some(explanation.clone()),
                    examples: Vec::new(),
                    ai_explanation: Some(explanation.clone()),
                    my_notes: Vec::new(),
                    reviews: Vec::new(),
                };
                md.save_to_file(&md_path)?;
            }
            println!("AI explanation for '{}' saved", phrase);
        }
    }

    Ok(())
}
