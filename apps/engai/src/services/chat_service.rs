use crate::db::ChatRepository;
use crate::error::{AppError, Result};
use crate::models::ChatEntry;
use crate::services::AiService;

#[derive(Clone)]
#[allow(dead_code)]
pub struct ChatService {
    chat_repo: ChatRepository,
    ai: AiService,
}

impl ChatService {
    pub fn new(chat_repo: ChatRepository, ai: AiService) -> Self {
        Self { chat_repo, ai }
    }

    pub async fn add_message(&self, role: &str, content: &str) -> Result<ChatEntry> {
        Ok(self.chat_repo.add_chat_message(role, content).await?)
    }

    pub async fn get_recent(&self, limit: i64) -> Result<Vec<ChatEntry>> {
        Ok(self.chat_repo.get_recent_chat(limit).await?)
    }

    #[allow(dead_code)]
    pub async fn send_message(&self, content: &str) -> Result<String> {
        self.chat_repo.add_chat_message("user", content).await?;

        let recent = self.chat_repo.get_recent_chat(20).await?;

        let mut messages: Vec<crate::ai::ChatMessage> = recent
            .into_iter()
            .rev()
            .map(|e| crate::ai::ChatMessage {
                role: e.role,
                content: e.content,
            })
            .collect();

        messages.insert(
            0,
            crate::ai::ChatMessage {
                role: "system".to_string(),
                content: "You are an English learning assistant. Help the user improve their English skills.".to_string(),
            },
        );

        let response = self
            .ai
            .chat_completion(&messages)
            .await
            .map_err(|e| AppError::AiError(e.to_string()))?;

        self.chat_repo
            .add_chat_message("assistant", &response)
            .await?;

        Ok(response)
    }

    #[allow(dead_code)]
    pub async fn clear(&self) -> Result<()> {
        self.chat_repo.clear_chat().await?;
        Ok(())
    }
}
