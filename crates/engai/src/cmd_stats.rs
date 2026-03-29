use anyhow::Result;

use engai_core::config::Config;
use engai_core::db::Db;

pub async fn run() -> Result<()> {
    let config = Config::load_global()?;
    let db = Db::new(&config.db_path()).await?;

    let word_count = db.word_count().await?;
    let phrase_count = db.phrase_count().await?;
    let reviewed_today = db.review_count_today().await?;
    let pending = db.pending_review_count().await?;

    println!("engai stats");
    println!("  Words:        {}", word_count);
    println!("  Phrases:      {}", phrase_count);
    println!("  Reviewed today: {}", reviewed_today);
    println!("  Pending review: {}", pending);
    println!("  DB path:      {}", config.db_path().display());
    println!("  Docs path:    {}", config.docs_path().display());

    Ok(())
}
