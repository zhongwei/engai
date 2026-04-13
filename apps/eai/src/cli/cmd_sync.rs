use anyhow::Result;

use crate::config::Config;
use crate::db::Db;
use crate::services::SyncService;
use std::sync::Arc;

pub async fn run() -> Result<()> {
    let config = Config::load_global()?;
    let db = Arc::new(Db::new(&config.db_path()).await?);
    let sync_service = SyncService::new(db, &config.docs_path(), &config.prompts_path());
    sync_service.sync_all().await?;
    println!("Sync complete");
    Ok(())
}
