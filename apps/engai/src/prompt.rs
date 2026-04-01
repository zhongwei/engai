use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct PromptEngine {
    prompts_dir: PathBuf,
}

impl PromptEngine {
    pub fn new(prompts_dir: PathBuf) -> Self {
        Self { prompts_dir }
    }

    pub async fn render(&self, template_name: &str, vars: &[(&str, &str)]) -> Result<String> {
        let path = self.prompts_dir.join(template_name);
        if !path.exists() {
            anyhow::bail!("Prompt template not found: {}", path.display());
        }
        let content = tokio::fs::read_to_string(&path).await?;
        let mut rendered = content;
        for (key, value) in vars {
            rendered = rendered.replace(&format!("{{{{{}}}}}", key), value);
        }
        Ok(rendered)
    }

    pub async fn load_raw(&self, template_name: &str) -> Result<String> {
        let path = self.prompts_dir.join(template_name);
        tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("Failed to load prompt: {}", path.display()))
    }
}
