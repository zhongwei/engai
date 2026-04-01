# Phase 2: Extract `esync` Crate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract Markdown sync engine and parser into a standalone crate `crates/esync/`.

**Architecture:** esync defines a `SyncDb` trait for database access, keeping it fully independent. The main app implements this trait against its repositories.

**Tech Stack:** Rust, sqlx 0.8, gray_matter, pulldown-cmark, tokio

---

### Task 1: Create esync crate skeleton

**Files:**
- Create: `crates/esync/Cargo.toml`
- Create: `crates/esync/src/lib.rs`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "esync"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlx = { workspace = true, features = ["runtime-tokio", "sqlite"] }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
chrono = { workspace = true, features = ["serde"] }
tokio = { workspace = true, features = ["fs"] }
anyhow = { workspace = true }
gray_matter = "0.2"
pulldown-cmark = "0.11"
```

- [ ] **Step 2: Create lib.rs**

```rust
pub mod markdown;
pub mod models;
pub mod sync;
```

- [ ] **Step 3: Update workspace Cargo.toml**

Add `"crates/esync"` to workspace members.

- [ ] **Step 4: Verify skeleton compiles**

Run: `cargo check -p esync`
Expected: FAIL (missing modules) — this is expected, we'll fill them in.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: create esync crate skeleton"
```

---

### Task 2: Move markdown types to esync

**Files:**
- Create: `crates/esync/src/models.rs`
- Move types from `engai-core/src/markdown.rs`: MarkdownWord, MarkdownPhrase, MarkdownReading

- [ ] **Step 1: Create models.rs**

Copy `MarkdownWord`, `MarkdownPhrase`, `MarkdownReading` structs and their helper types from `engai-core/src/markdown.rs` into `crates/esync/src/models.rs`. Adjust imports to use `serde`, `chrono`, etc. directly instead of crate-local paths.

- [ ] **Step 2: Verify esync compiles**

Run: `cargo check -p esync`

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: add markdown types to esync models"
```

---

### Task 3: Move markdown parser/generator to esync

**Files:**
- Create: `crates/esync/src/markdown.rs`

- [ ] **Step 1: Move markdown.rs**

Copy the entire `markdown.rs` from engai-core to `crates/esync/src/markdown.rs`. Update:
- Import types from `crate::models` instead of `crate::models`
- Keep all parse/generate/save methods

- [ ] **Step 2: Verify esync compiles**

Run: `cargo check -p esync`

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: move markdown parser/generator to esync"
```

---

### Task 4: Define SyncDb trait and move sync engine

**Files:**
- Create: `crates/esync/src/sync.rs`

- [ ] **Step 1: Define SyncDb trait**

```rust
use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait SyncDb: Send + Sync {
    async fn get_all_words(&self) -> Result<Vec<WordSummary>>;
    async fn get_all_phrases(&self) -> Result<Vec<PhraseSummary>>;
    async fn get_word_by_word(&self, word: &str) -> Result<Option<WordDetail>>;
    async fn get_phrase_by_phrase(&self, phrase: &str) -> Result<Option<PhraseDetail>>;
    async fn upsert_word(&self, word: &WordDetail) -> Result<()>;
    async fn upsert_phrase(&self, phrase: &PhraseDetail) -> Result<()>;
    async fn get_word_updated_at(&self, word: &str) -> Result<Option<chrono::NaiveDateTime>>;
    async fn get_phrase_updated_at(&self, phrase: &str) -> Result<Option<chrono::NaiveDateTime>>;
}

pub struct WordSummary {
    pub word: String,
    pub phonetic: Option<String>,
    pub meaning: String,
    pub familiarity: i32,
    pub interval: i32,
    pub next_review: Option<chrono::NaiveDateTime>,
    pub updated_at: chrono::NaiveDateTime,
}

pub struct PhraseSummary {
    pub phrase: String,
    pub meaning: String,
    pub familiarity: i32,
    pub interval: i32,
    pub next_review: Option<chrono::NaiveDateTime>,
    pub updated_at: chrono::NaiveDateTime,
}

pub struct WordDetail {
    pub id: i64,
    pub word: String,
    pub phonetic: Option<String>,
    pub meaning: String,
    pub familiarity: i32,
    pub interval: i32,
    pub ease_factor: f64,
    pub next_review: Option<chrono::NaiveDateTime>,
    pub updated_at: chrono::NaiveDateTime,
}

pub struct PhraseDetail {
    pub id: i64,
    pub phrase: String,
    pub meaning: String,
    pub familiarity: i32,
    pub interval: i32,
    pub ease_factor: f64,
    pub next_review: Option<chrono::NaiveDateTime>,
    pub updated_at: chrono::NaiveDateTime,
}

pub struct PhraseDetail {
    pub id: i64,
    pub phrase: String,
    pub meaning: String,
    pub familiarity: i32,
    pub interval: i32,
    pub ease_factor: f64,
    pub next_review: Option<chrono::NaiveDateTime>,
    pub updated_at: chrono::NaiveDateTime,
}

impl<T: SyncDb> SyncEngine<T> {
    pub fn new(db: T, docs_path: PathBuf) -> Self {
        Self { db, docs_path }
    }

    pub async fn sync_all(&self) -> anyhow::Result<()> {
        self.sync_words().await?;
        self.sync_phrases().await?;
        Ok(())
    }

    pub async fn sync_words(&self) -> anyhow::Result<()> {
        // Same logic as original but using self.db.get_all_words(), self.db.upsert_word(), etc.
    }

    pub async fn sync_phrases(&self) -> anyhow::Result<()> {
        // Same logic as original but using self.db.get_all_phrases(), self.db.upsert_phrase(), etc.
    }
}
```

- [ ] **Step 3: Verify esync compiles**

Run: `cargo check -p esync`

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add SyncDb trait and sync engine to esync"
```

---

### Task 5: Implement SyncDb for engai's repositories

**Files:**
- Create: `crates/engai-core/src/services/sync_adapter.rs`
- Modify: `crates/engai-core/src/services/mod.rs`
- Modify: `crates/engai-core/Cargo.toml` — add esync dependency

- [ ] **Step 1: Add esync dependency to engai-core**

```toml
[dependencies]
esync = { path = "../esync" }
```

- [ ] **Step 2: Implement SyncDb trait**

```rust
use esync::sync::{SyncDb, WordSummary, PhraseSummary, WordDetail, PhraseDetail};
use crate::db::{WordRepository, PhraseRepository};
use sqlx::SqlitePool;

pub struct SyncDbImpl {
    pool: SqlitePool,
}

impl SyncDbImpl {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SyncDb for SyncDbImpl {
    // Implement all trait methods using sqlx queries against the pool
    async fn get_all_words(&self) -> anyhow::Result<Vec<WordSummary>> { ... }
    async fn upsert_word(&self, word: &WordDetail) -> anyhow::Result<()> { ... }
    // etc.
}
```

- [ ] **Step 3: Update sync_service.rs**

```rust
use esync::sync::SyncEngine;
use crate::services::sync_adapter::SyncDbImpl;

pub struct SyncService {
    engine: SyncEngine<SyncDbImpl>,
}

impl SyncService {
    pub fn new(pool: SqlitePool, docs_path: std::path::PathBuf) -> Self {
        let db = SyncDbImpl::new(pool);
        let engine = SyncEngine::new(db, docs_path);
        Self { engine }
    }

    pub async fn sync_all(&self) -> anyhow::Result<()> {
        self.engine.sync_all().await
    }
}
```

- [ ] **Step 4: Verify full workspace compiles**

Run: `cargo check`

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: implement SyncDb adapter and wire up esync in sync service"
```

---

### Task 6: Remove old files from engai-core

**Files:**
- Delete: `crates/engai-core/src/markdown.rs`
- Delete: `crates/engai-core/src/sync.rs`
- Modify: `crates/engai-core/src/lib.rs`

- [ ] **Step 1: Remove markdown.rs and sync.rs from engai-core**

Delete both files. Update `lib.rs`:

```rust
pub mod ai;
pub mod config;
pub mod db;
pub mod models;
pub mod prompt;
pub mod review;
pub mod services;
```

- [ ] **Step 2: Update any remaining references**

Search for `crate::markdown` and `crate::sync` in engai-core. CLI commands that used `markdown::MarkdownWord` should now use `esync::models::MarkdownWord`. Update imports.

- [ ] **Step 3: Verify compilation**

Run: `cargo check`

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "refactor: remove markdown.rs and sync.rs from engai-core, fully migrated to esync"
```
