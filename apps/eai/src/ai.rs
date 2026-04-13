use anyhow::{Context, Result};
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::prompt::PromptEngine;

#[derive(Debug, Clone)]
pub struct AiClient {
    client: Client,
    #[allow(dead_code)]
    provider: String,
    api_key: String,
    model: String,
    base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

impl AiClient {
    pub fn from_config(config: &Config) -> Result<Self> {
        let resolved = config.resolve_model()?;
        
        let base_url = resolved
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
        
        // Safe: resolve_model() errors if api_key is None
        let api_key = resolved.api_key.unwrap();

        let client = Client::new();

        Ok(Self {
            client,
            provider: resolved.provider_name,
            api_key,
            model: resolved.model_id,
            base_url,
        })
    }

    pub async fn explain_word(
        &self,
        word: &str,
        prompt_engine: &PromptEngine,
    ) -> Result<String> {
        let user_prompt = prompt_engine
            .render("explain_word.md", &[("word", word)])
            .await?;
        let system_prompt =
            "You are a professional English teacher. Explain the word clearly and concisely."
                .to_string();
        self.chat_completion(&[
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ])
        .await
    }

    pub async fn explain_phrase(
        &self,
        phrase: &str,
        prompt_engine: &PromptEngine,
    ) -> Result<String> {
        let user_prompt = prompt_engine
            .render("explain_phrase.md", &[("phrase", phrase)])
            .await?;
        let system_prompt =
            "You are a professional English teacher. Explain the phrase clearly and concisely."
                .to_string();
        self.chat_completion(&[
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ])
        .await
    }

    pub async fn analyze_reading(
        &self,
        content: &str,
        prompt_engine: &PromptEngine,
    ) -> Result<String> {
        let user_prompt = prompt_engine
            .render("reading_analyze.md", &[("content", content)])
            .await?;
        let system_prompt =
            "You are a professional English teacher. Analyze the reading passage thoroughly."
                .to_string();
        self.chat_completion(&[
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ])
        .await
    }

    pub async fn chat_completion(&self, messages: &[ChatMessage]) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = ChatRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .with_context(|| "Failed to send chat completion request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Chat completion failed ({}): {}", status, body);
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .with_context(|| "Failed to parse chat completion response")?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No choices in chat completion response"))
    }

    pub async fn chat_completion_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Result<String>> + Send>>> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: true,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .with_context(|| "Failed to send streaming chat completion request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Streaming chat completion failed ({}): {}", status, body);
        }

        let byte_stream = response.bytes_stream();

        let stream = byte_stream
            .map(|chunk| {
                let chunk = chunk.with_context(|| "Failed to read stream chunk")?;
                let text = String::from_utf8_lossy(&chunk);
                let content = parse_sse_lines(&text);
                Ok(content)
            })
            .filter_map(|result| async move {
                match result {
                    Ok(content) if content.is_empty() => None,
                    Ok(content) => Some(Ok(content)),
                    Err(e) => Some(Err(e)),
                }
            });

        Ok(Box::pin(stream) as std::pin::Pin<Box<dyn futures::Stream<Item = Result<String>> + Send>>)
    }
}

fn parse_sse_lines(text: &str) -> String {
    let mut content = String::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(data) = line.strip_prefix("data: ") {
            if data.trim() == "[DONE]" {
                break;
            }
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(delta) = json
                    .get("choices")
                    .and_then(|c| c.get(0))
                    .and_then(|c| c.get("delta"))
                    .and_then(|d| d.get("content"))
                {
                    if let Some(s) = delta.as_str() {
                        content.push_str(s);
                    }
                }
            }
        }
    }
    content
}
