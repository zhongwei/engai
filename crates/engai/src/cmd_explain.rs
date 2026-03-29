use anyhow::Result;

use engai_core::ai::AiClient;
use engai_core::config::Config;
use engai_core::db::Db;
use engai_core::markdown::{MarkdownPhrase, MarkdownWord};
use engai_core::prompt::PromptEngine;

#[derive(clap::Subcommand)]
pub enum ExplainTarget {
    Word { word: String },
    Phrase { phrase: String },
}

pub async fn run(target: ExplainTarget) -> Result<()> {
    let config = Config::load_global()?;
    let db = Db::new(&config.db_path()).await?;
    let ai = AiClient::from_config(&config)?;
    let prompts_dir = Config::config_dir().join("prompts");
    let engine = PromptEngine::new(prompts_dir);

    match target {
        ExplainTarget::Word { word } => {
            let w = db.get_word(&word).await?.ok_or_else(|| {
                anyhow::anyhow!("Word '{}' not found. Add it first with `engai add word {}`", word, word)
            })?;
            let explanation = ai.explain_word(&word, &engine).await?;
            db.update_word(w.id, None, None, Some(&explanation), None, None, None, None)
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
            let p = db.get_phrase(&phrase).await?.ok_or_else(|| {
                anyhow::anyhow!("Phrase '{}' not found. Add it first with `engai add phrase {}`", phrase, phrase)
            })?;
            let explanation = ai.explain_phrase(&phrase, &engine).await?;
            db.update_phrase(p.id, None, Some(&explanation), None, None, None, None)
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
