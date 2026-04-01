use std::path::Path;
use std::sync::Arc;

use crate::db::Db;
use crate::sync::SyncEngine;

#[derive(Clone)]
pub struct SyncService {
    engine: Arc<SyncEngine>,
}

impl SyncService {
    pub fn new(db: Arc<Db>, docs_path: &Path, prompts_path: &Path) -> Self {
        Self {
            engine: Arc::new(SyncEngine::new(db, docs_path, prompts_path)),
        }
    }

    pub async fn sync_all(&self) -> anyhow::Result<()> {
        self.engine.sync_all().await
    }
}
