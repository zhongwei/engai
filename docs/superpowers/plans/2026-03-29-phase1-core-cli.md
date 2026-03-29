# Engai Phase 1: Core + CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the complete `engai-core` library and CLI binary with all subcommands, AI integration, bidirectional Markdown sync, and spaced repetition — producing a fully functional CLI-based English learning system.

**Architecture:** Cargo workspace with two crates: `engai-core` (shared library: DB, Markdown, sync, review, AI, config) and `engai` (CLI binary). Phase 2/3 will add the Axum server and ratatui TUI to the same binary.

**Tech Stack:** Rust (axum 0.8.8, sqlx 0.8.6, clap 4.6.0, serde 1.0.228, reqwest 0.13.2, tokio 1.50.0, chrono, toml, pulldown-cmark, gray_matter, tower-http, tracing, anyhow, thiserror), SQLite, Markdown

---

## File Structure

```
engai/
├── Cargo.toml                          # workspace root
├── crates/
│   ├── engai-core/
│   │   ├── Cargo.toml
│   │   ├── migrations/
│   │   │   └── 001_init.sql            # full schema
│   │   └── src/
│   │       ├── lib.rs                  # pub mod declarations
│   │       ├── models.rs               # Word, Phrase, Example, Review, Reading, Note
│   │       ├── db.rs                   # init, connection pool, CRUD operations
│   │       ├── config.rs               # Config struct, load/save, defaults
│   │       ├── markdown.rs             # parse Markdown files, generate Markdown from models
│   │       ├── sync.rs                 # bidirectional sync engine
│   │       ├── review.rs               # SM-2 algorithm
│   │       ├── ai.rs                   # LLM client (Kimi/OpenAI), streaming
│   │       └── prompt.rs               # template loading, variable interpolation
│   └── engai/
│       ├── Cargo.toml
│       ├── build.rs                    # future: embed frontend
│       └── src/
│           ├── main.rs                 # clap CLI entry point
│           ├── cmd_add.rs              # add word / add phrase
│           ├── cmd_explain.rs          # explain word / explain phrase
│           ├── cmd_review.rs           # review / review --all
│           ├── cmd_sync.rs             # sync
│           ├── cmd_read.rs             # read <file>
│           ├── cmd_import.rs           # import
│           ├── cmd_export.rs           # export
│           ├── cmd_stats.rs            # stats
│           ├── cmd_config.rs           # config init/set/get
│           └── cmd_note.rs             # note add
├── docs/                               # Markdown notes (user content)
│   ├── 01_vocab/
│   ├── 02_phrases/
│   ├── 03_reading/
│   └── 99_review/
├── prompts/
│   ├── explain_word.md
│   ├── explain_phrase.md
│   ├── reading_analyze.md
│   └── chat_english.md
└── .sqlx/                              # offline mode query data (generated)
```

---

### Task 1: Cargo Workspace + Project Skeleton

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/engai-core/Cargo.toml`
- Create: `crates/engai-core/src/lib.rs`
- Create: `crates/engai/Cargo.toml`
- Create: `crates/engai/src/main.rs`
- Create: `crates/engai/build.rs`
- Create: `docs/01_vocab/.gitkeep`
- Create: `docs/02_phrases/.gitkeep`
- Create: `docs/03_reading/.gitkeep`
- Create: `docs/99_review/.gitkeep`
- Create: `prompts/explain_word.md`
- Create: `prompts/explain_phrase.md`
- Create: `prompts/reading_analyze.md`
- Create: `prompts/chat_english.md`

- [ ] **Step 1: Create workspace root Cargo.toml**

```toml
[workspace]
resolver = "2"
members = ["crates/engai-core", "crates/engai"]

[workspace.dependencies]
tokio = { version = "1.50.0", features = ["full"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

- [ ] **Step 2: Create engai-core Cargo.toml**

```toml
[package]
name = "engai-core"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
sqlx = { version = "0.8.6", features = ["runtime-tokio", "sqlite", "migrate", "chrono"] }
reqwest = { version = "0.13.2", features = ["json", "stream"] }
toml = "0.8"
pulldown-cmark = "0.13"
gray_matter = "0.2"
dirs = "6"
futures = "0.3"
uuid = { version = "1", features = ["v4"] }
tokio-stream = "0.1"
```

- [ ] **Step 3: Create engai-core/src/lib.rs**

```rust
pub mod ai;
pub mod config;
pub mod db;
pub mod markdown;
pub mod models;
pub mod prompt;
pub mod review;
pub mod sync;
```

- [ ] **Step 4: Create engai Cargo.toml**

```toml
[package]
name = "engai"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "engai"
path = "src/main.rs"

[dependencies]
engai-core = { path = "../engai-core" }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
clap = { version = "4.6.0", features = ["derive"] }
```

- [ ] **Step 5: Create engai/src/main.rs (minimal placeholder)**

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "engai", about = "AI English Learning System")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Add a word or phrase
    Add { target: String },
    /// Explain a word or phrase with AI
    Explain { target: String },
    /// Start today's review
    Review { #[arg(long)] all: bool },
    /// Sync Markdown ↔ SQLite
    Sync,
    /// Import and analyze a reading
    Read { file: String },
    /// Import Markdown notes
    Import { path: String },
    /// Export to Markdown
    Export {
        #[arg(long)]
        word: Option<String>,
        #[arg(long)]
        phrase: Option<String>,
        #[arg(long)]
        all: bool,
    },
    /// Show learning statistics
    Stats,
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Manage notes
    Note {
        #[command(subcommand)]
        action: NoteAction,
    },
    /// Start web server only
    #[command(alias = "-s")]
    Server {
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

#[derive(clap::Subcommand)]
enum ConfigAction {
    /// Initialize configuration interactively
    Init,
    /// Set a config value
    Set { key: String, value: String },
    /// Get a config value
    Get { key: String },
}

#[derive(clap::Subcommand)]
enum NoteAction {
    /// Add a note to a target
    Add {
        target_type: String,
        target_id: i64,
        content: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::init();
    let _cli = Cli::parse();
    println!("engai: placeholder");
    Ok(())
}
```

- [ ] **Step 6: Create build.rs (placeholder for Phase 2 frontend embedding)**

```rust
fn main() {
    println!("cargo:rerun-if-changed=../frontend/dist");
}
```

- [ ] **Step 7: Create stub modules in engai-core**

For `models.rs`, `db.rs`, `config.rs`, `markdown.rs`, `sync.rs`, `review.rs`, `ai.rs`, `prompt.rs` — create minimal files with no-impl stubs so the project compiles:

`models.rs`:
```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Word {
    pub id: i64,
    pub word: String,
    pub phonetic: Option<String>,
    pub meaning: Option<String>,
    pub familiarity: i32,
    pub next_review: Option<NaiveDateTime>,
    pub interval: i32,
    pub ease_factor: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Phrase {
    pub id: i64,
    pub phrase: String,
    pub meaning: Option<String>,
    pub familiarity: i32,
    pub next_review: Option<NaiveDateTime>,
    pub interval: i32,
    pub ease_factor: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Example {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub sentence: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Review {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub quality: i32,
    pub reviewed_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Reading {
    pub id: i64,
    pub title: Option<String>,
    pub content: String,
    pub source: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewWord {
    pub word: String,
    pub phonetic: Option<String>,
    pub meaning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPhrase {
    pub phrase: String,
    pub meaning: Option<String>,
}
```

For the remaining stubs (`db.rs`, `config.rs`, `markdown.rs`, `sync.rs`, `review.rs`, `ai.rs`, `prompt.rs`), create each with:
```rust
// Module stub — will be implemented in subsequent tasks
```

- [ ] **Step 8: Create .gitkeep files for docs directories**

Create empty `.gitkeep` in `docs/01_vocab/`, `docs/02_phrases/`, `docs/03_reading/`, `docs/99_review/`.

- [ ] **Step 9: Create prompt template files**

`prompts/explain_word.md`:
```markdown
You are an English teacher.

Explain the word: {{word}}

Requirements:
1. Simple English
2. Give 3 examples with context
3. Compare with similar words
4. Output in Markdown
```

`prompts/explain_phrase.md`:
```markdown
You are an English teacher.

Explain the phrase: {{phrase}}

Requirements:
1. Simple English
2. Give 3 examples with context
3. Explain different usage contexts if applicable
4. Output in Markdown
```

`prompts/reading_analyze.md`:
```markdown
You are an English reading assistant.

Analyze the following text:

{{content}}

Requirements:
1. Extract vocabulary words (with definitions)
2. Analyze key sentences (grammar and meaning)
3. Write a summary in simple English
4. Output in Markdown
```

`prompts/chat_english.md`:
```markdown
You are an English conversation partner. Help the user practice English.

Rules:
1. Respond in English
2. If the user makes mistakes, gently correct them
3. Keep conversations natural and engaging
4. Ask follow-up questions to keep the conversation going
```

- [ ] **Step 10: Verify project compiles**

Run: `cargo check`
Expected: Compiles without errors

- [ ] **Step 11: Commit**

```bash
git add -A
git commit -m "feat: scaffold cargo workspace with engai-core and engai crates"
```

---

### Task 2: Configuration System

**Files:**
- Modify: `crates/engai-core/src/config.rs`
- Create: `crates/engai-core/tests/test_config.rs`

- [ ] **Step 1: Write failing test**

`crates/engai-core/tests/test_config.rs`:
```rust
use engai_core::config::Config;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.server.port, 3000);
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.ai.provider, "kimi");
}

#[test]
fn test_config_dir() {
    let dir = Config::config_dir();
    assert!(dir.ends_with(".engai"));
}

#[test]
fn test_config_file_path() {
    let path = Config::config_file_path();
    assert!(path.to_string_lossy().contains("config.toml"));
}

#[tokio::test]
async fn test_save_and_load_config() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_path_buf();

    let config = Config::default();
    config.save_to(&path).await.unwrap();

    let loaded = Config::load_from(&path).await.unwrap();
    assert_eq!(loaded.server.port, config.server.port);
    assert_eq!(loaded.ai.provider, config.ai.provider);
}

#[tokio::test]
async fn test_load_missing_config_returns_default() {
    let path = std::path::PathBuf::from("/nonexistent/config.toml");
    let config = Config::load_from(&path).await.unwrap();
    assert_eq!(config.server.port, 3000);
}
```

Add `tempfile` to dev-dependencies in `crates/engai-core/Cargo.toml`:
```toml
[dev-dependencies]
tempfile = "3"
tokio = { workspace = true, features = ["full", "macros"] }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p engai-core test_config`
Expected: FAIL — no `config` module with these types

- [ ] **Step 3: Implement Config**

`crates/engai-core/src/config.rs`:
```rust
use anyhow::{Context, Result};
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                port: 3000,
                host: "127.0.0.1".to_string(),
            },
            ai: AiConfig {
                provider: "kimi".to_string(),
                api_key: String::new(),
                model: String::new(),
                base_url: String::new(),
            },
            learning: LearningConfig {
                daily_new_words: 20,
                daily_review_limit: 100,
                default_deck: "01_vocab".to_string(),
            },
            storage: StorageConfig {
                db_path: "~/.engai/engai.db".to_string(),
                docs_path: "./docs".to_string(),
            },
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".engai")
    }

    pub fn config_file_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn db_path() -> PathBuf {
        let config = Self::load_global().unwrap_or_default();
        let raw = &config.storage.db_path;
        let path = PathBuf::from(raw);
        if raw.starts_with("~/") {
            dirs::home_dir()
                .unwrap_or_default()
                .join(&raw[2..])
        } else {
            path
        }
    }

    pub fn docs_path() -> PathBuf {
        let config = Self::load_global().unwrap_or_default();
        PathBuf::from(&config.storage.docs_path)
    }

    pub async fn load_global() -> Result<Self> {
        let path = Self::config_file_path();
        Self::load_from(&path).await
    }

    pub async fn load_from(path: &std::path::Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read config file")?;
        let config: Config =
            toml::from_str(&content).context("Failed to parse config file")?;
        Ok(config)
    }

    pub async fn save_to(&self, path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create config directory")?;
        }
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        tokio::fs::write(path, content)
            .await
            .context("Failed to write config file")?;
        Ok(())
    }

    pub fn resolve_api_key(&self) -> String {
        if !self.ai.api_key.is_empty() {
            return self.ai.api_key.clone();
        }
        std::env::var("ENGAI_AI_API_KEY").unwrap_or_default()
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p engai-core test_config`
Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add configuration system with default values and file persistence"
```

---

### Task 3: SQLite Database Setup

**Files:**
- Create: `crates/engai-core/migrations/001_init.sql`
- Modify: `crates/engai-core/src/db.rs`
- Create: `crates/engai-core/tests/test_db.rs`

- [ ] **Step 1: Write migration SQL**

`crates/engai-core/migrations/001_init.sql`:
```sql
CREATE TABLE IF NOT EXISTS words (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    word        TEXT UNIQUE NOT NULL,
    phonetic    TEXT,
    meaning     TEXT,
    familiarity INTEGER DEFAULT 0,
    next_review DATETIME,
    interval    INTEGER DEFAULT 0,
    ease_factor REAL DEFAULT 2.5,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS phrases (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    phrase      TEXT UNIQUE NOT NULL,
    meaning     TEXT,
    familiarity INTEGER DEFAULT 0,
    next_review DATETIME,
    interval    INTEGER DEFAULT 0,
    ease_factor REAL DEFAULT 2.5,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS examples (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    target_type TEXT NOT NULL CHECK(target_type IN ('word', 'phrase')),
    target_id   INTEGER NOT NULL,
    sentence    TEXT NOT NULL,
    source      TEXT
);

CREATE TABLE IF NOT EXISTS reviews (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    target_type TEXT NOT NULL CHECK(target_type IN ('word', 'phrase')),
    target_id   INTEGER NOT NULL,
    quality     INTEGER NOT NULL CHECK(quality >= 0 AND quality <= 5),
    reviewed_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS readings (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT,
    content    TEXT NOT NULL,
    source     TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS notes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    target_type TEXT NOT NULL CHECK(target_type IN ('word', 'phrase', 'reading')),
    target_id   INTEGER NOT NULL,
    content     TEXT NOT NULL,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS chat_history (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    role       TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
    content    TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_words_next_review ON words(next_review);
CREATE INDEX IF NOT EXISTS idx_phrases_next_review ON phrases(next_review);
CREATE INDEX IF NOT EXISTS idx_examples_target ON examples(target_type, target_id);
CREATE INDEX IF NOT EXISTS idx_reviews_target ON reviews(target_type, target_id);
CREATE INDEX IF NOT EXISTS idx_notes_target ON notes(target_type, target_id);
```

- [ ] **Step 2: Write failing test**

`crates/engai-core/tests/test_db.rs`:
```rust
use engai_core::db::Db;

#[tokio::test]
async fn test_db_init_and_word_crud() {
    let db = Db::new_in_memory().await.unwrap();

    db.add_word("abandon", None, None).await.unwrap();
    let word = db.get_word("abandon").await.unwrap();
    assert_eq!(word.word, "abandon");
    assert_eq!(word.familiarity, 0);

    let words = db.list_words(None, None, 10, 0).await.unwrap();
    assert_eq!(words.len(), 1);

    db.delete_word("abandon").await.unwrap();
    let result = db.get_word("abandon").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_phrase_crud() {
    let db = Db::new_in_memory().await.unwrap();

    db.add_phrase("take off", None).await.unwrap();
    let phrase = db.get_phrase("take off").await.unwrap();
    assert_eq!(phrase.phrase, "take off");

    let phrases = db.list_phrases(None, None, 10, 0).await.unwrap();
    assert_eq!(phrases.len(), 1);
}

#[tokio::test]
async fn test_examples_crud() {
    let db = Db::new_in_memory().await.unwrap();
    let word_id = db.add_word("test", None, None).await.unwrap();

    db.add_example("word", word_id, "a test sentence", Some("cli"))
        .await
        .unwrap();
    let examples = db.get_examples("word", word_id).await.unwrap();
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0].sentence, "a test sentence");
}

#[tokio::test]
async fn test_reading_crud() {
    let db = Db::new_in_memory().await.unwrap();

    let id = db.add_reading("Test Title", "Some content", Some("test"))
        .await
        .unwrap();
    let reading = db.get_reading(id).await.unwrap();
    assert_eq!(reading.title.as_deref(), Some("Test Title"));
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p engai-core test_db`
Expected: FAIL — no `db` module

- [ ] **Step 4: Implement Db**

`crates/engai-core/src/db.rs`:
```rust
use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::migrate::Migrator;
use std::str::FromStr;

use crate::models::{Example, Note, Phrase, Reading, Review, Word};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn new(db_path: &std::path::Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let options = SqliteConnectOptions::from_str(&db_url)?
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .context("Failed to connect to SQLite")?;
        MIGRATOR.run(&pool).await.context("Failed to run migrations")?;
        Ok(Self { pool })
    }

    pub async fn new_in_memory() -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        MIGRATOR.run(&pool).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    // --- Words ---

    pub async fn add_word(
        &self,
        word: &str,
        phonetic: Option<&str>,
        meaning: Option<&str>,
    ) -> Result<i64> {
        let result = sqlx::query_scalar::<_, i64>(
            "INSERT INTO words (word, phonetic, meaning) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(word)
        .bind(phonetic)
        .bind(meaning)
        .fetch_one(&self.pool)
        .await
        .context("Failed to add word")?;
        Ok(result)
    }

    pub async fn get_word(&self, word: &str) -> Result<Option<Word>> {
        let result = sqlx::query_as::<_, Word>("SELECT * FROM words WHERE word = ?")
            .bind(word)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }

    pub async fn get_word_by_id(&self, id: i64) -> Result<Option<Word>> {
        let result = sqlx::query_as::<_, Word>("SELECT * FROM words WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }

    pub async fn list_words(
        &self,
        search: Option<&str>,
        familiarity_gte: Option<i32>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Word>> {
        let mut query = String::from("SELECT * FROM words WHERE 1=1");
        if let Some(s) = search {
            query.push_str(&format!(" AND word LIKE '%{}%'", s));
        }
        if let Some(f) = familiarity_gte {
            query.push_str(&format!(" AND familiarity >= {}", f));
        }
        query.push_str(&format!(" ORDER BY updated_at DESC LIMIT {} OFFSET {}", limit, offset));
        let result = sqlx::query_as::<_, Word>(&query).fetch_all(&self.pool).await?;
        Ok(result)
    }

    pub async fn update_word(
        &self,
        word: &str,
        phonetic: Option<&str>,
        meaning: Option<&str>,
        familiarity: Option<i32>,
        next_review: Option<NaiveDateTime>,
        interval: Option<i32>,
        ease_factor: Option<f64>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE words SET phonetic = COALESCE(?, phonetic), meaning = COALESCE(?, meaning), familiarity = COALESCE(?, familiarity), next_review = COALESCE(?, next_review), interval = COALESCE(?, interval), ease_factor = COALESCE(?, ease_factor), updated_at = CURRENT_TIMESTAMP WHERE word = ?",
        )
        .bind(phonetic)
        .bind(meaning)
        .bind(familiarity)
        .bind(next_review)
        .bind(interval)
        .bind(ease_factor)
        .bind(word)
        .execute(&self.pool)
        .await
        .context("Failed to update word")?;
        Ok(())
    }

    pub async fn delete_word(&self, word: &str) -> Result<()> {
        sqlx::query("DELETE FROM words WHERE word = ?")
            .bind(word)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Phrases ---

    pub async fn add_phrase(&self, phrase: &str, meaning: Option<&str>) -> Result<i64> {
        let result = sqlx::query_scalar::<_, i64>(
            "INSERT INTO phrases (phrase, meaning) VALUES (?, ?) RETURNING id",
        )
        .bind(phrase)
        .bind(meaning)
        .fetch_one(&self.pool)
        .await
        .context("Failed to add phrase")?;
        Ok(result)
    }

    pub async fn get_phrase(&self, phrase: &str) -> Result<Option<Phrase>> {
        let result = sqlx::query_as::<_, Phrase>("SELECT * FROM phrases WHERE phrase = ?")
            .bind(phrase)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }

    pub async fn get_phrase_by_id(&self, id: i64) -> Result<Option<Phrase>> {
        let result = sqlx::query_as::<_, Phrase>("SELECT * FROM phrases WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }

    pub async fn list_phrases(
        &self,
        search: Option<&str>,
        familiarity_gte: Option<i32>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Phrase>> {
        let mut query = String::from("SELECT * FROM phrases WHERE 1=1");
        if let Some(s) = search {
            query.push_str(&format!(" AND phrase LIKE '%{}%'", s));
        }
        if let Some(f) = familiarity_gte {
            query.push_str(&format!(" AND familiarity >= {}", f));
        }
        query.push_str(&format!(" ORDER BY updated_at DESC LIMIT {} OFFSET {}", limit, offset));
        let result = sqlx::query_as::<_, Phrase>(&query).fetch_all(&self.pool).await?;
        Ok(result)
    }

    pub async fn update_phrase(
        &self,
        phrase: &str,
        meaning: Option<&str>,
        familiarity: Option<i32>,
        next_review: Option<NaiveDateTime>,
        interval: Option<i32>,
        ease_factor: Option<f64>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE phrases SET meaning = COALESCE(?, meaning), familiarity = COALESCE(?, familiarity), next_review = COALESCE(?, next_review), interval = COALESCE(?, interval), ease_factor = COALESCE(?, ease_factor), updated_at = CURRENT_TIMESTAMP WHERE phrase = ?",
        )
        .bind(meaning)
        .bind(familiarity)
        .bind(next_review)
        .bind(interval)
        .bind(ease_factor)
        .bind(phrase)
        .execute(&self.pool)
        .await
        .context("Failed to update phrase")?;
        Ok(())
    }

    pub async fn delete_phrase(&self, phrase: &str) -> Result<()> {
        sqlx::query("DELETE FROM phrases WHERE phrase = ?")
            .bind(phrase)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Examples ---

    pub async fn add_example(
        &self,
        target_type: &str,
        target_id: i64,
        sentence: &str,
        source: Option<&str>,
    ) -> Result<i64> {
        let result = sqlx::query_scalar::<_, i64>(
            "INSERT INTO examples (target_type, target_id, sentence, source) VALUES (?, ?, ?, ?) RETURNING id",
        )
        .bind(target_type)
        .bind(target_id)
        .bind(sentence)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn get_examples(&self, target_type: &str, target_id: i64) -> Result<Vec<Example>> {
        let result = sqlx::query_as::<_, Example>(
            "SELECT * FROM examples WHERE target_type = ? AND target_id = ? ORDER BY id",
        )
        .bind(target_type)
        .bind(target_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn delete_examples(&self, target_type: &str, target_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM examples WHERE target_type = ? AND target_id = ?")
            .bind(target_type)
            .bind(target_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Reviews ---

    pub async fn add_review(
        &self,
        target_type: &str,
        target_id: i64,
        quality: i32,
    ) -> Result<i64> {
        let result = sqlx::query_scalar::<_, i64>(
            "INSERT INTO reviews (target_type, target_id, quality) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(target_type)
        .bind(target_id)
        .bind(quality)
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn get_reviews(
        &self,
        target_type: &str,
        target_id: i64,
    ) -> Result<Vec<Review>> {
        let result = sqlx::query_as::<_, Review>(
            "SELECT * FROM reviews WHERE target_type = ? AND target_id = ? ORDER BY reviewed_at",
        )
        .bind(target_type)
        .bind(target_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    // --- Readings ---

    pub async fn add_reading(
        &self,
        title: Option<&str>,
        content: &str,
        source: Option<&str>,
    ) -> Result<i64> {
        let result = sqlx::query_scalar::<_, i64>(
            "INSERT INTO readings (title, content, source) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(title)
        .bind(content)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn get_reading(&self, id: i64) -> Result<Option<Reading>> {
        let result = sqlx::query_as::<_, Reading>("SELECT * FROM readings WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }

    pub async fn list_readings(&self, limit: i64, offset: i64) -> Result<Vec<Reading>> {
        let result = sqlx::query_as::<_, Reading>(
            "SELECT * FROM readings ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn delete_reading(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM readings WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Notes ---

    pub async fn add_note(
        &self,
        target_type: &str,
        target_id: i64,
        content: &str,
    ) -> Result<i64> {
        let result = sqlx::query_scalar::<_, i64>(
            "INSERT INTO notes (target_type, target_id, content) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(target_type)
        .bind(target_id)
        .bind(content)
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn get_notes(&self, target_type: &str, target_id: i64) -> Result<Vec<Note>> {
        let result = sqlx::query_as::<_, Note>(
            "SELECT * FROM notes WHERE target_type = ? AND target_id = ? ORDER BY created_at",
        )
        .bind(target_type)
        .bind(target_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn delete_note(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM notes WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Review queue ---

    pub async fn get_today_review_words(&self) -> Result<Vec<Word>> {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let result = sqlx::query_as::<_, Word>(
            "SELECT * FROM words WHERE next_review IS NOT NULL AND next_review <= ? ORDER BY next_review",
        )
        .bind(&now)
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn get_today_review_phrases(&self) -> Result<Vec<Phrase>> {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let result = sqlx::query_as::<_, Phrase>(
            "SELECT * FROM phrases WHERE next_review IS NOT NULL AND next_review <= ? ORDER BY next_review",
        )
        .bind(&now)
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    // --- Stats ---

    pub async fn word_count(&self) -> Result<i64> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM words").fetch_one(&self.pool).await?;
        Ok(result.0)
    }

    pub async fn phrase_count(&self) -> Result<i64> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM phrases").fetch_one(&self.pool).await?;
        Ok(result.0)
    }

    pub async fn review_count_today(&self) -> Result<i64> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM reviews WHERE reviewed_at >= ?",
        )
        .bind(format!("{} 00:00:00", today))
        .fetch_one(&self.pool)
        .await?;
        Ok(result.0)
    }

    pub async fn pending_review_count(&self) -> Result<i64> {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let words: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM words WHERE next_review IS NOT NULL AND next_review <= ?",
        )
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;
        let phrases: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM phrases WHERE next_review IS NOT NULL AND next_review <= ?",
        )
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;
        Ok(words.0 + phrases.0)
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p engai-core test_db`
Expected: All PASS

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add SQLite database with full schema, migrations, and CRUD operations"
```

---

### Task 4: Markdown Parsing and Generation

**Files:**
- Modify: `crates/engai-core/src/markdown.rs`
- Create: `crates/engai-core/tests/test_markdown.rs`

- [ ] **Step 1: Write failing tests**

`crates/engai-core/tests/test_markdown.rs`:
```rust
use engai_core::markdown::{MarkdownWord, MarkdownPhrase, MarkdownReading};
use engai_core::models;
use std::io::Write;
use tempfile::NamedTempFile;

fn word_md_content() -> &'static str {
    r#"---
word: abandon
familiarity: 3
interval: 7
next_review: 2026-04-05 00:00:00
synced_at: 2026-03-29T10:00:00
---

# abandon

## Meaning
to leave something behind

## Example
I abandoned the project.

## Synonyms
- leave
- quit

## AI Explanation
> AI generated explanation here.

## My Notes
- personal note

## Review
- 2026-03-29 ⭐
"#
}

#[test]
fn test_parse_word_markdown() {
    let parsed = MarkdownWord::parse(word_md_content()).unwrap();
    assert_eq!(parsed.word, "abandon");
    assert_eq!(parsed.familiarity, 3);
    assert_eq!(parsed.interval, 7);
    assert_eq!(parsed.meaning, "to leave something behind");
    assert_eq!(parsed.examples.len(), 1);
    assert_eq!(parsed.synonyms.len(), 2);
    assert_eq!(parsed.my_notes.len(), 1);
}

#[test]
fn test_generate_word_markdown() {
    let md = MarkdownWord {
        word: "test".to_string(),
        phonetic: None,
        familiarity: 0,
        interval: 0,
        next_review: None,
        meaning: Some("a test meaning".to_string()),
        examples: vec!["example sentence".to_string()],
        synonyms: vec![],
        ai_explanation: None,
        my_notes: vec![],
        reviews: vec![],
    };
    let output = md.to_markdown_string();
    assert!(output.contains("# test"));
    assert!(output.contains("a test meaning"));
    assert!(output.contains("## Example"));
}

fn phrase_md_content() -> &'static str {
    r#"---
phrase: take off
familiarity: 2
interval: 3
next_review: 2026-04-01 00:00:00
synced_at: 2026-03-29T10:00:00
---

# take off

## Meaning
to remove something

## Examples
- She took off her coat.

## AI Explanation
> AI explanation.

## My Notes
- two meanings

## Review
- 2026-03-29 ⭐
"#
}

#[test]
fn test_parse_phrase_markdown() {
    let parsed = MarkdownPhrase::parse(phrase_md_content()).unwrap();
    assert_eq!(parsed.phrase, "take off");
    assert_eq!(parsed.meaning, "to remove something");
}

fn reading_md_content() -> &'static str {
    r#"---
title: "The Art of Learning"
source: https://example.com
imported_at: 2026-03-29T10:00:00
synced_at: 2026-03-29T10:00:00
---

# The Art of Learning

## Content
The most important skill is learning how to learn.

## Vocabulary
- abandon: to leave behind

## Summary (AI)
> A summary here.

## My Notes
- good article
"#
}

#[test]
fn test_parse_reading_markdown() {
    let parsed = MarkdownReading::parse(reading_md_content()).unwrap();
    assert_eq!(parsed.title, "The Art of Learning");
    assert!(parsed.content.contains("learning how to learn"));
}

#[tokio::test]
async fn test_roundtrip_word_file() {
    let mut tmp = NamedTempFile::new().unwrap();
    write!(tmp, "{}", word_md_content()).unwrap();

    let parsed = MarkdownWord::parse_file(tmp.path()).await.unwrap();
    assert_eq!(parsed.word, "abandon");

    let output = parsed.to_markdown_string();
    let mut tmp2 = NamedTempFile::new().unwrap();
    write!(tmp2, "{}", output).unwrap();

    let parsed2 = MarkdownWord::parse_file(tmp2.path()).await.unwrap();
    assert_eq!(parsed2.word, parsed.word);
    assert_eq!(parsed2.familiarity, parsed.familiarity);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p engai-core test_markdown`
Expected: FAIL

- [ ] **Step 3: Implement markdown module**

`crates/engai-core/src/markdown.rs`:
```rust
use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownWord {
    pub word: String,
    pub phonetic: Option<String>,
    pub familiarity: i32,
    pub interval: i32,
    pub next_review: Option<NaiveDateTime>,
    pub meaning: Option<String>,
    pub examples: Vec<String>,
    pub synonyms: Vec<String>,
    pub ai_explanation: Option<String>,
    pub my_notes: Vec<String>,
    pub reviews: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordFrontmatter {
    pub word: String,
    pub phonetic: Option<String>,
    pub familiarity: i32,
    pub interval: i32,
    pub next_review: Option<String>,
    pub synced_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownPhrase {
    pub phrase: String,
    pub familiarity: i32,
    pub interval: i32,
    pub next_review: Option<NaiveDateTime>,
    pub meaning: Option<String>,
    pub examples: Vec<String>,
    pub ai_explanation: Option<String>,
    pub my_notes: Vec<String>,
    pub reviews: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhraseFrontmatter {
    pub phrase: String,
    pub familiarity: i32,
    pub interval: i32,
    pub next_review: Option<String>,
    pub synced_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownReading {
    pub title: String,
    pub source: Option<String>,
    pub content: String,
    pub vocabulary: Vec<String>,
    pub summary: Option<String>,
    pub my_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingFrontmatter {
    pub title: Option<String>,
    pub source: Option<String>,
    pub imported_at: Option<String>,
    pub synced_at: Option<String>,
}

fn parse_frontmatter<T: serde::de::DeserializeOwned>(
    content: &str,
) -> Result<(Option<T>, String)> {
    let matter = gray_matter::Matter::<gray_matter::engine::YAML>::new();
    let result = matter.parse_with_struct::<T>(content);
    let (fm, body) = (result.data, result.content);
    Ok((fm, body))
}

fn parse_naive_datetime(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
        .ok()
}

fn extract_section(body: &str, section_name: &str) -> Option<String> {
    let header = format!("## {}", section_name);
    if let Some(start) = body.find(&header) {
        let after = &body[start + header.len()..];
        let end = after.find("\n## ").map(|i| i).unwrap_or(after.len());
        let content = after[..end].trim().to_string();
        if content.is_empty() {
            None
        } else {
            Some(content)
        }
    } else {
        None
    }
}

fn extract_list(body: &str, section_name: &str) -> Vec<String> {
    extract_section(body, section_name)
        .map(|s| {
            s.lines()
                .filter(|l| l.starts_with("- ") || l.starts_with("* "))
                .map(|l| l[2..].trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

impl MarkdownWord {
    pub fn parse(content: &str) -> Result<Self> {
        let (fm, body) = parse_frontmatter::<WordFrontmatter>(content)?;
        let fm = fm.context("Missing frontmatter")?;

        Ok(Self {
            word: fm.word,
            phonetic: fm.phonetic,
            familiarity: fm.familiarity,
            interval: fm.interval,
            next_review: fm.next_review.as_deref().and_then(parse_naive_datetime),
            meaning: extract_section(&body, "Meaning"),
            examples: {
                let mut examples = extract_list(&body, "Example");
                if examples.is_empty() {
                    examples = extract_list(&body, "Examples");
                }
                if examples.is_empty() {
                    if let Some(s) = extract_section(&body, "Example") {
                        examples.push(s);
                    }
                }
                examples
            },
            synonyms: extract_list(&body, "Synonyms"),
            ai_explanation: extract_section(&body, "AI Explanation"),
            my_notes: extract_list(&body, "My Notes"),
            reviews: extract_list(&body, "Review"),
        })
    }

    pub async fn parse_file(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::parse(&content)
    }

    pub fn to_markdown_string(&self) -> String {
        let mut out = String::new();

        out.push_str("---\n");
        out.push_str(&format!("word: {}\n", self.word));
        if let Some(p) = &self.phonetic {
            out.push_str(&format!("phonetic: {}\n", p));
        }
        out.push_str(&format!("familiarity: {}\n", self.familiarity));
        out.push_str(&format!("interval: {}\n", self.interval));
        if let Some(nr) = &self.next_review {
            out.push_str(&format!("next_review: {}\n", nr.format("%Y-%m-%d %H:%M:%S")));
        }
        out.push_str(&format!(
            "synced_at: {}\n",
            chrono::Local::now().format("%Y-%m-%dT%H:%M:%S")
        ));
        out.push_str("---\n\n");

        out.push_str(&format!("# {}\n\n", self.word));

        if let Some(m) = &self.meaning {
            out.push_str("## Meaning\n");
            out.push_str(m);
            out.push_str("\n\n");
        }

        out.push_str("## Example\n");
        for ex in &self.examples {
            out.push_str(&format!("- {}\n", ex));
        }
        if self.examples.is_empty() {
            out.push_str("\n");
        }
        out.push('\n');

        if !self.synonyms.is_empty() {
            out.push_str("## Synonyms\n");
            for s in &self.synonyms {
                out.push_str(&format!("- {}\n", s));
            }
            out.push('\n');
        }

        if let Some(ai) = &self.ai_explanation {
            out.push_str("## AI Explanation\n");
            out.push_str("> ");
            out.push_str(ai);
            out.push_str("\n\n");
        }

        if !self.my_notes.is_empty() {
            out.push_str("## My Notes\n");
            for n in &self.my_notes {
                out.push_str(&format!("- {}\n", n));
            }
            out.push('\n');
        }

        if !self.reviews.is_empty() {
            out.push_str("## Review\n");
            for r in &self.reviews {
                out.push_str(&format!("- {}\n", r));
            }
            out.push('\n');
        }

        out
    }

    pub async fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, self.to_markdown_string()).await?;
        Ok(())
    }
}

impl MarkdownPhrase {
    pub fn parse(content: &str) -> Result<Self> {
        let (fm, body) = parse_frontmatter::<PhraseFrontmatter>(content)?;
        let fm = fm.context("Missing frontmatter")?;

        Ok(Self {
            phrase: fm.phrase,
            familiarity: fm.familiarity,
            interval: fm.interval,
            next_review: fm.next_review.as_deref().and_then(parse_naive_datetime),
            meaning: extract_section(&body, "Meaning"),
            examples: extract_list(&body, "Examples"),
            ai_explanation: extract_section(&body, "AI Explanation"),
            my_notes: extract_list(&body, "My Notes"),
            reviews: extract_list(&body, "Review"),
        })
    }

    pub async fn parse_file(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::parse(&content)
    }

    pub fn to_markdown_string(&self) -> String {
        let mut out = String::new();

        out.push_str("---\n");
        out.push_str(&format!("phrase: {}\n", self.phrase));
        out.push_str(&format!("familiarity: {}\n", self.familiarity));
        out.push_str(&format!("interval: {}\n", self.interval));
        if let Some(nr) = &self.next_review {
            out.push_str(&format!("next_review: {}\n", nr.format("%Y-%m-%d %H:%M:%S")));
        }
        out.push_str(&format!(
            "synced_at: {}\n",
            chrono::Local::now().format("%Y-%m-%dT%H:%M:%S")
        ));
        out.push_str("---\n\n");

        out.push_str(&format!("# {}\n\n", self.phrase));

        if let Some(m) = &self.meaning {
            out.push_str("## Meaning\n");
            out.push_str(m);
            out.push_str("\n\n");
        }

        out.push_str("## Examples\n");
        for ex in &self.examples {
            out.push_str(&format!("- {}\n", ex));
        }
        if self.examples.is_empty() {
            out.push_str("\n");
        }
        out.push('\n');

        if let Some(ai) = &self.ai_explanation {
            out.push_str("## AI Explanation\n");
            out.push_str("> ");
            out.push_str(ai);
            out.push_str("\n\n");
        }

        if !self.my_notes.is_empty() {
            out.push_str("## My Notes\n");
            for n in &self.my_notes {
                out.push_str(&format!("- {}\n", n));
            }
            out.push('\n');
        }

        if !self.reviews.is_empty() {
            out.push_str("## Review\n");
            for r in &self.reviews {
                out.push_str(&format!("- {}\n", r));
            }
            out.push('\n');
        }

        out
    }

    pub async fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, self.to_markdown_string()).await?;
        Ok(())
    }
}

impl MarkdownReading {
    pub fn parse(content: &str) -> Result<Self> {
        let (fm, body) = parse_frontmatter::<ReadingFrontmatter>(content)?;
        let fm = fm.context("Missing frontmatter")?;

        Ok(Self {
            title: fm.title.unwrap_or_default(),
            source: fm.source,
            content: extract_section(&body, "Content").unwrap_or_default(),
            vocabulary: extract_list(&body, "Vocabulary"),
            summary: extract_section(&body, "Summary (AI)"),
            my_notes: extract_list(&body, "My Notes"),
        })
    }

    pub async fn parse_file(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::parse(&content)
    }

    pub fn to_markdown_string(&self) -> String {
        let mut out = String::new();

        out.push_str("---\n");
        if !self.title.is_empty() {
            out.push_str(&format!("title: \"{}\"\n", self.title.replace('"', "\\\"")));
        }
        if let Some(s) = &self.source {
            out.push_str(&format!("source: {}\n", s));
        }
        out.push_str(&format!(
            "imported_at: {}\n",
            chrono::Local::now().format("%Y-%m-%dT%H:%M:%S")
        ));
        out.push_str(&format!(
            "synced_at: {}\n",
            chrono::Local::now().format("%Y-%m-%dT%H:%M:%S")
        ));
        out.push_str("---\n\n");

        out.push_str(&format!("# {}\n\n", self.title));

        out.push_str("## Content\n");
        out.push_str(&self.content);
        out.push_str("\n\n");

        if !self.vocabulary.is_empty() {
            out.push_str("## Vocabulary\n");
            for v in &self.vocabulary {
                out.push_str(&format!("- {}\n", v));
            }
            out.push('\n');
        }

        if let Some(s) = &self.summary {
            out.push_str("## Summary (AI)\n");
            out.push_str("> ");
            out.push_str(s);
            out.push_str("\n\n");
        }

        if !self.my_notes.is_empty() {
            out.push_str("## My Notes\n");
            for n in &self.my_notes {
                out.push_str(&format!("- {}\n", n));
            }
            out.push('\n');
        }

        out
    }

    pub async fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, self.to_markdown_string()).await?;
        Ok(())
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p engai-core test_markdown`
Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add Markdown parsing and generation for words, phrases, and readings"
```

---

### Task 5: Prompt Template System

**Files:**
- Modify: `crates/engai-core/src/prompt.rs`
- Create: `crates/engai-core/tests/test_prompt.rs`

- [ ] **Step 1: Write failing test**

`crates/engai-core/tests/test_prompt.rs`:
```rust
use engai_core::prompt::PromptEngine;
use std::io::Write;
use tempfile::TempDir;

#[tokio::test]
async fn test_load_and_render_template() {
    let tmp = TempDir::new().unwrap();
    let template_path = tmp.path().join("test.md");
    let mut f = std::fs::File::create(&template_path).unwrap();
    write!(f, "Explain the word: {{{{word}}}}\nLevel: {{{{level}}}}").unwrap();

    let engine = PromptEngine::new(tmp.path().to_path_buf());
    let rendered = engine.render("test.md", &[("word", "abandon"), ("level", "B2")]).await.unwrap();
    assert!(rendered.contains("abandon"));
    assert!(rendered.contains("B2"));
    assert!(!rendered.contains("{{"));
}

#[tokio::test]
async fn test_template_not_found() {
    let tmp = TempDir::new().unwrap();
    let engine = PromptEngine::new(tmp.path().to_path_buf());
    let result = engine.render("nonexistent.md", &[]).await;
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p engai-core test_prompt`
Expected: FAIL

- [ ] **Step 3: Implement prompt module**

`crates/engai-core/src/prompt.rs`:
```rust
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

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
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p engai-core test_prompt`
Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add prompt template engine with variable interpolation"
```

---

### Task 6: SM-2 Spaced Repetition Algorithm

**Files:**
- Modify: `crates/engai-core/src/review.rs`
- Create: `crates/engai-core/tests/test_review.rs`

- [ ] **Step 1: Write failing test**

`crates/engai-core/tests/test_review.rs`:
```rust
use engai_core::review::{ReviewResult, calculate_next_review};

#[test]
fn test_quality_0_resets() {
    let result = calculate_next_review(0, 1, 2.5);
    assert_eq!(result.interval, 1);
    assert_eq!(result.familiarity, 0);
    assert!(result.ease_factor < 2.5);
}

#[test]
fn test_quality_2_resets() {
    let result = calculate_next_review(2, 7, 2.5);
    assert_eq!(result.interval, 1);
    assert_eq!(result.familiarity, 0);
}

#[test]
fn test_quality_3_no_change() {
    let result = calculate_next_review(3, 7, 2.5);
    assert_eq!(result.interval, 7);
    assert_eq!(result.familiarity, 1);
}

#[test]
fn test_quality_4_increases() {
    let result = calculate_next_review(4, 1, 2.5);
    assert!(result.interval > 1);
    assert!(result.familiarity > 0);
}

#[test]
fn test_quality_5_increases_more() {
    let result = calculate_next_review(5, 1, 2.5);
    let result_q4 = calculate_next_review(4, 1, 2.5);
    assert!(result.interval >= result_q4.interval);
    assert!(result.ease_factor > result_q4.ease_factor);
}

#[test]
fn test_ease_factor_floor() {
    let mut ef = 2.5;
    for _ in 0..20 {
        ef = ef - 0.2 + (0.1 * 0.0);
    }
    let result = calculate_next_review(0, 7, ef);
    assert!(result.ease_factor >= 1.3);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p engai-core test_review`
Expected: FAIL

- [ ] **Step 3: Implement review module**

`crates/engai-core/src/review.rs`:
```rust
use chrono::{Duration, Local};

pub struct ReviewResult {
    pub familiarity: i32,
    pub interval: i32,
    pub ease_factor: f64,
    pub next_review: chrono::NaiveDateTime,
}

pub fn calculate_next_review(quality: i32, current_interval: i32, current_ease_factor: f64) -> ReviewResult {
    let q = quality.clamp(0, 5);

    if q <= 2 {
        return ReviewResult {
            familiarity: 0,
            interval: 1,
            ease_factor: (current_ease_factor - 0.2).max(1.3),
            next_review: Local::now().naive_local() + Duration::days(1),
        };
    }

    if q == 3 {
        return ReviewResult {
            familiarity: 1,
            interval: current_interval,
            ease_factor: current_ease_factor,
            next_review: Local::now().naive_local() + Duration::days(current_interval as i64),
        };
    }

    let ease_adjust = match q {
        4 => 0.15,
        _ => 0.20,
    };
    let new_ef = (current_ease_factor + ease_adjust).min(3.0);
    let new_interval = if current_interval == 0 {
        1
    } else {
        (current_interval as f64 * new_ef).round() as i32
    };

    ReviewResult {
        familiarity: (q - 2).min(5),
        interval: new_interval,
        ease_factor: new_ef,
        next_review: Local::now().naive_local() + Duration::days(new_interval as i64),
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p engai-core test_review`
Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add SM-2 spaced repetition algorithm"
```

---

### Task 7: AI Integration

**Files:**
- Modify: `crates/engai-core/src/ai.rs`
- Create: `crates/engai-core/tests/test_ai.rs` (integration test, requires API key to run)

- [ ] **Step 1: Write the AI client module**

`crates/engai-core/src/ai.rs`:
```rust
use anyhow::{Context, Result};
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::Config;
use crate::prompt::PromptEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiClient {
    client: Client,
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

impl AiClient {
    pub fn from_config(config: &Config) -> Result<Self> {
        let api_key = config.resolve_api_key();
        if api_key.is_empty() {
            anyhow::bail!("AI API key not configured. Set it via config or ENGAI_AI_API_KEY env var.");
        }

        let (base_url, model) = match config.ai.provider.as_str() {
            "kimi" => (
                config
                    .ai
                    .base_url
                    .clone()
                    .if_empty("https://api.moonshot.cn/v1".to_string()),
                config.ai.model.clone().if_empty("moonshot-v1-8k".to_string()),
            ),
            "openai" => (
                config
                    .ai
                    .base_url
                    .clone()
                    .if_empty("https://api.openai.com/v1".to_string()),
                config.ai.model.clone().if_empty("gpt-4o-mini".to_string()),
            ),
            _ => (
                config.ai.base_url.clone(),
                config.ai.model.clone(),
            ),
        };

        Ok(Self {
            client: Client::new(),
            provider: config.ai.provider.clone(),
            api_key,
            model,
            base_url,
        })
    }

    pub async fn explain_word(
        &self,
        word: &str,
        prompt_engine: &PromptEngine,
    ) -> Result<String> {
        let system_prompt = prompt_engine
            .render("explain_word.md", &[])
            .await
            .unwrap_or_else(|_| "Explain the given English word clearly.".to_string());

        let user_prompt = prompt_engine
            .render("explain_word.md", &[("word", word)])
            .await
            .unwrap_or_else(|_| format!("Explain the word: {}", word));

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ];

        self.chat_completion(&messages).await
    }

    pub async fn explain_phrase(
        &self,
        phrase: &str,
        prompt_engine: &PromptEngine,
    ) -> Result<String> {
        let system_prompt = prompt_engine
            .render("explain_phrase.md", &[])
            .await
            .unwrap_or_else(|_| "Explain the given English phrase clearly.".to_string());

        let user_prompt = prompt_engine
            .render("explain_phrase.md", &[("phrase", phrase)])
            .await
            .unwrap_or_else(|_| format!("Explain the phrase: {}", phrase));

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ];

        self.chat_completion(&messages).await
    }

    pub async fn analyze_reading(
        &self,
        content: &str,
        prompt_engine: &PromptEngine,
    ) -> Result<String> {
        let user_prompt = prompt_engine
            .render("reading_analyze.md", &[("content", content)])
            .await
            .unwrap_or_else(|_| format!("Analyze this text:\n{}", content));

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: user_prompt,
        }];

        self.chat_completion(&messages).await
    }

    pub async fn chat_completion(&self, messages: &[ChatMessage]) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);

        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "stream": false,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send request to AI API")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("AI API error {}: {}", status, text);
        }

        let resp_json: serde_json::Value = response.json().await?;
        let content = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    pub async fn chat_completion_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<impl futures::Stream<Item = Result<String>>> {
        let url = format!("{}/chat/completions", self.base_url);

        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "stream": true,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send streaming request to AI API")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("AI API stream error {}: {}", status, text);
        }

        let stream = response.bytes_stream().map(move |chunk| {
            let text = String::from_utf8_lossy(&chunk?);
            let mut result = String::new();
            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data.trim() == "[DONE]" {
                        break;
                    }
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(delta) = json["choices"][0]["delta"]["content"].as_str() {
                            result.push_str(delta);
                        }
                    }
                }
            }
            if result.is_empty() {
                Err(anyhow::anyhow!("empty chunk"))
            } else {
                Ok(result)
            }
        });

        Ok(stream)
    }
}

trait IfEmpty {
    fn if_empty(self, default: Self) -> Self;
}

impl IfEmpty for String {
    fn if_empty(self, default: Self) -> Self {
        if self.is_empty() { default } else { self }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p engai-core`
Expected: Compiles

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: add AI integration client with streaming support for Kimi/OpenAI"
```

---

### Task 8: Bidirectional Sync Engine

**Files:**
- Modify: `crates/engai-core/src/sync.rs`
- Create: `crates/engai-core/tests/test_sync.rs`

- [ ] **Step 1: Write failing test**

`crates/engai-core/tests/test_sync.rs`:
```rust
use engai_core::db::Db;
use engai_core::markdown::MarkdownWord;
use engai_core::sync::SyncEngine;
use tempfile::TempDir;

#[tokio::test]
async fn test_sync_word_from_db_to_markdown() {
    let tmp = TempDir::new().unwrap();
    let docs_path = tmp.path().join("docs");

    let db = std::sync::Arc::new(Db::new_in_memory().await.unwrap());
    let engine = SyncEngine::new(db.clone(), &docs_path, &tmp.path().join("prompts"));

    db.add_word("abandon", None, Some("to leave behind"))
        .await
        .unwrap();

    engine.sync_all().await.unwrap();

    let md_path = docs_path.join("01_vocab").join("abandon.md");
    assert!(md_path.exists());

    let parsed = MarkdownWord::parse_file(&md_path).await.unwrap();
    assert_eq!(parsed.word, "abandon");
    assert_eq!(parsed.meaning.as_deref(), Some("to leave behind"));
}

#[tokio::test]
async fn test_sync_word_from_markdown_to_db() {
    let tmp = TempDir::new().unwrap();
    let docs_path = tmp.path().join("docs");

    tokio::fs::create_dir_all(docs_path.join("01_vocab")).await.unwrap();

    let content = r#"---
word: derive
familiarity: 0
interval: 0
synced_at: 2026-03-29T10:00:00
---

# derive

## Meaning
to obtain from a source

## Example
- We derive knowledge from experience.
"#
    .to_string();

    tokio::fs::write(docs_path.join("01_vocab").join("derive.md"), &content)
        .await
        .unwrap();

    let db = std::sync::Arc::new(Db::new_in_memory().await.unwrap());
    let engine = SyncEngine::new(db.clone(), &docs_path, &tmp.path().join("prompts"));

    engine.sync_all().await.unwrap();

    let word = db.get_word("derive").await.unwrap();
    assert!(word.is_some());
    let word = word.unwrap();
    assert_eq!(word.meaning.as_deref(), Some("to obtain from a source"));
}

#[tokio::test]
async fn test_sync_phrase() {
    let tmp = TempDir::new().unwrap();
    let docs_path = tmp.path().join("docs");

    tokio::fs::create_dir_all(docs_path.join("02_phrases")).await.unwrap();

    let content = r#"---
phrase: give up
familiarity: 0
interval: 0
synced_at: 2026-03-29T10:00:00
---

# give up

## Meaning
to stop trying
"#
    .to_string();

    tokio::fs::write(docs_path.join("02_phrases").join("give_up.md"), &content)
        .await
        .unwrap();

    let db = std::sync::Arc::new(Db::new_in_memory().await.unwrap());
    let engine = SyncEngine::new(db.clone(), &docs_path, &tmp.path().join("prompts"));

    engine.sync_all().await.unwrap();

    let phrase = db.get_phrase("give up").await.unwrap();
    assert!(phrase.is_some());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p engai-core test_sync`
Expected: FAIL

- [ ] **Step 3: Implement sync module**

`crates/engai-core/src/sync.rs`:
```rust
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::db::Db;
use crate::markdown::{MarkdownPhrase, MarkdownWord};

pub struct SyncEngine {
    db: std::sync::Arc<Db>,
    docs_path: PathBuf,
}

impl SyncEngine {
    pub fn new(db: std::sync::Arc<Db>, docs_path: &Path, _prompts_path: &Path) -> Self {
        Self {
            db,
            docs_path: docs_path.to_path_buf(),
        }
    }

    pub async fn sync_all(&self) -> Result<()> {
        self.sync_words().await?;
        self.sync_phrases().await?;
        Ok(())
    }

    async fn sync_words(&self) -> Result<()> {
        let vocab_dir = self.docs_path.join("01_vocab");
        if !vocab_dir.exists() {
            tokio::fs::create_dir_all(&vocab_dir).await?;
        }

        // DB → Markdown
        let words = self.db.list_words(None, None, 10000, 0).await?;
        for word in &words {
            let md_path = vocab_dir.join(format!("{}.md", word.word));
            let needs_write = if md_path.exists() {
                let file_time = tokio::fs::metadata(&md_path)
                    .await
                    .ok()
                    .and_then(|m| m.modified().ok());
                let db_time = chrono::DateTime::from_utc(word.updated_at, chrono::Local::now().offset().clone());
                let db_file_time = std::time::SystemTime::from(db_time);
                match (file_time, db_file_time) {
                    (Some(ft), Some(dt)) => dt > ft,
                    _ => true,
                }
            } else {
                true
            };

            if needs_write {
                let examples = self
                    .db
                    .get_examples("word", word.id)
                    .await
                    .unwrap_or_default();
                let notes = self
                    .db
                    .get_notes("word", word.id)
                    .await
                    .unwrap_or_default();
                let reviews = self
                    .db
                    .get_reviews("word", word.id)
                    .await
                    .unwrap_or_default();

                let md = MarkdownWord {
                    word: word.word.clone(),
                    phonetic: word.phonetic.clone(),
                    familiarity: word.familiarity,
                    interval: word.interval,
                    next_review: word.next_review,
                    meaning: word.meaning.clone(),
                    examples: examples.iter().map(|e| e.sentence.clone()).collect(),
                    synonyms: vec![],
                    ai_explanation: None,
                    my_notes: notes.iter().map(|n| n.content.clone()).collect(),
                    reviews: reviews
                        .iter()
                        .map(|r| {
                            format!(
                                "{} ⭐{}",
                                r.reviewed_at.format("%Y-%m-%d"),
                                "⭐".repeat(r.quality as usize)
                            )
                        })
                        .collect(),
                };
                md.save_to_file(&md_path).await?;
                info!(word = %word.word, "DB → Markdown");
            }
        }

        // Markdown → DB
        let mut entries = tokio::fs::read_dir(&vocab_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            if let Ok(parsed) = MarkdownWord::parse_file(&path).await {
                let existing = self.db.get_word(&parsed.word).await?;
                match existing {
                    None => {
                        self.db
                            .add_word(&parsed.word, parsed.phonetic.as_deref(), parsed.meaning.as_deref())
                            .await?;
                        self.db
                            .update_word(
                                &parsed.word,
                                None,
                                None,
                                Some(parsed.familiarity),
                                parsed.next_review,
                                Some(parsed.interval),
                                None,
                            )
                            .await?;
                        for ex in &parsed.examples {
                            if let Ok(word) = self.db.get_word(&parsed.word).await {
                                if let Some(w) = word {
                                    let _ = self.db.add_example("word", w.id, ex, Some("markdown")).await;
                                }
                            }
                        }
                        info!(word = %parsed.word, "Markdown → DB (new)");
                    }
                    Some(existing_word) => {
                        let file_time = tokio::fs::metadata(&path)
                            .await
                            .ok()
                            .and_then(|m| m.modified().ok());
                        let db_time = chrono::DateTime::from_utc(
                            existing_word.updated_at,
                            chrono::Local::now().offset().clone(),
                        );
                        let db_file_time = std::time::SystemTime::from(db_time);

                        if let (Some(ft), Some(dt)) = (file_time, db_file_time) {
                            if ft > dt {
                                self.db
                                    .update_word(
                                        &parsed.word,
                                        parsed.phonetic.as_deref(),
                                        parsed.meaning.as_deref(),
                                        Some(parsed.familiarity),
                                        parsed.next_review,
                                        Some(parsed.interval),
                                        None,
                                    )
                                    .await?;
                                info!(word = %parsed.word, "Markdown → DB (update)");
                            }
                        }
                    }
                }
            } else {
                warn!(path = %path.display(), "Failed to parse word Markdown file");
            }
        }

        Ok(())
    }

    async fn sync_phrases(&self) -> Result<()> {
        let phrases_dir = self.docs_path.join("02_phrases");
        if !phrases_dir.exists() {
            tokio::fs::create_dir_all(&phrases_dir).await?;
        }

        let phrases = self.db.list_phrases(None, None, 10000, 0).await?;
        for phrase in &phrases {
            let safe_name = phrase.phrase.replace(' ', "_");
            let md_path = phrases_dir.join(format!("{}.md", safe_name));

            let needs_write = if md_path.exists() {
                let file_time = tokio::fs::metadata(&md_path)
                    .await
                    .ok()
                    .and_then(|m| m.modified().ok());
                let db_time = chrono::DateTime::from_utc(
                    phrase.updated_at,
                    chrono::Local::now().offset().clone(),
                );
                let db_file_time = std::time::SystemTime::from(db_time);
                match (file_time, db_file_time) {
                    (Some(ft), Some(dt)) => dt > ft,
                    _ => true,
                }
            } else {
                true
            };

            if needs_write {
                let examples = self
                    .db
                    .get_examples("phrase", phrase.id)
                    .await
                    .unwrap_or_default();
                let notes = self
                    .db
                    .get_notes("phrase", phrase.id)
                    .await
                    .unwrap_or_default();
                let reviews = self
                    .db
                    .get_reviews("phrase", phrase.id)
                    .await
                    .unwrap_or_default();

                let md = MarkdownPhrase {
                    phrase: phrase.phrase.clone(),
                    familiarity: phrase.familiarity,
                    interval: phrase.interval,
                    next_review: phrase.next_review,
                    meaning: phrase.meaning.clone(),
                    examples: examples.iter().map(|e| e.sentence.clone()).collect(),
                    ai_explanation: None,
                    my_notes: notes.iter().map(|n| n.content.clone()).collect(),
                    reviews: reviews
                        .iter()
                        .map(|r| {
                            format!(
                                "{} ⭐{}",
                                r.reviewed_at.format("%Y-%m-%d"),
                                "⭐".repeat(r.quality as usize)
                            )
                        })
                        .collect(),
                };
                md.save_to_file(&md_path).await?;
                info!(phrase = %phrase.phrase, "DB → Markdown");
            }
        }

        let mut entries = tokio::fs::read_dir(&phrases_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            if let Ok(parsed) = MarkdownPhrase::parse_file(&path).await {
                let existing = self.db.get_phrase(&parsed.phrase).await?;
                match existing {
                    None => {
                        self.db
                            .add_phrase(&parsed.phrase, parsed.meaning.as_deref())
                            .await?;
                        self.db
                            .update_phrase(
                                &parsed.phrase,
                                None,
                                Some(parsed.familiarity),
                                parsed.next_review,
                                Some(parsed.interval),
                                None,
                            )
                            .await?;
                        info!(phrase = %parsed.phrase, "Markdown → DB (new)");
                    }
                    Some(existing_phrase) => {
                        let file_time = tokio::fs::metadata(&path)
                            .await
                            .ok()
                            .and_then(|m| m.modified().ok());
                        let db_time = chrono::DateTime::from_utc(
                            existing_phrase.updated_at,
                            chrono::Local::now().offset().clone(),
                        );
                        let db_file_time = std::time::SystemTime::from(db_time);

                        if let (Some(ft), Some(dt)) = (file_time, db_file_time) {
                            if ft > dt {
                                self.db
                                    .update_phrase(
                                        &parsed.phrase,
                                        parsed.meaning.as_deref(),
                                        Some(parsed.familiarity),
                                        parsed.next_review,
                                        Some(parsed.interval),
                                        None,
                                    )
                                    .await?;
                                info!(phrase = %parsed.phrase, "Markdown → DB (update)");
                            }
                        }
                    }
                }
            } else {
                warn!(path = %path.display(), "Failed to parse phrase Markdown file");
            }
        }

        Ok(())
    }
}
```

> **Note:** The `'static` lifetime on `db` in `SyncEngine` is for simplicity in Phase 1 CLI usage. Phase 2 will use `Arc<Db>` properly. If the compiler rejects `'static`, change to `Arc<Db>` or a plain reference.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p engai-core test_sync`
Expected: All PASS (if compilation issues with `'static`, fix the lifetime)

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add bidirectional Markdown ↔ SQLite sync engine for words and phrases"
```

---

### Task 9: CLI — `add` Command

**Files:**
- Modify: `crates/engai/src/main.rs`
- Create: `crates/engai/src/cmd_add.rs`

- [ ] **Step 1: Implement cmd_add.rs**

```rust
use anyhow::Result;
use clap::Subcommand;
use engai_core::config::Config;
use engai_core::db::Db;

#[derive(Subcommand)]
pub enum AddTarget {
    /// Add a word
    Word { word: String },
    /// Add a phrase
    Phrase { phrase: String },
}

pub async fn run(target: AddTarget) -> Result<()> {
    let config = Config::load_global().await?;
    let db_path = Config::db_path();
    let db = Db::new(&db_path).await?;

    match target {
        AddTarget::Word { word } => {
            let id = db.add_word(&word, None, None).await?;
            println!("Added word: {} (id: {})", word, id);

            let docs_path = Config::docs_path();
            let md_path = docs_path.join("01_vocab").join(format!("{}.md", word));
            if let Some(word_row) = db.get_word(&word).await? {
                let md = engai_core::markdown::MarkdownWord {
                    word: word_row.word,
                    phonetic: word_row.phonetic,
                    familiarity: word_row.familiarity,
                    interval: word_row.interval,
                    next_review: word_row.next_review,
                    meaning: word_row.meaning,
                    examples: vec![],
                    synonyms: vec![],
                    ai_explanation: None,
                    my_notes: vec![],
                    reviews: vec![],
                };
                md.save_to_file(&md_path).await?;
                println!("Created: {}", md_path.display());
            }
        }
        AddTarget::Phrase { phrase } => {
            let id = db.add_phrase(&phrase, None).await?;
            println!("Added phrase: {} (id: {})", phrase, id);

            let docs_path = Config::docs_path();
            let safe_name = phrase.replace(' ', "_");
            let md_path = docs_path.join("02_phrases").join(format!("{}.md", safe_name));
            if let Some(phrase_row) = db.get_phrase(&phrase).await? {
                let md = engai_core::markdown::MarkdownPhrase {
                    phrase: phrase_row.phrase,
                    familiarity: phrase_row.familiarity,
                    interval: phrase_row.interval,
                    next_review: phrase_row.next_review,
                    meaning: phrase_row.meaning,
                    examples: vec![],
                    ai_explanation: None,
                    my_notes: vec![],
                    reviews: vec![],
                };
                md.save_to_file(&md_path).await?;
                println!("Created: {}", md_path.display());
            }
        }
    }

    Ok(())
}
```

- [ ] **Step 2: Update main.rs to wire up `add` subcommand**

Modify the `Commands::Add` variant in `main.rs`:

```rust
Add {
    #[command(subcommand)]
    target: AddTarget,
},
```

Add `mod cmd_add;` and use `cmd_add::AddTarget` in the imports. In the match arm for `Commands::Add`, call `cmd_add::run(target).await?`.

Remove `mod cmd_add.rs` — the submodule files will be separate modules. Update `main.rs`:

```rust
mod cmd_add;
mod cmd_explain;
mod cmd_review;
mod cmd_sync;
mod cmd_read;
mod cmd_import;
mod cmd_export;
mod cmd_stats;
mod cmd_config;
mod cmd_note;

use clap::Parser;

#[derive(Parser)]
#[command(name = "engai", about = "AI English Learning System")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Add a word or phrase
    Add {
        #[command(subcommand)]
        target: cmd_add::AddTarget,
    },
    /// Explain a word or phrase with AI
    Explain {
        #[command(subcommand)]
        target: cmd_explain::ExplainTarget,
    },
    /// Start today's review
    Review { #[arg(long)] all: bool },
    /// Sync Markdown ↔ SQLite
    Sync,
    /// Import and analyze a reading
    Read { file: String },
    /// Import Markdown notes
    Import { path: String },
    /// Export to Markdown
    Export {
        #[arg(long)]
        word: Option<String>,
        #[arg(long)]
        phrase: Option<String>,
        #[arg(long)]
        all: bool,
    },
    /// Show learning statistics
    Stats,
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: cmd_config::ConfigAction,
    },
    /// Manage notes
    Note {
        #[command(subcommand)]
        action: cmd_note::NoteAction,
    },
    /// Start web server only
    #[command(alias = "-s")]
    Server {
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::init();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { target }) => cmd_add::run(target).await?,
        Some(Commands::Explain { target }) => cmd_explain::run(target).await?,
        Some(Commands::Review { all }) => cmd_review::run(all).await?,
        Some(Commands::Sync) => cmd_sync::run().await?,
        Some(Commands::Read { file }) => cmd_read::run(&file).await?,
        Some(Commands::Import { path }) => cmd_import::run(&path).await?,
        Some(Commands::Export { word, phrase, all }) => cmd_export::run(word, phrase, all).await?,
        Some(Commands::Stats) => cmd_stats::run().await?,
        Some(Commands::Config { action }) => cmd_config::run(action).await?,
        Some(Commands::Note { action }) => cmd_note::run(action).await?,
        Some(Commands::Server { port }) => println!("Server mode on port {} (Phase 2)", port),
        None => println!("Run `engai --help` for available commands"),
    }

    Ok(())
}
```

- [ ] **Step 3: Create placeholder stubs for all other cmd modules**

Each file: `cmd_explain.rs`, `cmd_review.rs`, `cmd_sync.rs`, `cmd_read.rs`, `cmd_import.rs`, `cmd_export.rs`, `cmd_stats.rs`, `cmd_config.rs`, `cmd_note.rs`

Each needs the minimum structs/enums and a `run` function that prints "Not yet implemented". Example for `cmd_explain.rs`:

```rust
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ExplainTarget {
    Word { word: String },
    Phrase { phrase: String },
}

pub async fn run(target: ExplainTarget) -> Result<()> {
    match target {
        ExplainTarget::Word { word } => println!("Explain word: {} (not yet implemented)", word),
        ExplainTarget::Phrase { phrase } => println!("Explain phrase: {} (not yet implemented)", phrase),
    }
    Ok(())
}
```

Similarly create stubs for all other cmd modules with appropriate structs matching the `main.rs` enum definitions.

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p engai`
Expected: Compiles

- [ ] **Step 5: Test `engai add word test` manually**

Run: `cargo run -p engai -- add word test`
Expected: Prints "Added word: test (id: 1)" and creates `docs/01_vocab/test.md`

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: implement CLI add command with Markdown file generation"
```

---

### Task 10: CLI — `explain` Command

**Files:**
- Modify: `crates/engai/src/cmd_explain.rs`

- [ ] **Step 1: Implement cmd_explain.rs**

```rust
use anyhow::Result;
use clap::Subcommand;
use engai_core::config::Config;
use engai_core::db::Db;

#[derive(Subcommand)]
pub enum ExplainTarget {
    /// Explain a word with AI
    Word { word: String },
    /// Explain a phrase with AI
    Phrase { phrase: String },
}

pub async fn run(target: ExplainTarget) -> Result<()> {
    let config = Config::load_global().await?;
    let db_path = Config::db_path();
    let db = Db::new(&db_path).await?;

    let ai = engai_core::ai::AiClient::from_config(&config)?;
    let prompt_engine = engai_core::prompt::PromptEngine::new(
        std::path::PathBuf::from("prompts"),
    );

    match target {
        ExplainTarget::Word { word } => {
            println!("Explaining word: {}...", word);
            let explanation = ai.explain_word(&word, &prompt_engine).await?;

            db.update_word(&word, None, Some(&explanation), None, None, None, None)
                .await?;

            let docs_path = Config::docs_path();
            let md_path = docs_path.join("01_vocab").join(format!("{}.md", word));
            if md_path.exists() {
                if let Ok(mut md) =
                    engai_core::markdown::MarkdownWord::parse_file(&md_path).await
                {
                    md.ai_explanation = Some(explanation);
                    md.save_to_file(&md_path).await?;
                    println!("Updated: {}", md_path.display());
                }
            } else {
                db.add_word(&word, None, None).await?;
                let md = engai_core::markdown::MarkdownWord {
                    word: word.clone(),
                    phonetic: None,
                    familiarity: 0,
                    interval: 0,
                    next_review: None,
                    meaning: None,
                    examples: vec![],
                    synonyms: vec![],
                    ai_explanation: Some(explanation),
                    my_notes: vec![],
                    reviews: vec![],
                };
                md.save_to_file(&md_path).await?;
                println!("Created: {}", md_path.display());
            }

            println!("Done.");
        }
        ExplainTarget::Phrase { phrase } => {
            println!("Explaining phrase: {}...", phrase);
            let explanation = ai.explain_phrase(&phrase, &prompt_engine).await?;

            db.update_phrase(&phrase, Some(&explanation), None, None, None, None)
                .await?;

            let docs_path = Config::docs_path();
            let safe_name = phrase.replace(' ', "_");
            let md_path = docs_path.join("02_phrases").join(format!("{}.md", safe_name));
            if md_path.exists() {
                if let Ok(mut md) =
                    engai_core::markdown::MarkdownPhrase::parse_file(&md_path).await
                {
                    md.ai_explanation = Some(explanation);
                    md.save_to_file(&md_path).await?;
                    println!("Updated: {}", md_path.display());
                }
            } else {
                db.add_phrase(&phrase, None).await?;
                let md = engai_core::markdown::MarkdownPhrase {
                    phrase: phrase.clone(),
                    familiarity: 0,
                    interval: 0,
                    next_review: None,
                    meaning: None,
                    examples: vec![],
                    ai_explanation: Some(explanation),
                    my_notes: vec![],
                    reviews: vec![],
                };
                md.save_to_file(&md_path).await?;
                println!("Created: {}", md_path.display());
            }

            println!("Done.");
        }
    }

    Ok(())
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p engai`

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: implement CLI explain command with AI integration"
```

---

### Task 11: CLI — `review` Command

**Files:**
- Modify: `crates/engai/src/cmd_review.rs`

- [ ] **Step 1: Implement cmd_review.rs**

```rust
use anyhow::Result;
use engai_core::config::Config;
use engai_core::db::Db;
use engai_core::review::calculate_next_review;

pub async fn run(show_all: bool) -> Result<()> {
    let config = Config::load_global().await?;
    let db_path = Config::db_path();
    let db = Db::new(&db_path).await?;

    if show_all {
        let words = db.list_words(None, None, 1000, 0).await?;
        let phrases = db.list_phrases(None, None, 1000, 0).await?;
        println!("=== All Words ({}) ===", words.len());
        for w in &words {
            println!(
                "  {} | familiarity: {} | interval: {}d | next: {}",
                w.word,
                w.familiarity,
                w.interval,
                w.next_review
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or("-".to_string())
            );
        }
        println!("\n=== All Phrases ({}) ===", phrases.len());
        for p in &phrases {
            println!(
                "  {} | familiarity: {} | interval: {}d | next: {}",
                p.phrase,
                p.familiarity,
                p.interval,
                p.next_review
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or("-".to_string())
            );
        }
        return Ok(());
    }

    let review_words = db.get_today_review_words().await?;
    let review_phrases = db.get_today_review_phrases().await?;

    let total = review_words.len() + review_phrases.len();
    if total == 0 {
        println!("No reviews due today. Great job!");
        return Ok(());
    }

    println!("Today's review: {} words, {} phrases (total: {})", review_words.len(), review_phrases.len(), total);

    for word in &review_words {
        println!("\n--- {} ---", word.word);
        if let Some(m) = &word.meaning {
            println!("Meaning: {}", m);
        }
        println!("Familiarity: {} | Interval: {}d", word.familiarity, word.interval);
        println!("Rate your recall (0-5): ");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let quality: i32 = input.trim().parse().unwrap_or(3);

        let result = calculate_next_review(quality, word.interval, word.ease_factor);
        db.update_word(
            &word.word,
            None,
            None,
            Some(result.familiarity),
            Some(result.next_review),
            Some(result.interval),
            Some(result.ease_factor),
        )
        .await?;
        db.add_review("word", word.id, quality).await?;
        println!(
            "→ familiarity: {} | next: {} | interval: {}d",
            result.familiarity,
            result.next_review.format("%Y-%m-%d"),
            result.interval
        );
    }

    for phrase in &review_phrases {
        println!("\n--- {} ---", phrase.phrase);
        if let Some(m) = &phrase.meaning {
            println!("Meaning: {}", m);
        }
        println!("Familiarity: {} | Interval: {}d", phrase.familiarity, phrase.interval);
        println!("Rate your recall (0-5): ");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let quality: i32 = input.trim().parse().unwrap_or(3);

        let result = calculate_next_review(quality, phrase.interval, phrase.ease_factor);
        db.update_phrase(
            &phrase.phrase,
            None,
            Some(result.familiarity),
            Some(result.next_review),
            Some(result.interval),
            Some(result.ease_factor),
        )
        .await?;
        db.add_review("phrase", phrase.id, quality).await?;
        println!(
            "→ familiarity: {} | next: {} | interval: {}d",
            result.familiarity,
            result.next_review.format("%Y-%m-%d"),
            result.interval
        );
    }

    println!("\nReview complete! {} items reviewed.", total);
    Ok(())
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p engai`

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: implement CLI review command with interactive SM-2 spaced repetition"
```

---

### Task 12: CLI — `sync`, `read`, `import`, `export`, `stats`, `config`, `note` Commands

**Files:**
- Modify: `crates/engai/src/cmd_sync.rs`
- Modify: `crates/engai/src/cmd_read.rs`
- Modify: `crates/engai/src/cmd_import.rs`
- Modify: `crates/engai/src/cmd_export.rs`
- Modify: `crates/engai/src/cmd_stats.rs`
- Modify: `crates/engai/src/cmd_config.rs`
- Modify: `crates/engai/src/cmd_note.rs`

- [ ] **Step 1: Implement cmd_sync.rs**

```rust
use anyhow::Result;
use engai_core::config::Config;
use engai_core::db::Db;
use engai_core::sync::SyncEngine;
use std::sync::Arc;

pub async fn run() -> Result<()> {
    let config = Config::load_global().await?;
    let db_path = Config::db_path();
    let db = Arc::new(Db::new(&db_path).await?);
    let docs_path = Config::docs_path();
    let prompts_path = std::path::PathBuf::from("prompts");

    println!("Syncing Markdown ↔ SQLite...");
    let engine = SyncEngine::new(db, &docs_path, &prompts_path);
    engine.sync_all().await?;
    println!("Sync complete.");

    Ok(())
}
```

- [ ] **Step 2: Implement cmd_read.rs**

```rust
use anyhow::Result;
use engai_core::config::Config;
use engai_core::db::Db;

pub async fn run(file: &str) -> Result<()> {
    let config = Config::load_global().await?;
    let db_path = Config::db_path();
    let db = Db::new(&db_path).await?;

    let content = tokio::fs::read_to_string(file).await?;
    println!("Importing reading: {} ({} bytes)", file, content.len());

    let id = db
        .add_reading(
            Some(file),
            &content,
            None,
        )
        .await?;
    println!("Saved to database (id: {})", id);

    if config.resolve_api_key().is_empty() {
        println!("Skipping AI analysis: no API key configured.");
        return Ok(());
    }

    let ai = engai_core::ai::AiClient::from_config(&config)?;
    let prompt_engine =
        engai_core::prompt::PromptEngine::new(std::path::PathBuf::from("prompts"));

    println!("Analyzing with AI...");
    let analysis = ai.analyze_reading(&content, &prompt_engine).await?;
    println!("{}", analysis);

    let docs_path = Config::docs_path();
    let title = file.split('/').last().unwrap_or("reading");
    let safe_title = title.replace('.', "_");
    let md_path = docs_path
        .join("03_reading")
        .join(format!("{}.md", safe_title));

    let md = engai_core::markdown::MarkdownReading {
        title: title.to_string(),
        source: None,
        content,
        vocabulary: vec![],
        summary: Some(analysis),
        my_notes: vec![],
    };
    md.save_to_file(&md_path).await?;
    println!("Saved: {}", md_path.display());

    Ok(())
}
```

- [ ] **Step 3: Implement cmd_import.rs**

```rust
use anyhow::{Context, Result};
use engai_core::config::Config;
use engai_core::db::Db;
use std::path::Path;

pub async fn run(path: &str) -> Result<()> {
    let config = Config::load_global().await?;
    let db_path = Config::db_path();
    let db = Db::new(&db_path).await?;
    let docs_path = Config::docs_path();

    let path = Path::new(path);
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }

    let mut count = 0i32;

    if path.is_dir() {
        import_dir(path, &db, &docs_path, &mut count).await?;
    } else {
        import_file(path, &db, &docs_path, &mut count).await?;
    }

    println!("Imported {} items.", count);
    Ok(())
}

async fn import_file(
    path: &Path,
    db: &Db,
    docs_base: &Path,
    count: &mut i32,
) -> Result<()> {
    let relative = path
        .file_name()
        .context("No filename")?
        .to_string_lossy()
        .to_string();

    let parent = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string());

    match parent.as_deref() {
        Some("01_vocab") | Some("vocab") => {
            if let Ok(md) = engai_core::markdown::MarkdownWord::parse_file(path).await {
                if db.get_word(&md.word).await?.is_none() {
                    db.add_word(&md.word, md.phonetic.as_deref(), md.meaning.as_deref())
                        .await?;
                }
                db.update_word(
                    &md.word,
                    md.phonetic.as_deref(),
                    md.meaning.as_deref(),
                    Some(md.familiarity),
                    md.next_review,
                    Some(md.interval),
                    None,
                )
                .await?;
                *count += 1;
            }
        }
        Some("02_phrases") | Some("phrases") => {
            if let Ok(md) = engai_core::markdown::MarkdownPhrase::parse_file(path).await {
                if db.get_phrase(&md.phrase).await?.is_none() {
                    db.add_phrase(&md.phrase, md.meaning.as_deref()).await?;
                }
                db.update_phrase(
                    &md.phrase,
                    md.meaning.as_deref(),
                    Some(md.familiarity),
                    md.next_review,
                    Some(md.interval),
                    None,
                )
                .await?;
                *count += 1;
            }
        }
        _ => {
            println!("Skipping unrecognized file: {}", relative);
        }
    }

    Ok(())
}

async fn import_dir(
    dir: &Path,
    db: &Db,
    docs_base: &Path,
    count: &mut i32,
) -> Result<()> {
    let mut entries = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            Box::pin(import_dir(&path, db, docs_base, count)).await?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            import_file(&path, db, docs_base, count).await?;
        }
    }
    Ok(())
}
```

- [ ] **Step 4: Implement cmd_export.rs**

```rust
use anyhow::Result;
use engai_core::config::Config;
use engai_core::db::Db;
use engai_core::markdown::{MarkdownPhrase, MarkdownWord};

pub async fn run(
    word: Option<String>,
    phrase: Option<String>,
    all: bool,
) -> Result<()> {
    let config = Config::load_global().await?;
    let db_path = Config::db_path();
    let db = Db::new(&db_path).await?;
    let docs_path = Config::docs_path();

    if all {
        let words = db.list_words(None, None, 10000, 0).await?;
        for w in &words {
            let examples = db.get_examples("word", w.id).await.unwrap_or_default();
            let notes = db.get_notes("word", w.id).await.unwrap_or_default();
            let md = MarkdownWord {
                word: w.word.clone(),
                phonetic: w.phonetic.clone(),
                familiarity: w.familiarity,
                interval: w.interval,
                next_review: w.next_review,
                meaning: w.meaning.clone(),
                examples: examples.iter().map(|e| e.sentence.clone()).collect(),
                synonyms: vec![],
                ai_explanation: None,
                my_notes: notes.iter().map(|n| n.content.clone()).collect(),
                reviews: vec![],
            };
            let md_path = docs_path.join("01_vocab").join(format!("{}.md", w.word));
            md.save_to_file(&md_path).await?;
        }
        println!("Exported {} words.", words.len());

        let phrases = db.list_phrases(None, None, 10000, 0).await?;
        for p in &phrases {
            let examples = db.get_examples("phrase", p.id).await.unwrap_or_default();
            let notes = db.get_notes("phrase", p.id).await.unwrap_or_default();
            let md = MarkdownPhrase {
                phrase: p.phrase.clone(),
                familiarity: p.familiarity,
                interval: p.interval,
                next_review: p.next_review,
                meaning: p.meaning.clone(),
                examples: examples.iter().map(|e| e.sentence.clone()).collect(),
                ai_explanation: None,
                my_notes: notes.iter().map(|n| n.content.clone()).collect(),
                reviews: vec![],
            };
            let safe_name = p.phrase.replace(' ', "_");
            let md_path = docs_path.join("02_phrases").join(format!("{}.md", safe_name));
            md.save_to_file(&md_path).await?;
        }
        println!("Exported {} phrases.", phrases.len());
        return Ok(());
    }

    if let Some(word) = word {
        if let Some(w) = db.get_word(&word).await? {
            let examples = db.get_examples("word", w.id).await.unwrap_or_default();
            let notes = db.get_notes("word", w.id).await.unwrap_or_default();
            let md = MarkdownWord {
                word: w.word.clone(),
                phonetic: w.phonetic.clone(),
                familiarity: w.familiarity,
                interval: w.interval,
                next_review: w.next_review,
                meaning: w.meaning.clone(),
                examples: examples.iter().map(|e| e.sentence.clone()).collect(),
                synonyms: vec![],
                ai_explanation: None,
                my_notes: notes.iter().map(|n| n.content.clone()).collect(),
                reviews: vec![],
            };
            let md_path = docs_path.join("01_vocab").join(format!("{}.md", w.word));
            md.save_to_file(&md_path).await?;
            println!("Exported: {}", md_path.display());
        } else {
            println!("Word not found: {}", word);
        }
    }

    if let Some(phrase) = phrase {
        if let Some(p) = db.get_phrase(&phrase).await? {
            let examples = db.get_examples("phrase", p.id).await.unwrap_or_default();
            let notes = db.get_notes("phrase", p.id).await.unwrap_or_default();
            let md = MarkdownPhrase {
                phrase: p.phrase.clone(),
                familiarity: p.familiarity,
                interval: p.interval,
                next_review: p.next_review,
                meaning: p.meaning.clone(),
                examples: examples.iter().map(|e| e.sentence.clone()).collect(),
                ai_explanation: None,
                my_notes: notes.iter().map(|n| n.content.clone()).collect(),
                reviews: vec![],
            };
            let safe_name = p.phrase.replace(' ', "_");
            let md_path = docs_path.join("02_phrases").join(format!("{}.md", safe_name));
            md.save_to_file(&md_path).await?;
            println!("Exported: {}", md_path.display());
        } else {
            println!("Phrase not found: {}", phrase);
        }
    }

    Ok(())
}
```

- [ ] **Step 5: Implement cmd_stats.rs**

```rust
use anyhow::Result;
use engai_core::config::Config;
use engai_core::db::Db;

pub async fn run() -> Result<()> {
    let config = Config::load_global().await?;
    let db_path = Config::db_path();
    let db = Db::new(&db_path).await?;

    let words = db.word_count().await?;
    let phrases = db.phrase_count().await?;
    let reviewed_today = db.review_count_today().await?;
    let pending = db.pending_review_count().await?;

    println!("=== Engai Learning Stats ===");
    println!("Total words:     {}", words);
    println!("Total phrases:   {}", phrases);
    println!("Reviewed today:  {}", reviewed_today);
    println!("Pending review:  {}", pending);
    println!("DB path:         {}", db_path.display());
    println!("Docs path:       {}", Config::docs_path().display());
    println!("AI provider:     {}", config.ai.provider);

    Ok(())
}
```

- [ ] **Step 6: Implement cmd_config.rs**

```rust
use anyhow::Result;
use clap::Subcommand;
use engai_core::config::Config;

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Initialize configuration interactively
    Init,
    /// Set a config value
    Set { key: String, value: String },
    /// Get a config value
    Get { key: String },
}

pub async fn run(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Init => {
            let path = Config::config_file_path();
            if path.exists() {
                println!("Config already exists at: {}", path.display());
                println!("Edit it directly or use `engai config set <key> <value>`.");
                return Ok(());
            }
            let config = Config::default();
            config.save_to(&path).await?;
            println!("Config created at: {}", path.display());
            println!("\nNext steps:");
            println!("  1. Set your AI API key:");
            println!("     engai config set ai.api_key YOUR_KEY");
            println!("  2. Or set env var: ENGAI_AI_API_KEY=YOUR_KEY");
        }
        ConfigAction::Set { key, value } => {
            let mut config = Config::load_global().await?;
            set_config_value(&mut config, &key, &value)?;
            let path = Config::config_file_path();
            config.save_to(&path).await?;
            println!("Set {} = {}", key, value);
        }
        ConfigAction::Get { key } => {
            let config = Config::load_global().await?;
            let value = get_config_value(&config, &key)?;
            println!("{} = {}", key, value);
        }
    }
    Ok(())
}

fn set_config_value(config: &mut Config, key: &str, value: &str) -> Result<()> {
    match key {
        "server.port" => config.server.port = value.parse()?,
        "server.host" => config.server.host = value.to_string(),
        "ai.provider" => config.ai.provider = value.to_string(),
        "ai.api_key" => config.ai.api_key = value.to_string(),
        "ai.model" => config.ai.model = value.to_string(),
        "ai.base_url" => config.ai.base_url = value.to_string(),
        "learning.daily_new_words" => config.learning.daily_new_words = value.parse()?,
        "learning.daily_review_limit" => config.learning.daily_review_limit = value.parse()?,
        "storage.db_path" => config.storage.db_path = value.to_string(),
        "storage.docs_path" => config.storage.docs_path = value.to_string(),
        _ => anyhow::bail!("Unknown config key: {}", key),
    }
    Ok(())
}

fn get_config_value(config: &Config, key: &str) -> Result<String> {
    match key {
        "server.port" => Ok(config.server.port.to_string()),
        "server.host" => Ok(config.server.host.clone()),
        "ai.provider" => Ok(config.ai.provider.clone()),
        "ai.api_key" => Ok(if config.ai.api_key.is_empty() {
            "***".to_string()
        } else {
            "***".to_string()
        }),
        "ai.model" => Ok(config.ai.model.clone()),
        "ai.base_url" => Ok(config.ai.base_url.clone()),
        "learning.daily_new_words" => Ok(config.learning.daily_new_words.to_string()),
        "learning.daily_review_limit" => Ok(config.learning.daily_review_limit.to_string()),
        "storage.db_path" => Ok(config.storage.db_path.clone()),
        "storage.docs_path" => Ok(config.storage.docs_path.clone()),
        _ => anyhow::bail!("Unknown config key: {}", key),
    }
}
```

- [ ] **Step 7: Implement cmd_note.rs**

```rust
use anyhow::Result;
use clap::Subcommand;
use engai_core::config::Config;
use engai_core::db::Db;

#[derive(Subcommand)]
pub enum NoteAction {
    /// Add a note to a target
    Add {
        target_type: String,
        target_id: i64,
        content: Vec<String>,
    },
}

pub async fn run(action: NoteAction) -> Result<()> {
    match action {
        NoteAction::Add {
            target_type,
            target_id,
            content,
        } => {
            if !["word", "phrase", "reading"].contains(&target_type.as_str()) {
                anyhow::bail!("target_type must be 'word', 'phrase', or 'reading'");
            }
            let content = content.join(" ");
            if content.is_empty() {
                anyhow::bail!("Note content cannot be empty");
            }

            let db_path = Config::db_path();
            let db = Db::new(&db_path).await?;
            let id = db.add_note(&target_type, target_id, &content).await?;
            println!("Added note (id: {}) to {} #{}", id, target_type, target_id);
        }
    }
    Ok(())
}
```

- [ ] **Step 8: Verify full compilation**

Run: `cargo build -p engai`
Expected: Compiles successfully

- [ ] **Step 9: Test all CLI commands**

```bash
cargo run -p engai -- config init
cargo run -p engai -- add word hello
cargo run -p engai -- add phrase "take off"
cargo run -p engai -- stats
cargo run -p engai -- sync
cargo run -p engai -- review --all
cargo run -p engai -- export --all
```

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "feat: implement all CLI commands (sync, read, import, export, stats, config, note)"
```

---

### Task 13: Final Integration Test and Cleanup

**Files:**
- Create: `crates/engai-core/tests/test_integration.rs`

- [ ] **Step 1: Write integration test**

`crates/engai-core/tests/test_integration.rs`:
```rust
use engai_core::db::Db;
use engai_core::markdown::{MarkdownWord, MarkdownPhrase};
use engai_core::review::calculate_next_review;

#[tokio::test]
async fn test_full_word_lifecycle() {
    let db = Db::new_in_memory().await.unwrap();

    // Add word
    let id = db.add_word("test", Some("/test/"), Some("a test word")).await.unwrap();
    assert!(id > 0);

    // Get word
    let word = db.get_word("test").await.unwrap().unwrap();
    assert_eq!(word.phonetic.as_deref(), Some("/test/"));

    // Add example
    db.add_example("word", id, "this is a test", Some("unit test")).await.unwrap();
    let examples = db.get_examples("word", id).await.unwrap();
    assert_eq!(examples.len(), 1);

    // Add note
    db.add_note("word", id, "my note about test").await.unwrap();
    let notes = db.get_notes("word", id).await.unwrap();
    assert_eq!(notes.len(), 1);

    // Review cycle
    let result = calculate_next_review(5, 0, 2.5);
    db.update_word(
        "test", None, None,
        Some(result.familiarity),
        Some(result.next_review),
        Some(result.interval),
        Some(result.ease_factor),
    ).await.unwrap();

    let updated = db.get_word("test").await.unwrap().unwrap();
    assert!(updated.familiarity > 0);
    assert!(updated.interval > 0);

    // Add review record
    db.add_review("word", id, 5).await.unwrap();
    let reviews = db.get_reviews("word", id).await.unwrap();
    assert_eq!(reviews.len(), 1);

    // Markdown roundtrip
    let md = MarkdownWord {
        word: updated.word.clone(),
        phonetic: updated.phonetic,
        familiarity: updated.familiarity,
        interval: updated.interval,
        next_review: updated.next_review,
        meaning: updated.meaning,
        examples: examples.iter().map(|e| e.sentence.clone()).collect(),
        synonyms: vec!["exam".to_string()],
        ai_explanation: Some("Test is a word for checking.".to_string()),
        my_notes: notes.iter().map(|n| n.content.clone()).collect(),
        reviews: reviews.iter().map(|r| format!("{} ⭐", r.reviewed_at.format("%Y-%m-%d"))).collect(),
    };

    let md_string = md.to_markdown_string();
    let reparsed = MarkdownWord::parse(&md_string).unwrap();
    assert_eq!(reparsed.word, md.word);
    assert_eq!(reparsed.familiarity, md.familiarity);
    assert_eq!(reparsed.synonyms.len(), 1);

    // Delete
    db.delete_word("test").await.unwrap();
    assert!(db.get_word("test").await.unwrap().is_none());
}

#[tokio::test]
async fn test_full_phrase_lifecycle() {
    let db = Db::new_in_memory().await.unwrap();

    let id = db.add_phrase("give up", Some("to stop trying")).await.unwrap();
    let phrase = db.get_phrase("give up").await.unwrap().unwrap();
    assert_eq!(phrase.meaning.as_deref(), Some("to stop trying"));

    db.add_example("phrase", id, "I gave up on math", Some("cli")).await.unwrap();

    let result = calculate_next_review(4, 0, 2.5);
    db.update_phrase(
        "give up", None,
        Some(result.familiarity),
        Some(result.next_review),
        Some(result.interval),
        Some(result.ease_factor),
    ).await.unwrap();

    let md = MarkdownPhrase {
        phrase: "give up".to_string(),
        familiarity: result.familiarity,
        interval: result.interval,
        next_review: result.next_review,
        meaning: Some("to stop trying".to_string()),
        examples: vec!["I gave up on math".to_string()],
        ai_explanation: None,
        my_notes: vec![],
        reviews: vec![],
    };

    let md_string = md.to_markdown_string();
    let reparsed = MarkdownPhrase::parse(&md_string).unwrap();
    assert_eq!(reparsed.phrase, "give up");

    db.delete_phrase("give up").await.unwrap();
    assert!(db.get_phrase("give up").await.unwrap().is_none());
}
```

- [ ] **Step 2: Run all tests**

Run: `cargo test -p engai-core`
Expected: All PASS

- [ ] **Step 3: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Fix any warnings.

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "test: add integration tests for full word and phrase lifecycles"
```

---

## Summary

Phase 1 produces a fully functional CLI-based English learning system:

| Command | Status |
|---------|--------|
| `engai add word/phrase` | Working |
| `engai explain word/phrase` | Working (needs API key) |
| `engai review` | Working with SM-2 |
| `engai sync` | Working (bidirectional) |
| `engai read <file>` | Working (with AI analysis) |
| `engai import <path>` | Working |
| `engai export` | Working |
| `engai stats` | Working |
| `engai config init/set/get` | Working |
| `engai note add` | Working |

After Phase 1, proceed to Phase 2 plan (Axum Web Server + React Frontend).
