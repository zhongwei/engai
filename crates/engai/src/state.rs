use engai_core::ai::AiClient;
use engai_core::config::Config;
use engai_core::db::Db;
use engai_core::prompt::PromptEngine;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub config: Config,
    pub ai_client: Arc<AiClient>,
    pub prompt_engine: Arc<PromptEngine>,
}

impl AppState {
    pub fn new(db: Arc<Db>, config: Config) -> anyhow::Result<Self> {
        let ai_client = Arc::new(AiClient::from_config(&config)?);
        let prompt_engine = Arc::new(PromptEngine::new(config.prompts_path()));
        Ok(Self {
            db,
            config,
            ai_client,
            prompt_engine,
        })
    }
}
