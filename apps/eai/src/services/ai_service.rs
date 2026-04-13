use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use futures::Stream;

use crate::ai::{AiClient, ChatMessage};
use crate::prompt::PromptEngine;

pub type ChatStream = Pin<Box<dyn Stream<Item = Result<String>> + Send>>;

#[derive(Clone)]
pub struct AiService {
    client: Option<Arc<AiClient>>,
    prompt_engine: Arc<PromptEngine>,
}

impl AiService {
    #[allow(dead_code)]
    pub fn new(client: Option<Arc<AiClient>>, prompt_engine: Arc<PromptEngine>) -> Self {
        Self {
            client,
            prompt_engine,
        }
    }

    pub fn from_config(config: &crate::config::Config) -> Self {
        let client = AiClient::from_config(config).ok().map(Arc::new);
        let prompt_engine = Arc::new(PromptEngine::new(config.prompts_path()));
        Self {
            client,
            prompt_engine,
        }
    }

    fn require_client(&self) -> Result<&AiClient> {
        self.client
            .as_ref()
            .map(|c| c.as_ref())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "AI is not configured. Set ai.api_key in config or ENGAI_AI_API_KEY env var."
                )
            })
    }

    pub async fn explain_word(&self, word: &str) -> Result<String> {
        self.require_client()?
            .explain_word(word, &self.prompt_engine)
            .await
    }

    pub async fn explain_phrase(&self, phrase: &str) -> Result<String> {
        self.require_client()?
            .explain_phrase(phrase, &self.prompt_engine)
            .await
    }

    pub async fn analyze_reading(&self, content: &str) -> Result<String> {
        self.require_client()?
            .analyze_reading(content, &self.prompt_engine)
            .await
    }

    pub async fn chat_completion(&self, messages: &[ChatMessage]) -> Result<String> {
        self.require_client()?
            .chat_completion(messages)
            .await
    }

    pub async fn chat_completion_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatStream> {
        self.require_client()?
            .chat_completion_stream(messages)
            .await
    }
}
