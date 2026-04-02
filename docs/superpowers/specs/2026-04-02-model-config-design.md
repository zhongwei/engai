# Model Configuration Design for engai

## Summary

Refactor engai's model configuration to adopt zagent's multi-provider, per-model configuration pattern. This enables support for multiple AI providers with per-model limits, modalities, and thinking options.

## Motivation

Current engai configuration uses a flat `AiConfig` with single provider/model support. This limits flexibility when:
- Switching between different AI providers
- Using models with different context/output limits
- Enabling model-specific features (thinking, vision)
- Managing API keys for multiple providers

## Design

### Config Structure

Replace `AiConfig` with multi-provider configuration:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub model: String,  // "provider/model-id" format
    pub provider: HashMap<String, ProviderConfig>,
    pub server: ServerConfig,
    pub learning: LearningConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    pub options: Option<ProviderOptions>,
    pub models: HashMap<String, ModelConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderOptions {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub env: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfig {
    pub name: Option<String>,
    pub limit: Option<ModelLimit>,
    pub modalities: Option<ModelModalities>,
    pub thinking: Option<ThinkingConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelLimit {
    pub context: Option<u32>,
    pub output: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelModalities {
    pub input: Vec<String>,
    pub output: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThinkingConfig {
    #[serde(rename = "type")]
    pub thinking_type: Option<String>,
    pub budget_tokens: Option<u32>,
}
```

### ResolvedModel Helper

Runtime helper for convenient model access:

```rust
#[derive(Debug, Clone)]
pub struct ResolvedModel {
    pub provider_name: String,
    pub model_id: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub config: ModelConfig,
}

impl ResolvedModel {
    pub fn context_limit(&self) -> u32;
    pub fn output_limit(&self) -> u32;
    pub fn supports_thinking(&self) -> bool;
    pub fn supports_vision(&self) -> bool;
    pub fn thinking_budget(&self) -> Option<u32>;
}
```

### Config Loading

1. **Default Config**: Embedded via `include_str!("../../../.config/engai.toml")`
2. **User Config**: `~/.engai/config.toml` (optional override)
3. **Merge Logic**: User config merges into defaults (providers added, model overridden)
4. **Model Resolution**: `Config::resolve_model()` parses "provider/model" and returns `ResolvedModel`

### API Key Resolution

Priority order:
1. `env` variable specified in provider options
2. Explicit `api_key` in provider options
3. `{PROVIDER_NAME}_API_KEY` environment variable (uppercase, dashes to underscores)

### Default Config File

Create `.config/engai.toml` with pre-configured providers:

```toml
model = "bailian-coding-plan/qwen3-max"

[provider.zhipuai-coding-plan]
options = { base_url = "https://open.bigmodel.cn/api/coding/paas/v4", env = "ZHIPUAI_API_KEY" }

[provider.zhipuai-coding-plan.models]
"glm-5.1" = { limit = { context = 200000, output = 32768 }, thinking = { type = "enabled", budget_tokens = 16000 } }

[provider.bailian-coding-plan]
options = { base_url = "https://coding.dashscope.aliyuncs.com/apps/anthropic/v1", env = "BAILIAN_API_KEY" }

[provider.bailian-coding-plan.models]
"qwen3-max" = { name = "Qwen3 Max" }
"qwen3-coder-plus" = { name = "Qwen3 Coder Plus" }
"kimi-k2.5" = { name = "Kimi K2.5", modalities = { input = ["text", "image"], output = ["text"] }, thinking = { type = "enabled", budget_tokens = 1024 } }
```

### CLI Updates

Update `cmd_config.rs` to support new structure:

```bash
engai config set model "bailian-coding-plan/qwen3-max"
engai config get model
engai config set provider.bailian-coding-plan.options.api_key "sk-xxx"
```

## Migration

- Remove `AiConfig` struct
- Update code accessing `config.ai.*` to use `config.resolve_model()`
- Existing user configs in `~/.engai/config.toml` need manual migration

## Files Changed

1. `apps/engai/src/config.rs` - New config structures, loading logic, ResolvedModel
2. `apps/engai/src/cli/cmd_config.rs` - Updated CLI commands
3. `.config/engai.toml` - New default config file
4. Any files using `config.ai.*` - Update to use `resolve_model()`

## Testing

- Unit tests for config loading and merging
- Unit tests for model resolution
- Unit tests for API key resolution priority
- Integration test for CLI config commands
