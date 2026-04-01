use crate::config::Config;
use crate::db::Db;
use crate::services::{
    AiService, ChatService, NoteService, PhraseService, ReadingService, ReviewService, Services,
    StatsService, SyncService, WordService,
};
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub config: Config,
    pub ai_service: AiService,
    pub word_service: WordService,
    pub phrase_service: PhraseService,
    pub review_service: ReviewService,
    pub reading_service: ReadingService,
    pub note_service: NoteService,
    pub chat_service: ChatService,
    pub stats_service: StatsService,
    pub sync_service: SyncService,
}

impl AppState {
    pub fn new(db: Arc<Db>, config: Config) -> Self {
        let ai = AiService::from_config(&config);
        let pool = db.pool().clone();
        let services = Services::new(pool, ai.clone());

        let sync_service =
            SyncService::new(db.clone(), &config.docs_path(), &config.prompts_path());

        Self {
            db,
            config,
            ai_service: ai,
            word_service: services.word,
            phrase_service: services.phrase,
            review_service: services.review,
            reading_service: services.reading,
            note_service: services.note,
            chat_service: services.chat,
            stats_service: services.stats,
            sync_service,
        }
    }

    pub fn from_services(db: Arc<Db>, config: Config, services: Services, ai: AiService) -> Self {
        let sync_service =
            SyncService::new(db.clone(), &config.docs_path(), &config.prompts_path());
        Self {
            db,
            config,
            ai_service: ai,
            word_service: services.word,
            phrase_service: services.phrase,
            review_service: services.review,
            reading_service: services.reading,
            note_service: services.note,
            chat_service: services.chat,
            stats_service: services.stats,
            sync_service,
        }
    }
}
