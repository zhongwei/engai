use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const USER_CONFIG_DIR: &str = ".engai";
const USER_CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub model: String,
    pub provider: HashMap<String, ProviderConfig>,
    pub server: ServerConfig,
    pub learning: LearningConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(default)]
    pub options: Option<ProviderOptions>,
    #[serde(default)]
    pub models: HashMap<String, ModelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderOptions {
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub env: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLimit {
    #[serde(default)]
    pub context: Option<u32>,
    #[serde(default)]
    pub output: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelModalities {
    #[serde(default)]
    pub input: Vec<String>,
    #[serde(default)]
    pub output: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    #[serde(rename = "type", default)]
    pub thinking_type: Option<String>,
    #[serde(default)]
    pub budget_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    pub daily_new_words: i32,
    pub daily_review_limit: i32,
    pub default_deck: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[allow(dead_code)]
pub struct ResolvedModel {
    pub provider_name: String,
    pub model_id: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub config: ModelConfig,
}

#[allow(dead_code)]
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
        self.config.thinking.as_ref().and_then(|t| t.budget_tokens)
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
