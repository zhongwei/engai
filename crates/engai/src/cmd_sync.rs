use std::sync::Arc;

use anyhow::Result;

use engai_core::config::Config;
use engai_core::db::Db;
use engai_core::sync::SyncEngine;

pub async fn run() -> Result<()> {
    let config = Config::load_global()?;
    let db = Arc::new(Db::new(&config.db_path()).await?);
    let engine = SyncEngine::new(db, &config.docs_path(), &config.prompts_path());
    engine.sync_all().await?;
    println!("Sync complete");
    Ok(())
}
