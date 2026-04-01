use engai_core::ai::AiClient;
use engai_core::config::Config;
use engai_core::db::{
    ChatRepository, Db, ExampleRepository, NoteRepository, PhraseRepository, ReadingRepository,
    ReviewRepository, WordRepository,
};
use engai_core::prompt::PromptEngine;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub config: Config,
    pub ai_client: Arc<AiClient>,
    pub prompt_engine: Arc<PromptEngine>,
    pub word_repo: WordRepository,
    pub phrase_repo: PhraseRepository,
    pub example_repo: ExampleRepository,
    pub review_repo: ReviewRepository,
    pub reading_repo: ReadingRepository,
    pub note_repo: NoteRepository,
    pub chat_repo: ChatRepository,
}

impl AppState {
    pub fn new(db: Arc<Db>, config: Config) -> anyhow::Result<Self> {
        let ai_client = Arc::new(AiClient::from_config(&config)?);
        let prompt_engine = Arc::new(PromptEngine::new(config.prompts_path()));
        let pool = db.pool().clone();
        Ok(Self {
            db,
            config,
            ai_client,
            prompt_engine,
            word_repo: WordRepository::new(pool.clone()),
            phrase_repo: PhraseRepository::new(pool.clone()),
            example_repo: ExampleRepository::new(pool.clone()),
            review_repo: ReviewRepository::new(pool.clone()),
            reading_repo: ReadingRepository::new(pool.clone()),
            note_repo: NoteRepository::new(pool.clone()),
            chat_repo: ChatRepository::new(pool),
        })
    }
}
