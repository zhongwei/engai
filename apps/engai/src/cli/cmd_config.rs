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
