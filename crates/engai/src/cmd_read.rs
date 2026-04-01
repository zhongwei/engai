use std::path::Path;

use anyhow::Result;

use engai_core::config::Config;
use engai_core::db::{Db, ReadingRepository};
use esync::MarkdownReading;

pub async fn run(file: &str) -> Result<()> {
    let config = Config::load_global()?;
    let db = Db::new(&config.db_path()).await?;
    let reading_repo = ReadingRepository::new(db.pool().clone());

    let path = Path::new(file);
    if !path.exists() {
        anyhow::bail!("File not found: {}", file);
    }

    let content = std::fs::read_to_string(path)?;
    let title = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Untitled");
    let source = path.to_str().map(|s| s.to_string());

    let reading = reading_repo.add_reading(Some(title), &content, source.as_deref()).await?;

    let api_key = config.resolve_api_key();
    let summary = if !api_key.is_empty() {
        let ai = match engai_core::ai::AiClient::from_config(&config) {
            Ok(client) => Some(client),
            Err(_) => {
                println!("Warning: Could not create AI client, skipping analysis");
                None
            }
        };

        match ai {
            Some(ai) => {
                let prompts_dir = Config::config_dir().join("prompts");
                let engine = engai_core::prompt::PromptEngine::new(prompts_dir);
                match ai.analyze_reading(&content, &engine).await {
                    Ok(result) => Some(result),
                    Err(e) => {
                        println!("Warning: AI analysis failed: {}", e);
                        None
                    }
                }
            }
            None => None,
        }
    } else {
        None
    };

    let md = MarkdownReading {
        title: title.to_string(),
        source,
        content: content.clone(),
        vocabulary: Vec::new(),
        summary,
        my_notes: Vec::new(),
    };

    let md_path = config
        .docs_path()
        .join("03_reading")
        .join(format!("{}.md", reading.id));
    md.save_to_file(&md_path)?;

    println!(
        "Added reading '{}' (id: {})",
        title,
        reading.id
    );
    Ok(())
}
