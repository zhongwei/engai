use std::sync::Arc;

use crate::db::{Db, ExampleRepository, NoteRepository, PhraseRepository, ReviewRepository, WordRepository};
use crate::sync_db_adapter::SyncDbAdapter;
use esync::SyncEngine;

#[derive(Clone)]
pub struct SyncService {
    engine: Arc<SyncEngine<SyncDbAdapter>>,
}

impl SyncService {
    pub fn new(db: Arc<Db>, docs_path: &std::path::Path, _prompts_path: &std::path::Path) -> Self {
        let pool = db.pool().clone();
        let adapter = SyncDbAdapter::new(
            WordRepository::new(pool.clone()),
            PhraseRepository::new(pool.clone()),
            ExampleRepository::new(pool.clone()),
            NoteRepository::new(pool.clone()),
            ReviewRepository::new(pool),
        );
        Self {
            engine: Arc::new(SyncEngine::new(adapter, docs_path.to_path_buf())),
        }
    }

    pub async fn sync_all(&self) -> anyhow::Result<()> {
        self.engine.sync_all().await
    }
}
