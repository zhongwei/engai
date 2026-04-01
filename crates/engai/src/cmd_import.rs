use std::path::Path;

use anyhow::Result;

use engai_core::config::Config;
use engai_core::db::{Db, PhraseRepository, WordRepository};
use engai_core::markdown::{MarkdownPhrase, MarkdownWord};

pub async fn run(path: &str) -> Result<()> {
    let config = Config::load_global()?;
    let db = Db::new(&config.db_path()).await?;
    let pool = db.pool().clone();
    let word_repo = WordRepository::new(pool.clone());
    let phrase_repo = PhraseRepository::new(pool);
    let path = Path::new(path);

    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }

    if path.is_dir() {
        import_dir(&word_repo, &phrase_repo, path).await?;
    } else {
        import_file(&word_repo, &phrase_repo, path).await?;
    }

    Ok(())
}

fn import_dir<'a>(
    word_repo: &'a WordRepository,
    phrase_repo: &'a PhraseRepository,
    dir: &'a Path,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        let mut count = 0;
        let entries: Vec<_> = std::fs::read_dir(dir)?.collect();
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                import_dir(word_repo, phrase_repo, &path).await?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                import_file(word_repo, phrase_repo, &path).await?;
                count += 1;
            }
        }
        if count > 0 {
            println!("Imported {} files from {}", count, dir.display());
        }
        Ok(())
    })
}

async fn import_file(word_repo: &WordRepository, phrase_repo: &PhraseRepository, path: &Path) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new(""));
    let parent_name = parent
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let content = std::fs::read_to_string(path)?;

    match parent_name {
        "01_vocab" => {
            let md = MarkdownWord::parse(&content)?;
            let existing = word_repo.get_word(&md.word).await?;
            if let Some(w) = existing {
                let meaning = md.meaning.as_deref().or(w.meaning.as_deref());
                word_repo.update_word(
                    w.id,
                    None,
                    md.phonetic.as_deref(),
                    meaning,
                    None,
                    md.next_review,
                    Some(md.interval),
                    None,
                )
                .await?;
            } else {
                word_repo.add_word(&md.word, md.phonetic.as_deref(), md.meaning.as_deref())
                    .await?;
            }
            println!("Imported word: {}", md.word);
        }
        "02_phrases" => {
            let md = MarkdownPhrase::parse(&content)?;
            let existing = phrase_repo.get_phrase(&md.phrase).await?;
            if let Some(p) = existing {
                let meaning = md.meaning.as_deref().or(p.meaning.as_deref());
                phrase_repo.update_phrase(
                    p.id,
                    None,
                    meaning,
                    None,
                    md.next_review,
                    Some(md.interval),
                    None,
                )
                .await?;
            } else {
                phrase_repo.add_phrase(&md.phrase, md.meaning.as_deref()).await?;
            }
            println!("Imported phrase: {}", md.phrase);
        }
        _ => {
            println!(
                "Skipped {} (unknown parent directory: {})",
                path.display(),
                parent_name
            );
        }
    }

    Ok(())
}
