use anyhow::Result;

use crate::config::Config;
use crate::db::{Db, PhraseRepository, ReviewRepository, WordRepository};

pub async fn run() -> Result<()> {
    let config = Config::load_global()?;
    let db = Db::new(&config.db_path()).await?;
    let pool = db.pool().clone();
    let word_repo = WordRepository::new(pool.clone());
    let phrase_repo = PhraseRepository::new(pool.clone());
    let review_repo = ReviewRepository::new(pool);

    let word_count = word_repo.word_count().await?;
    let phrase_count = phrase_repo.phrase_count().await?;
    let reviewed_today = review_repo.review_count_today().await?;
    let pending = review_repo.pending_review_count().await?;

    println!("engai stats");
    println!("  Words:        {}", word_count);
    println!("  Phrases:      {}", phrase_count);
    println!("  Reviewed today: {}", reviewed_today);
    println!("  Pending review: {}", pending);
    println!("  DB path:      {}", config.db_path().display());
    println!("  Docs path:    {}", config.docs_path().display());

    Ok(())
}
