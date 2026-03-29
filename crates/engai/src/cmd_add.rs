use anyhow::Result;

use engai_core::config::Config;
use engai_core::db::Db;
use engai_core::markdown::{MarkdownPhrase, MarkdownWord};

#[derive(clap::Subcommand)]
pub enum AddTarget {
    Word { word: String },
    Phrase { phrase: String },
}

pub async fn run(target: AddTarget) -> Result<()> {
    let config = Config::load_global()?;
    let db = Db::new(&config.db_path()).await?;

    match target {
        AddTarget::Word { word } => {
            let w = db.get_word(&word).await?;
            if w.is_some() {
                println!("Word '{}' already exists", word);
                return Ok(());
            }
            db.add_word(&word, None, None).await?;
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
            let p = db.get_phrase(&phrase).await?;
            if p.is_some() {
                println!("Phrase '{}' already exists", phrase);
                return Ok(());
            }
            db.add_phrase(&phrase, None).await?;
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
