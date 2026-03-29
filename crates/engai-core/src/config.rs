use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub ai: AiConfig,
    pub learning: LearningConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
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
        Self {
            server: ServerConfig::default(),
            ai: AiConfig::default(),
            learning: LearningConfig::default(),
            storage: StorageConfig::default(),
        }
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

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: "kimi".to_string(),
            api_key: String::new(),
            model: String::new(),
            base_url: String::new(),
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

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .expect("failed to determine home directory")
            .join(".engai")
    }

    pub fn config_file_path() -> PathBuf {
        Self::config_dir().join("config.toml")
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

    pub fn load_global() -> anyhow::Result<Self> {
        Self::load_from(Self::config_file_path())
    }

    pub fn load_from(path: PathBuf) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to(&self, path: &PathBuf) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn resolve_api_key(&self) -> String {
        if !self.ai.api_key.is_empty() {
            return self.ai.api_key.clone();
        }
        std::env::var("ENGAI_AI_API_KEY").unwrap_or_default()
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}
