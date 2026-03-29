use anyhow::Result;

use engai_core::config::Config;
use engai_core::db::Db;
use engai_core::markdown::{MarkdownPhrase, MarkdownWord};

pub async fn run(word: Option<String>, phrase: Option<String>, all: bool) -> Result<()> {
    let config = Config::load_global()?;
    let db = Db::new(&config.db_path()).await?;
    let docs_path = config.docs_path();

    if all {
        let words = db.list_words(None, None, 1000, 0).await?;
        let phrases = db.list_phrases(None, None, 1000, 0).await?;

        let vocab_dir = docs_path.join("01_vocab");
        for w in &words {
            let md = MarkdownWord {
                word: w.word.clone(),
                phonetic: w.phonetic.clone(),
                familiarity: w.familiarity,
                interval: w.interval,
                next_review: w.next_review,
                meaning: w.meaning.clone(),
                examples: Vec::new(),
                synonyms: Vec::new(),
                ai_explanation: None,
                my_notes: Vec::new(),
                reviews: Vec::new(),
            };
            let path = vocab_dir.join(format!("{}.md", w.word));
            md.save_to_file(&path)?;
        }

        let phrases_dir = docs_path.join("02_phrases");
        for p in &phrases {
            let md = MarkdownPhrase {
                phrase: p.phrase.clone(),
                familiarity: p.familiarity,
                interval: p.interval,
                next_review: p.next_review,
                meaning: p.meaning.clone(),
                examples: Vec::new(),
                ai_explanation: None,
                my_notes: Vec::new(),
                reviews: Vec::new(),
            };
            let path = phrases_dir.join(format!("{}.md", p.phrase));
            md.save_to_file(&path)?;
        }

        println!(
            "Exported {} words and {} phrases",
            words.len(),
            phrases.len()
        );
        return Ok(());
    }

    if let Some(word) = word {
        let w = db.get_word(&word).await?.ok_or_else(|| {
            anyhow::anyhow!("Word '{}' not found", word)
        })?;
        let md = MarkdownWord {
            word: w.word.clone(),
            phonetic: w.phonetic.clone(),
            familiarity: w.familiarity,
            interval: w.interval,
            next_review: w.next_review,
            meaning: w.meaning.clone(),
            examples: Vec::new(),
            synonyms: Vec::new(),
            ai_explanation: None,
            my_notes: Vec::new(),
            reviews: Vec::new(),
        };
        let path = docs_path.join("01_vocab").join(format!("{}.md", w.word));
        md.save_to_file(&path)?;
        println!("Exported word: {}", w.word);
    }

    if let Some(phrase) = phrase {
        let p = db.get_phrase(&phrase).await?.ok_or_else(|| {
            anyhow::anyhow!("Phrase '{}' not found", phrase)
        })?;
        let md = MarkdownPhrase {
            phrase: p.phrase.clone(),
            familiarity: p.familiarity,
            interval: p.interval,
            next_review: p.next_review,
            meaning: p.meaning.clone(),
            examples: Vec::new(),
            ai_explanation: None,
            my_notes: Vec::new(),
            reviews: Vec::new(),
        };
        let path = docs_path.join("02_phrases").join(format!("{}.md", p.phrase));
        md.save_to_file(&path)?;
        println!("Exported phrase: {}", p.phrase);
    }

    Ok(())
}
