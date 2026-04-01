# Phase 1: Backend 3-Tier Layering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split engai-core's flat `db.rs` and `models.rs` into a clean 3-tier architecture (models → repositories → services) within the existing crate structure.

**Architecture:** Models define pure data types. Repositories provide raw SQL access. Services contain business logic, validation, and orchestration. Handlers/CLI/TUI call services only.

**Tech Stack:** Rust, sqlx 0.8, axum 0.8, tokio, serde

---

### Task 1: Split models.rs into models/ directory

**Files:**
- Create: `crates/engai-core/src/models/mod.rs`
- Create: `crates/engai-core/src/models/word.rs`
- Create: `crates/engai-core/src/models/phrase.rs`
- Create: `crates/engai-core/src/models/example.rs`
- Create: `crates/engai-core/src/models/review.rs`
- Create: `crates/engai-core/src/models/reading.rs`
- Create: `crates/engai-core/src/models/note.rs`
- Create: `crates/engai-core/src/models/chat.rs`
- Create: `crates/engai-core/src/models/query.rs`
- Delete: `crates/engai-core/src/models.rs`

- [ ] **Step 1: Create models/ directory and split files**

Move each struct from the monolithic `models.rs` into its own file. Each file re-exports its types. `mod.rs` re-exports everything.

`models/mod.rs`:
```rust
pub mod word;
pub mod phrase;
pub mod example;
pub mod review;
pub mod reading;
pub mod note;
pub mod chat;
pub mod query;

pub use word::*;
pub use phrase::*;
pub use example::*;
pub use review::*;
pub use reading::*;
pub use note::*;
pub use chat::*;
pub use query::*;
```

`models/word.rs`:
```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Word {
    pub id: i64,
    pub word: String,
    pub phonetic: Option<String>,
    pub meaning: String,
    pub familiarity: i32,
    pub next_review: Option<NaiveDateTime>,
    pub interval: i32,
    pub ease_factor: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewWord {
    pub word: String,
    pub phonetic: Option<String>,
    pub meaning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWord {
    pub word: Option<String>,
    pub phonetic: Option<String>,
    pub meaning: Option<String>,
    pub familiarity: Option<i32>,
    pub next_review: Option<NaiveDateTime>,
    pub interval: Option<i32>,
    pub ease_factor: Option<f64>,
}
```

`models/phrase.rs`:
```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Phrase {
    pub id: i64,
    pub phrase: String,
    pub meaning: String,
    pub familiarity: i32,
    pub next_review: Option<NaiveDateTime>,
    pub interval: i32,
    pub ease_factor: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPhrase {
    pub phrase: String,
    pub meaning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePhrase {
    pub phrase: Option<String>,
    pub meaning: Option<String>,
    pub familiarity: Option<i32>,
    pub next_review: Option<NaiveDateTime>,
    pub interval: Option<i32>,
    pub ease_factor: Option<f64>,
}
```

`models/example.rs`:
```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Example {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub sentence: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewExample {
    pub target_type: String,
    pub target_id: i64,
    pub sentence: String,
    pub source: Option<String>,
}
```

`models/review.rs`:
```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Review {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub quality: i32,
    pub reviewed_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSubmit {
    pub quality: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewStats {
    pub pending_reviews: i64,
    pub reviewed_today: i64,
}
```

`models/reading.rs`:
```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Reading {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub source: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewReading {
    pub title: String,
    pub content: String,
    pub source: Option<String>,
}
```

`models/note.rs`:
```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Note {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewNote {
    pub target_type: String,
    pub target_id: i64,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNote {
    pub content: String,
}
```

`models/chat.rs`:
```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatEntry {
    pub id: i64,
    pub role: String,
    pub content: String,
    pub created_at: NaiveDateTime,
}
```

`models/query.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordQuery {
    pub search: Option<String>,
    pub familiarity_gte: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhraseQuery {
    pub search: Option<String>,
    pub familiarity_gte: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteQuery {
    pub target_type: Option<String>,
    pub target_id: Option<i64>,
}
```

- [ ] **Step 2: Update lib.rs to use models as module**

Change `crates/engai-core/src/lib.rs`:
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

(Remove `mod models;` flat file, add `pub mod models;` directory — same syntax but Rust picks up the directory.)

- [ ] **Step 3: Delete old models.rs flat file**

Delete `crates/engai-core/src/models.rs`.

- [ ] **Step 4: Update all imports that reference model types**

Search and replace across all files in engai-core and engai crates. Change:
- `use crate::models::{Word, NewWord}` → `use crate::models::{Word, NewWord}` (unchanged due to `pub use *`)
- `use engai_core::models::{Word}` → `use engai_core::models::Word` (unchanged)

Since `mod.rs` re-exports everything with `pub use *`, existing imports continue to work.

- [ ] **Step 5: Verify compilation**

Run: `cargo check`
Expected: success

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor: split models.rs into models/ directory"
```

---

### Task 2: Split db.rs into db/ directory with repositories

**Files:**
- Create: `crates/engai-core/src/db/mod.rs`
- Create: `crates/engai-core/src/db/pool.rs`
- Create: `crates/engai-core/src/db/repositories/mod.rs`
- Create: `crates/engai-core/src/db/repositories/word_repository.rs`
- Create: `crates/engai-core/src/db/repositories/phrase_repository.rs`
- Create: `crates/engai-core/src/db/repositories/example_repository.rs`
- Create: `crates/engai-core/src/db/repositories/review_repository.rs`
- Create: `crates/engai-core/src/db/repositories/reading_repository.rs`
- Create: `crates/engai-core/src/db/repositories/note_repository.rs`
- Create: `crates/engai-core/src/db/repositories/chat_repository.rs`
- Delete: `crates/engai-core/src/db.rs`

- [ ] **Step 1: Create db/pool.rs**

Extract the `Db` struct, constructors, and pool access from `db.rs`:

```rust
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteJournalMode, SqliteSynchronous};
use sqlx::SqlitePool;
use std::str::FromStr;

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn new(db_path: &str) -> anyhow::Result<Self> {
        let options = SqliteConnectOptions::from_str(db_path)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn new_in_memory() -> anyhow::Result<Self> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")?
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
```

- [ ] **Step 2: Create db/repositories/word_repository.rs**

Move all word-related query methods from `db.rs`. Each repository takes `&SqlitePool`:

```rust
use sqlx::SqlitePool;
use crate::models::*;

pub struct WordRepository {
    pool: SqlitePool,
}

impl WordRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self, search: Option<&str>, familiarity_gte: Option<i32>, limit: i64, offset: i64) -> anyhow::Result<Vec<Word>> {
        let words = if let Some(q) = search {
            if let Some(fam) = familiarity_gte {
                sqlx::query_as::<_, Word>("SELECT * FROM words WHERE word LIKE ? AND familiarity >= ? ORDER BY updated_at DESC LIMIT ? OFFSET ?")
                    .bind(format!("%{}%", q)).bind(fam).bind(limit).bind(offset)
                    .fetch_all(&self.pool).await?
            } else {
                sqlx::query_as::<_, Word>("SELECT * FROM words WHERE word LIKE ? ORDER BY updated_at DESC LIMIT ? OFFSET ?")
                    .bind(format!("%{}%", q)).bind(limit).bind(offset)
                    .fetch_all(&self.pool).await?
            }
        } else if let Some(fam) = familiarity_gte {
            sqlx::query_as::<_, Word>("SELECT * FROM words WHERE familiarity >= ? ORDER BY updated_at DESC LIMIT ? OFFSET ?")
                .bind(fam).bind(limit).bind(offset)
                .fetch_all(&self.pool).await?
        } else {
            sqlx::query_as::<_, Word>("SELECT * FROM words ORDER BY updated_at DESC LIMIT ? OFFSET ?")
                .bind(limit).bind(offset)
                .fetch_all(&self.pool).await?
        };
        Ok(words)
    }

    pub async fn find_by_word(&self, word: &str) -> anyhow::Result<Option<Word>> {
        let result = sqlx::query_as::<_, Word>("SELECT * FROM words WHERE word = ?")
            .bind(word)
            .fetch_optional(&self.pool).await?;
        Ok(result)
    }

    pub async fn find_by_id(&self, id: i64) -> anyhow::Result<Option<Word>> {
        let result = sqlx::query_as::<_, Word>("SELECT * FROM words WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool).await?;
        Ok(result)
    }

    pub async fn create(&self, input: &NewWord) -> anyhow::Result<Word> {
        let word = sqlx::query_as::<_, Word>(
            "INSERT INTO words (word, phonetic, meaning) VALUES (?, ?, ?) RETURNING *"
        ).bind(&input.word).bind(&input.phonetic).bind(&input.meaning)
            .fetch_one(&self.pool).await?;
        Ok(word)
    }

    pub async fn update(&self, id: i64, word: Option<&str>, phonetic: Option<&str>, meaning: Option<&str>, familiarity: Option<i32>, next_review: Option<chrono::NaiveDateTime>, interval: Option<i32>, ease_factor: Option<f64>) -> anyhow::Result<Word> {
        let current = self.find_by_id(id).await?.ok_or_else(|| anyhow::anyhow!("Word not found"))?;
        let word = sqlx::query_as::<_, Word>(
            "UPDATE words SET word = ?, phonetic = ?, meaning = ?, familiarity = ?, next_review = ?, interval = ?, ease_factor = ? WHERE id = ? RETURNING *"
        ).bind(word.unwrap_or(&current.word))
         .bind(phonetic.or(current.phonetic.as_deref()))
         .bind(meaning.unwrap_or(&current.meaning))
         .bind(familiarity.unwrap_or(current.familiarity))
         .bind(next_review.or(current.next_review))
         .bind(interval.unwrap_or(current.interval))
         .bind(ease_factor.unwrap_or(current.ease_factor))
         .bind(id)
         .fetch_one(&self.pool).await?;
        Ok(word)
    }

    pub async fn delete(&self, id: i64) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM words WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_due_reviews(&self) -> anyhow::Result<Vec<Word>> {
        let now = chrono::Utc::now().naive_utc();
        let words = sqlx::query_as::<_, Word>(
            "SELECT * FROM words WHERE next_review IS NOT NULL AND next_review <= ? ORDER BY next_review"
        ).bind(now).fetch_all(&self.pool).await?;
        Ok(words)
    }

    pub async fn count(&self) -> anyhow::Result<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM words").fetch_one(&self.pool).await?;
        Ok(count.0)
    }
}
```

- [ ] **Step 3: Create remaining repositories**

Follow the same pattern for `phrase_repository.rs`, `example_repository.rs`, `review_repository.rs`, `reading_repository.rs`, `note_repository.rs`, `chat_repository.rs`. Move the corresponding methods from `db.rs` into each repository.

`db/repositories/mod.rs`:
```rust
pub mod word_repository;
pub mod phrase_repository;
pub mod example_repository;
pub mod review_repository;
pub mod reading_repository;
pub mod note_repository;
pub mod chat_repository;

pub use word_repository::WordRepository;
pub use phrase_repository::PhraseRepository;
pub use example_repository::ExampleRepository;
pub use review_repository::ReviewRepository;
pub use reading_repository::ReadingRepository;
pub use note_repository::NoteRepository;
pub use chat_repository::ChatRepository;
```

- [ ] **Step 4: Create db/mod.rs**

```rust
pub mod pool;
pub mod repositories;

pub use pool::Db;
pub use repositories::*;
```

- [ ] **Step 5: Update lib.rs**

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

- [ ] **Step 6: Delete old db.rs**

Delete `crates/engai-core/src/db.rs`.

- [ ] **Step 7: Update all `Db` method callers to use repositories**

This is the breaking change. Every call to `db.list_words(...)` must become `word_repo.find_all(...)`. Update:
- All route files in `crates/engai/src/routes/`
- All CLI command files in `crates/engai/src/cmd_*.rs`
- All TUI panel files in `crates/engai/src/tui/panel_*.rs`
- `sync.rs` in engai-core
- `state.rs` in engai (add repositories to AppState)

Update `crates/engai/src/state.rs`:
```rust
use std::sync::Arc;
use engai_core::db::{Db, WordRepository, PhraseRepository, ExampleRepository, ReviewRepository, ReadingRepository, NoteRepository, ChatRepository};
use engai_core::ai::AiClient;
use engai_core::config::Config;
use engai_core::prompt::PromptEngine;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub config: Config,
    pub ai_client: Arc<AiClient>,
    pub prompt_engine: Arc<PromptEngine>,
    pub word_repo: WordRepository,
    pub phrase_repo: PhraseRepository,
    pub example_repo: ExampleRepository,
    pub review_repo: ReviewRepository,
    pub reading_repo: ReadingRepository,
    pub note_repo: NoteRepository,
    pub chat_repo: ChatRepository,
}

impl AppState {
    pub fn new(db: Db, config: Config) -> anyhow::Result<Self> {
        let pool = db.pool().clone();
        let ai_client = Arc::new(AiClient::from_config(&config));
        let prompts_dir = config.prompts_path();
        let prompt_engine = Arc::new(PromptEngine::new(prompts_dir));
        Ok(Self {
            db: Arc::new(db),
            config,
            ai_client,
            prompt_engine,
            word_repo: WordRepository::new(pool.clone()),
            phrase_repo: PhraseRepository::new(pool.clone()),
            example_repo: ExampleRepository::new(pool.clone()),
            review_repo: ReviewRepository::new(pool.clone()),
            reading_repo: ReadingRepository::new(pool.clone()),
            note_repo: NoteRepository::new(pool.clone()),
            chat_repo: ChatRepository::new(pool),
        })
    }
}
```

- [ ] **Step 8: Verify compilation**

Run: `cargo check`
Expected: success (may require several import fix iterations)

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "refactor: split db.rs into db/ directory with repositories"
```

---

### Task 3: Create services layer

**Files:**
- Create: `crates/engai-core/src/services/mod.rs`
- Create: `crates/engai-core/src/services/word_service.rs`
- Create: `crates/engai-core/src/services/phrase_service.rs`
- Create: `crates/engai-core/src/services/review_service.rs`
- Create: `crates/engai-core/src/services/reading_service.rs`
- Create: `crates/engai-core/src/services/note_service.rs`
- Create: `crates/engai-core/src/services/chat_service.rs`
- Create: `crates/engai-core/src/services/ai_service.rs`
- Create: `crates/engai-core/src/services/stats_service.rs`
- Create: `crates/engai-core/src/services/sync_service.rs`

- [ ] **Step 1: Create services/ai_service.rs**

Wrap AiClient + PromptEngine:

```rust
use std::sync::Arc;
use crate::ai::AiClient;
use crate::prompt::PromptEngine;

#[derive(Clone)]
pub struct AiService {
    client: Arc<AiClient>,
    prompts: Arc<PromptEngine>,
}

impl AiService {
    pub fn new(client: Arc<AiClient>, prompts: Arc<PromptEngine>) -> Self {
        Self { client, prompts }
    }

    pub async fn explain_word(&self, word: &str) -> anyhow::Result<String> {
        self.client.explain_word(word, &self.prompts).await
    }

    pub async fn explain_phrase(&self, phrase: &str) -> anyhow::Result<String> {
        self.client.explain_phrase(phrase, &self.prompts).await
    }

    pub async fn analyze_reading(&self, content: &str) -> anyhow::Result<String> {
        self.client.analyze_reading(content, &self.prompts).await
    }

    pub async fn chat_completion(&self, messages: Vec<crate::ai::ChatMessage>) -> anyhow::Result<String> {
        self.client.chat_completion(&messages).await
    }

    pub fn client(&self) -> &AiClient {
        &self.client
    }
}
```

- [ ] **Step 2: Create services/word_service.rs**

```rust
use crate::db::WordRepository;
use crate::models::*;
use crate::services::AiService;
use std::sync::Arc;

#[derive(Clone)]
pub struct WordService {
    repo: WordRepository,
    ai: Arc<AiService>,
}

impl WordService {
    pub fn new(repo: WordRepository, ai: Arc<AiService>) -> Self {
        Self { repo, ai }
    }

    pub async fn list(&self, search: Option<&str>, familiarity_gte: Option<i32>, limit: i64, offset: i64) -> anyhow::Result<Vec<Word>> {
        self.repo.find_all(search, familiarity_gte, limit, offset).await
    }

    pub async fn get(&self, word: &str) -> anyhow::Result<Word> {
        self.repo.find_by_word(word).await?.ok_or_else(|| anyhow::anyhow!("Word not found: {}", word))
    }

    pub async fn create(&self, input: &NewWord) -> anyhow::Result<Word> {
        if input.word.trim().is_empty() {
            return Err(anyhow::anyhow!("word cannot be empty"));
        }
        self.repo.create(input).await
    }

    pub async fn update(&self, word: &str, input: &UpdateWord) -> anyhow::Result<Word> {
        let current = self.repo.find_by_word(word).await?.ok_or_else(|| anyhow::anyhow!("Word not found"))?;
        self.repo.update(
            current.id,
            input.word.as_deref(),
            input.phonetic.as_deref(),
            input.meaning.as_deref(),
            input.familiarity,
            input.next_review,
            input.interval,
            input.ease_factor,
        ).await
    }

    pub async fn delete(&self, word: &str) -> anyhow::Result<()> {
        let w = self.repo.find_by_word(word).await?.ok_or_else(|| anyhow::anyhow!("Word not found"))?;
        self.repo.delete(w.id).await
    }

    pub async fn explain(&self, word: &str) -> anyhow::Result<String> {
        self.ai.explain_word(word).await
    }

    pub async fn get_examples(&self, word: &str) -> anyhow::Result<Vec<Example>> {
        let w = self.repo.find_by_word(word).await?.ok_or_else(|| anyhow::anyhow!("Word not found"))?;
        self.example_repo.find_by_target("word", w.id).await
    }
}
```

- [ ] **Step 3: Create remaining services**

Follow the same pattern for phrase_service, review_service, reading_service, note_service, chat_service, stats_service, sync_service. Each service:
- Takes its repository as a constructor arg
- Takes AiService as an Arc where needed (word, phrase, reading, chat)
- Contains validation logic
- Delegates to repositories for data access

`services/review_service.rs` should encapsulate the SM-2 logic:
```rust
pub async fn submit_review(&self, target_type: &str, id: i64, quality: i32) -> anyhow::Result<()> {
    let result = crate::review::calculate_next_review(quality, current_interval, current_ease_factor);
    // Update the word/phrase with new review schedule
    // Add review record
}
```

`services/stats_service.rs`:
```rust
pub async fn get_stats(&self) -> anyhow::Result<StatsData> {
    let word_count = self.word_repo.count().await?;
    let phrase_count = self.phrase_repo.count().await?;
    let pending = self.word_repo.count_due().await? + self.phrase_repo.count_due().await?;
    let reviewed_today = self.review_repo.count_today().await?;
    Ok(StatsData { word_count, phrase_count, pending_reviews: pending, reviewed_today })
}
```

- [ ] **Step 4: Create services/mod.rs**

```rust
pub mod ai_service;
pub mod chat_service;
pub mod note_service;
pub mod phrase_service;
pub mod reading_service;
pub mod review_service;
pub mod stats_service;
pub mod sync_service;
pub mod word_service;

pub use ai_service::AiService;
pub use chat_service::ChatService;
pub use note_service::NoteService;
pub use phrase_service::PhraseService;
pub use reading_service::ReadingService;
pub use review_service::ReviewService;
pub use stats_service::StatsService;
pub use sync_service::SyncService;
pub use word_service::WordService;
```

- [ ] **Step 5: Update lib.rs**

```rust
pub mod ai;
pub mod config;
pub mod db;
pub mod markdown;
pub mod models;
pub mod prompt;
pub mod review;
pub mod services;
pub mod sync;
```

- [ ] **Step 6: Verify compilation**

Run: `cargo check`

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: add services layer with AI, word, phrase, review, reading, note, chat, stats, sync services"
```

---

### Task 4: Update routes to use services

**Files:**
- Modify: `crates/engai/src/state.rs` — add services to AppState
- Modify: `crates/engai/src/routes/words.rs` — use WordService
- Modify: `crates/engai/src/routes/phrases.rs` — use PhraseService
- Modify: `crates/engai/src/routes/reviews.rs` — use ReviewService
- Modify: `crates/engai/src/routes/readings.rs` — use ReadingService
- Modify: `crates/engai/src/routes/notes.rs` — use NoteService
- Modify: `crates/engai/src/routes/chat.rs` — use ChatService
- Modify: `crates/engai/src/routes/stats.rs` — use StatsService
- Modify: `crates/engai/src/routes/sync.rs` — use SyncService

- [ ] **Step 1: Update AppState to include services**

```rust
use std::sync::Arc;
use engai_core::db::Db;
use engai_core::config::Config;
use engai_core::services::*;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub config: Config,
    pub word_service: WordService,
    pub phrase_service: PhraseService,
    pub review_service: ReviewService,
    pub reading_service: ReadingService,
    pub note_service: NoteService,
    pub chat_service: ChatService,
    pub ai_service: Arc<AiService>,
    pub stats_service: StatsService,
    pub sync_service: SyncService,
}
```

- [ ] **Step 2: Update each route handler**

Replace direct `db.xxx()` calls with `state.xxx_service.xxx()` calls. For example in `words.rs`:

Before:
```rust
let words = state.db.list_words(search, familiarity_gte, limit, offset).await?;
```

After:
```rust
let words = state.word_service.list(search, familiarity_gte, limit, offset).await?;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "refactor: update routes to use services instead of direct db access"
```

---

### Task 5: Update CLI commands to use services

**Files:**
- Modify: All `crates/engai/src/cmd_*.rs` files

- [ ] **Step 1: Update each cmd file**

Replace direct `db.xxx()` calls with service calls. CLI commands should construct services the same way as the web server (or receive them as args).

- [ ] **Step 2: Verify compilation**

Run: `cargo check`

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "refactor: update CLI commands to use services"
```

---

### Task 6: Update TUI panels to use services

**Files:**
- Modify: `crates/engai/src/tui/panel_vocab.rs`
- Modify: `crates/engai/src/tui/panel_review.rs`
- Modify: `crates/engai/src/tui/panel_read.rs`
- Modify: `crates/engai/src/tui/panel_chat.rs`
- Modify: `crates/engai/src/tui/panel_stats.rs`
- Modify: `crates/engai/src/tui/mod.rs`

- [ ] **Step 1: Update each panel**

Replace direct `state.db.xxx()` and `state.ai_client.xxx()` calls with service calls.

- [ ] **Step 2: Verify compilation**

Run: `cargo check`

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "refactor: update TUI panels to use services"
```

---

### Task 8: Replace anyhow with structured AppError

**Files:**
- Modify: `crates/engai-core/src/lib.rs` — add error module
- Create: `crates/engai-core/src/error.rs`
- Modify: `crates/engai/src/error.rs` — align with core error type
- Modify: All service files — use `crate::error::Result<T>` instead of `anyhow::Result<T>`

- [ ] **Step 1: Create engai-core error type**

`crates/engai-core/src/error.rs`:
```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    NotFound(String),
    ValidationError(String),
    AiError(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Database(e) => write!(f, "Database error: {}", e),
            AppError::NotFound(s) => write!(f, "Not found: {}", s),
            AppError::ValidationError(s) => write!(f, "Validation error: {}", s),
            AppError::AiError(s) => write!(f, "AI error: {}", s),
            AppError::Internal(s) => write!(f, "Internal error: {}", s),
        }
    }
}

impl std::error::Error for AppError {}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::NotFound(s) => (StatusCode::NOT_FOUND, s.as_str()),
            AppError::ValidationError(s) => (StatusCode::BAD_REQUEST, s.as_str()),
            AppError::AiError(s) => (StatusCode::BAD_GATEWAY, s.as_str()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };
        (status, axum::Json(json!({ "error": message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
```

- [ ] **Step 2: Update lib.rs**

Add `pub mod error;` to `crates/engai-core/src/lib.rs`.

- [ ] **Step 3: Migrate services to use AppError**

Replace `anyhow::Result<T>` with `crate::error::Result<T>` in all service files. Replace `anyhow::anyhow!()` with appropriate `AppError` variants.

- [ ] **Step 4: Update engai binary's error.rs**

Align `crates/engai/src/error.rs` with the core error type, or re-export it.

- [ ] **Step 5: Verify compilation**

Run: `cargo check`

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: replace anyhow with structured AppError enum in services"
```

---

### Task 9: Final cleanup

**Files:**
- Modify: `crates/engai-core/src/lib.rs` — ensure clean exports
- Remove any dead code or unused imports

- [ ] **Step 1: Remove unused imports and dead code**

Run: `cargo clippy` and fix warnings.

- [ ] **Step 2: Full build verification**

Run: `cargo build`
Expected: success

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "chore: cleanup after 3-tier refactoring"
```
