# Model Configuration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor engai's model configuration to support multi-provider, per-model settings with limits, modalities, and thinking options.

**Architecture:** Replace flat `AiConfig` with multi-provider config structure. Add `ResolvedModel` helper for runtime access. Create embedded default config with pre-configured providers.

**Tech Stack:** Rust, serde, toml

---

## File Structure

| File | Action | Purpose |
|------|--------|---------|
| `.config/engai.toml` | Create | Default embedded config with providers |
| `apps/engai/src/config.rs` | Modify | New config structs, ResolvedModel, loading logic |
| `apps/engai/src/ai.rs` | Modify | Use ResolvedModel instead of AiConfig |
| `apps/engai/src/cli/cmd_config.rs` | Modify | Support new config structure |

---

### Task 1: Create Default Config File

**Files:**
- Create: `.config/engai.toml`

- [ ] **Step 1: Create the default config file**

```toml
# EngAI Configuration
#
# Default model: Uses "provider/model" format
# Example: model = "bailian-coding-plan/qwen3-max"

model = "bailian-coding-plan/qwen3-max"

# Provider: zhipuai-coding-plan (智谱AI Coding Plan)
# Set ZHIPUAI_API_KEY environment variable
[provider.zhipuai-coding-plan]
options = { base_url = "https://open.bigmodel.cn/api/coding/paas/v4", env = "ZHIPUAI_API_KEY" }

[provider.zhipuai-coding-plan.models]
"glm-5.1" = { limit = { context = 200000, output = 32768 }, thinking = { type = "enabled", budget_tokens = 16000 } }
"glm-4.7" = { limit = { context = 200000, output = 32768 }, thinking = { type = "enabled", budget_tokens = 16000 } }

# Provider: bailian-coding-plan (阿里云百炼 Model Studio)
# Set BAILIAN_API_KEY environment variable
[provider.bailian-coding-plan]
options = { base_url = "https://coding.dashscope.aliyuncs.com/apps/anthropic/v1", env = "BAILIAN_API_KEY" }

[provider.bailian-coding-plan.models]
"qwen3-max" = { name = "Qwen3 Max" }
"qwen3-coder-plus" = { name = "Qwen3 Coder Plus" }
"kimi-k2.5" = { name = "Kimi K2.5", modalities = { input = ["text", "image"], output = ["text"] }, thinking = { type = "enabled", budget_tokens = 1024 } }

# Provider: kimi (Moonshot AI)
# Set KIMI_API_KEY environment variable
[provider.kimi]
options = { base_url = "https://api.moonshot.cn/v1", env = "KIMI_API_KEY" }

[provider.kimi.models]
"moonshot-v1-8k" = { name = "Moonshot V1 8K", limit = { context = 8192, output = 4096 } }
"moonshot-v1-32k" = { name = "Moonshot V1 32K", limit = { context = 32768, output = 4096 } }
"moonshot-v1-128k" = { name = "Moonshot V1 128K", limit = { context = 131072, output = 4096 } }

# Server configuration
[server]
port = 3000
host = "127.0.0.1"

# Learning configuration
[learning]
daily_new_words = 20
daily_review_limit = 100
default_deck = "01_vocab"

# Storage configuration
[storage]
db_path = "~/.engai/engai.db"
docs_path = "./docs"
prompts_path = "./prompts"
```

- [ ] **Step 2: Commit**

```bash
git add .config/engai.toml
git commit -m "feat(config): add default embedded config with providers"
```

---

### Task 2: Update Config Structs

**Files:**
- Modify: `apps/engai/src/config.rs`

- [ ] **Step 1: Replace the entire config.rs with new structure**

```rust
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

const USER_CONFIG_DIR: &str = ".engai";
const USER_CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub model: String,
    pub provider: HashMap<String, ProviderConfig>,
    pub server: ServerConfig,
    pub learning: LearningConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    #[serde(default)]
    pub options: Option<ProviderOptions>,
    #[serde(default)]
    pub models: HashMap<String, ModelConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderOptions {
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub env: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ModelConfig {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub limit: Option<ModelLimit>,
    #[serde(default)]
    pub modalities: Option<ModelModalities>,
    #[serde(default)]
    pub thinking: Option<ThinkingConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelLimit {
    #[serde(default)]
    pub context: Option<u32>,
    #[serde(default)]
    pub output: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelModalities {
    #[serde(default)]
    pub input: Vec<String>,
    #[serde(default)]
    pub output: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThinkingConfig {
    #[serde(rename = "type", default)]
    pub thinking_type: Option<String>,
    #[serde(default)]
    pub budget_tokens: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LearningConfig {
    pub daily_new_words: i32,
    pub daily_review_limit: i32,
    pub default_deck: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub db_path: String,
    pub docs_path: String,
    #[serde(default = "default_prompts_path")]
    pub prompts_path: String,
}

fn default_prompts_path() -> String {
    "./prompts".to_string()
}

impl Default for Config {
    fn default() -> Self {
        let default_config = include_str!("../../../.config/engai.toml");
        toml::from_str(default_config).unwrap_or_else(|e| panic!("Invalid default config: {e}"))
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "127.0.0.1".to_string(),
        }
    }
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            daily_new_words: 20,
            daily_review_limit: 100,
            default_deck: "01_vocab".to_string(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            db_path: "~/.engai/engai.db".to_string(),
            docs_path: "./docs".to_string(),
            prompts_path: "./prompts".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedModel {
    pub provider_name: String,
    pub model_id: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub config: ModelConfig,
}

impl ResolvedModel {
    pub fn context_limit(&self) -> u32 {
        self.config
            .limit
            .as_ref()
            .and_then(|l| l.context)
            .unwrap_or(128_000)
    }

    pub fn output_limit(&self) -> u32 {
        self.config
            .limit
            .as_ref()
            .and_then(|l| l.output)
            .unwrap_or(16_384)
    }

    pub fn supports_thinking(&self) -> bool {
        self.config
            .thinking
            .as_ref()
            .is_some_and(|t| t.thinking_type.as_deref().unwrap_or("") == "enabled")
    }

    pub fn supports_vision(&self) -> bool {
        self.config
            .modalities
            .as_ref()
            .is_some_and(|m| m.input.iter().any(|i| i == "image"))
    }

    pub fn thinking_budget(&self) -> Option<u32> {
        self.config
            .thinking
            .as_ref()
            .and_then(|t| t.budget_tokens)
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .expect("failed to determine home directory")
            .join(USER_CONFIG_DIR)
    }

    pub fn config_file_path() -> PathBuf {
        Self::config_dir().join(USER_CONFIG_FILE)
    }

    pub fn db_path(&self) -> PathBuf {
        expand_tilde(&self.storage.db_path)
    }

    pub fn docs_path(&self) -> PathBuf {
        expand_tilde(&self.storage.docs_path)
    }

    pub fn prompts_path(&self) -> PathBuf {
        expand_tilde(&self.storage.prompts_path)
    }

    pub fn load_global() -> Result<Self> {
        let default_config = Self::default();
        let user_config_path = Self::config_file_path();

        if user_config_path.exists() {
            let user_content = std::fs::read_to_string(&user_config_path)?;
            let user_config: Config = toml::from_str(&user_content)?;
            Ok(default_config.merge(user_config))
        } else {
            Ok(default_config)
        }
    }

    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn resolve_model(&self) -> Result<ResolvedModel> {
        let (provider_name, model_id) = Self::parse_model_ref(&self.model)?;

        let provider = self
            .provider
            .get(&provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", &provider_name))?;

        let model_config = provider.models.get(&model_id).ok_or_else(|| {
            anyhow::anyhow!(
                "Model '{}' not found in provider '{}'",
                &model_id,
                &provider_name
            )
        })?;

        let api_key = Self::get_api_key(&provider_name, provider);
        if api_key.is_none() {
            anyhow::bail!(
                "API key not found for provider '{}'. Set {}_API_KEY env var or configure provider.options.api_key",
                &provider_name,
                provider_name.to_uppercase().replace('-', "_")
            );
        }

        Ok(ResolvedModel {
            provider_name,
            model_id,
            base_url: provider.options.as_ref().and_then(|o| o.base_url.clone()),
            api_key,
            config: model_config.clone(),
        })
    }

    fn parse_model_ref(model_ref: &str) -> Result<(String, String)> {
        let mut parts = model_ref.splitn(2, '/');
        let provider = parts.next().ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid model reference '{}', expected format 'provider/model'",
                model_ref
            )
        })?;
        let model = parts.next().ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid model reference '{}', expected format 'provider/model'",
                model_ref
            )
        })?;
        Ok((provider.to_string(), model.to_string()))
    }

    fn get_api_key(provider_name: &str, provider: &ProviderConfig) -> Option<String> {
        if let Some(options) = &provider.options {
            if let Some(env_var) = &options.env {
                if let Ok(key) = std::env::var(env_var) {
                    return Some(key);
                }
            }
            if let Some(key) = &options.api_key {
                if !key.is_empty() {
                    return Some(key.clone());
                }
            }
        }

        let env_var = format!("{}_API_KEY", provider_name.to_uppercase().replace('-', "_"));
        std::env::var(&env_var).ok()
    }

    fn merge(self, other: Self) -> Self {
        let mut providers = self.provider;
        for (name, config) in other.provider {
            providers.insert(name, config);
        }

        Self {
            provider: providers,
            model: if other.model.is_empty() {
                self.model
            } else {
                other.model
            },
            server: ServerConfig {
                port: if other.server.port == 0 {
                    self.server.port
                } else {
                    other.server.port
                },
                host: if other.server.host.is_empty() {
                    self.server.host
                } else {
                    other.server.host
                },
            },
            learning: LearningConfig {
                daily_new_words: if other.learning.daily_new_words == 0 {
                    self.learning.daily_new_words
                } else {
                    other.learning.daily_new_words
                },
                daily_review_limit: if other.learning.daily_review_limit == 0 {
                    self.learning.daily_review_limit
                } else {
                    other.learning.daily_review_limit
                },
                default_deck: if other.learning.default_deck.is_empty() {
                    self.learning.default_deck
                } else {
                    other.learning.default_deck
                },
            },
            storage: StorageConfig {
                db_path: if other.storage.db_path.is_empty() {
                    self.storage.db_path
                } else {
                    other.storage.db_path
                },
                docs_path: if other.storage.docs_path.is_empty() {
                    self.storage.docs_path
                } else {
                    other.storage.docs_path
                },
                prompts_path: if other.storage.prompts_path.is_empty() {
                    self.storage.prompts_path
                } else {
                    other.storage.prompts_path
                },
            },
        }
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}
```

- [ ] **Step 2: Commit**

```bash
git add apps/engai/src/config.rs
git commit -m "feat(config): add multi-provider model configuration"
```

---

### Task 3: Update AiClient

**Files:**
- Modify: `apps/engai/src/ai.rs`

- [ ] **Step 1: Update AiClient to use ResolvedModel**

Replace the `from_config` method and remove the `IfEmpty` trait:

```rust
use anyhow::{Context, Result};
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::{Config, ResolvedModel};
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
        
        let api_key = resolved
            .api_key
            .ok_or_else(|| anyhow::anyhow!("API key required for provider '{}'", resolved.provider_name))?;

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
```

- [ ] **Step 2: Commit**

```bash
git add apps/engai/src/ai.rs
git commit -m "feat(ai): use ResolvedModel for AI client initialization"
```

---

### Task 4: Update CLI Config Commands

**Files:**
- Modify: `apps/engai/src/cli/cmd_config.rs`

- [ ] **Step 1: Update config CLI for new structure**

```rust
use anyhow::Result;

use crate::config::Config;

#[derive(clap::Subcommand)]
pub enum ConfigAction {
    Init,
    Set { key: String, value: String },
    Get { key: String },
    List,
}

pub async fn run(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Init => {
            let config_path = Config::config_file_path();
            if config_path.exists() {
                println!("Config already exists at {}", config_path.display());
                return Ok(());
            }
            let config = Config::default();
            config.save_to(&config_path)?;
            println!("Config created at {}", config_path.display());
        }
        ConfigAction::Set { key, value } => {
            let config_path = Config::config_file_path();
            if !config_path.exists() {
                let config = Config::default();
                config.save_to(&config_path)?;
            }
            let mut config = Config::load_global()?;
            set_config_value(&mut config, &key, &value)?;
            config.save_to(&config_path)?;
            println!("Set {} = {}", key, value);
        }
        ConfigAction::Get { key } => {
            let config = Config::load_global()?;
            let value = get_config_value(&config, &key);
            println!("{} = {}", key, value);
        }
        ConfigAction::List => {
            let config = Config::load_global()?;
            println!("model = {}", config.model);
            println!("\n[server]");
            println!("port = {}", config.server.port);
            println!("host = {}", config.server.host);
            println!("\n[learning]");
            println!("daily_new_words = {}", config.learning.daily_new_words);
            println!("daily_review_limit = {}", config.learning.daily_review_limit);
            println!("default_deck = {}", config.learning.default_deck);
            println!("\n[storage]");
            println!("db_path = {}", config.storage.db_path);
            println!("docs_path = {}", config.storage.docs_path);
            println!("prompts_path = {}", config.storage.prompts_path);
            println!("\n[providers]");
            for (name, provider) in &config.provider {
                println!("  {} ({} models)", name, provider.models.len());
            }
        }
    }

    Ok(())
}

fn set_config_value(config: &mut Config, key: &str, value: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["model"] => config.model = value.to_string(),
        ["server", "port"] => config.server.port = value.parse()?,
        ["server", "host"] => config.server.host = value.to_string(),
        ["learning", "daily_new_words"] => config.learning.daily_new_words = value.parse()?,
        ["learning", "daily_review_limit"] => config.learning.daily_review_limit = value.parse()?,
        ["learning", "default_deck"] => config.learning.default_deck = value.to_string(),
        ["storage", "db_path"] => config.storage.db_path = value.to_string(),
        ["storage", "docs_path"] => config.storage.docs_path = value.to_string(),
        _ => anyhow::bail!("Unknown config key: {}", key),
    }
    Ok(())
}

fn get_config_value(config: &Config, key: &str) -> String {
    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["model"] => config.model.clone(),
        ["server", "port"] => config.server.port.to_string(),
        ["server", "host"] => config.server.host.clone(),
        ["learning", "daily_new_words"] => config.learning.daily_new_words.to_string(),
        ["learning", "daily_review_limit"] => config.learning.daily_review_limit.to_string(),
        ["learning", "default_deck"] => config.learning.default_deck.clone(),
        ["storage", "db_path"] => config.storage.db_path.clone(),
        ["storage", "docs_path"] => config.storage.docs_path.clone(),
        _ => format!("(unknown key: {})", key),
    }
}
```

- [ ] **Step 2: Update main.rs to add List command**

Add `List` variant to the Config command in `apps/engai/src/main.rs`:

```rust
Config {
    #[command(subcommand)]
    action: cli::cmd_config::ConfigAction,
},
```

The `ConfigAction` enum already has `List` added in step 1.

- [ ] **Step 3: Commit**

```bash
git add apps/engai/src/cli/cmd_config.rs apps/engai/src/main.rs
git commit -m "feat(cli): update config commands for multi-provider model"
```

---

### Task 5: Build and Test

**Files:**
- None (verification only)

- [ ] **Step 1: Build the project**

```bash
cd C:/Dev/engai && cargo build
```

Expected: Build succeeds without errors

- [ ] **Step 2: Run basic config test**

```bash
cd C:/Dev/engai && cargo run -- config list
```

Expected: Shows config with providers listed

- [ ] **Step 3: Test model resolution**

```bash
cd C:/Dev/engai && cargo run -- config get model
```

Expected: Shows `model = bailian-coding-plan/qwen3-max`

- [ ] **Step 4: Commit any fixes if needed**

---

### Task 6: Final Verification

- [ ] **Step 1: Run clippy**

```bash
cd C:/Dev/engai && cargo clippy -- -D warnings
```

Expected: No warnings

- [ ] **Step 2: Run tests if any**

```bash
cd C:/Dev/engai && cargo test
```

Expected: All tests pass

- [ ] **Step 3: Final commit message**

```bash
git add -A
git commit -m "feat(config): complete multi-provider model configuration"
```
