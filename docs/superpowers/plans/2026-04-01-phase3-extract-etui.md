# Phase 3: Extract `etui` Crate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract TUI into standalone `crates/etui/` crate that communicates with the server via HTTP only.

**Architecture:** etui uses `ApiClient` (reqwest) for all backend communication. No direct DB or AI client access. Server handles all business logic.

**Tech Stack:** Rust, ratatui 0.30, crossterm 0.28, reqwest 0.12, tokio

---

### Task 1: Create etui crate skeleton

**Files:**
- Create: `crates/etui/Cargo.toml`
- Create: `crates/etui/src/lib.rs`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "etui"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.30"
crossterm = "0.28"
tokio = { workspace = true, features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
chrono = { workspace = true, features = ["serde"] }
anyhow = { workspace = true }
```

- [ ] **Step 2: Create lib.rs**

```rust
pub mod api;
pub mod panel;
pub mod sidebar;
pub mod tui;

pub async fn run_tui(server_url: &str) -> anyhow::Result<()> {
    let client = api::client::ApiClient::new(server_url.to_string());
    let mut terminal = tui::setup_terminal()?;
    let mut app = tui::app::App::new(client);
    tui::run_app(&mut terminal, &mut app).await?;
    tui::restore_terminal(&mut terminal)?;
    Ok(())
}
```

- [ ] **Step 3: Update workspace Cargo.toml**

Add `"crates/etui"` to workspace members.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: create etui crate skeleton"
```

---

### Task 2: Create API client and models

**Files:**
- Create: `crates/etui/src/api/mod.rs`
- Create: `crates/etui/src/api/client.rs`
- Create: `crates/etui/src/api/models.rs`

- [ ] **Step 1: Create api/models.rs**

Define API response types that mirror the backend responses:

```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct Phrase { /* same fields as backend */ }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reading { /* same fields as backend */ }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem {
    pub target_type: String,
    pub id: i64,
    pub display: String,
    pub meaning: String,
    pub familiarity: i32,
    pub interval: i32,
    pub ease_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsData {
    pub word_count: i64,
    pub phrase_count: i64,
    pub pending_reviews: i64,
    pub reviewed_today: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewStats {
    pub pending_reviews: i64,
    pub reviewed_today: i64,
}
```

- [ ] **Step 2: Create api/client.rs**

```rust
use anyhow::Result;
use crate::api::models::*;

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            base_url,
        }
    }

    pub async fn list_words(&self, search: Option<&str>, limit: i64, offset: i64) -> Result<Vec<Word>> {
        let mut url = format!("{}/api/words?limit={}&offset={}", self.base_url, limit, offset);
        if let Some(s) = search {
            url.push_str(&format!("&search={}", s));
        }
        let words = self.client.get(&url).send().await?.json().await?;
        Ok(words)
    }

    pub async fn get_word(&self, word: &str) -> Result<Word> {
        let url = format!("{}/api/words/{}", self.base_url, word);
        let word = self.client.get(&url).send().await?.json().await?;
        Ok(word)
    }

    pub async fn explain_word(&self, word: &str) -> Result<String> {
        let url = format!("{}/api/words/{}/explain", self.base_url, word);
        let resp = self.client.get(&url).send().await?.text().await?;
        Ok(resp)
    }

    pub async fn list_phrases(&self, search: Option<&str>, limit: i64, offset: i64) -> Result<Vec<Phrase>> { /* similar */ }
    pub async fn get_phrase(&self, id: i64) -> Result<Phrase> { /* similar */ }
    pub async fn explain_phrase(&self, id: i64) -> Result<String> { /* similar */ }

    pub async fn today_reviews(&self) -> Result<Vec<ReviewItem>> {
        let url = format!("{}/api/review/today", self.base_url);
        let items = self.client.get(&url).send().await?.json().await?;
        Ok(items)
    }

    pub async fn submit_review(&self, target_type: &str, id: i64, quality: i32) -> Result<()> {
        let url = format!("{}/api/review/{}/{}", self.base_url, target_type, id);
        self.client.post(&url).json(&serde_json::json!({"quality": quality})).send().await?;
        Ok(())
    }

    pub async fn review_stats(&self) -> Result<ReviewStats> { /* similar */ }

    pub async fn list_readings(&self, limit: i64, offset: i64) -> Result<Vec<Reading>> { /* similar */ }
    pub async fn get_reading(&self, id: i64) -> Result<Reading> { /* similar */ }
    pub async fn analyze_reading(&self, id: i64) -> Result<String> { /* similar */ }

    pub async fn chat(&self, message: &str) -> Result<String> {
        let url = format!("{}/api/chat", self.base_url);
        // Use non-streaming HTTP call for TUI chat
        let resp = self.client.post(&url).json(&serde_json::json!({"role": "user", "content": message})).send().await?.text().await?;
        Ok(resp)
    }

    pub async fn get_stats(&self) -> Result<StatsData> {
        let url = format!("{}/api/stats", self.base_url);
        let stats = self.client.get(&url).send().await?.json().await?;
        Ok(stats)
    }
}
```

- [ ] **Step 3: Create api/mod.rs**

```rust
pub mod client;
pub mod models;

pub use client::ApiClient;
pub use models::*;
```

- [ ] **Step 4: Verify etui compiles**

Run: `cargo check -p etui`

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add API client and models to etui"
```

---

### Task 3: Move TUI framework core

**Files:**
- Create: `crates/etui/src/tui/mod.rs`
- Create: `crates/etui/src/tui/app.rs`
- Create: `crates/etui/src/tui/event.rs`
- Create: `crates/etui/src/tui/focus.rs`
- Create: `crates/etui/src/tui/ui.rs`

- [ ] **Step 1: Create tui/event.rs**

Copy from `crates/engai/src/tui/event.rs`. Use crossterm event polling with 100ms timeout (same as current).

- [ ] **Step 2: Create tui/focus.rs** (new, from zbooks pattern)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    Content,
}

impl Default for Focus {
    fn default() -> Self {
        Focus::Sidebar
    }
}
```

- [ ] **Step 3: Create tui/app.rs**

Migrate from `crates/engai/src/tui/app.rs`. Replace `AppState` dependency with `ApiClient`:

```rust
use crate::api::ApiClient;
use crate::api::models::*;
use crate::tui::focus::Focus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Vocab,
    Review,
    Read,
    Chat,
    Stats,
}

impl Panel {
    pub fn label(&self) -> &str {
        match self {
            Panel::Vocab => "Vocabulary",
            Panel::Review => "Review",
            Panel::Read => "Reading",
            Panel::Chat => "Chat",
            Panel::Stats => "Stats",
        }
    }

    pub fn next(&self) -> Self { /* cycle */ }
    pub fn prev(&self) -> Self { /* cycle */ }
}

pub struct App {
    pub panel: Panel,
    pub focus: Focus,
    pub should_quit: bool,
    pub api: ApiClient,
    // Panel-specific state (same fields as current app.rs)
    pub words: Vec<Word>,
    pub phrases: Vec<Phrase>,
    pub vocab_tab: VocabTab,
    pub vocab_list_index: usize,
    pub vocab_detail: Option<VocabDetail>,
    pub review_items: Vec<ReviewItem>,
    pub review_index: usize,
    pub review_show_answer: bool,
    pub review_loading: bool,
    pub readings: Vec<Reading>,
    pub reading_list_index: usize,
    pub reading_detail: Option<ReadingDetail>,
    pub chat_messages: Vec<ChatMessage>,
    pub chat_input: String,
    pub chat_loading: bool,
    pub stats: Option<StatsData>,
    pub status_message: Option<(String, std::time::Instant)>,
}
```

- [ ] **Step 4: Create tui/ui.rs**

Copy the rendering logic from `crates/engai/src/tui/ui.rs` (557 lines). Adapt to use the new App struct with ApiClient.

- [ ] **Step 5: Create tui/mod.rs**

```rust
pub mod app;
pub mod event;
pub mod focus;
pub mod ui;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::tui::app::App;

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

pub async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    // Main event loop — same structure as current tui/mod.rs
    // but using ApiClient for data loading instead of AppState
    loop {
        terminal.draw(|f| ui::render(f, app))?;
        if let Some(event) = event::poll_event(std::time::Duration::from_millis(100))? {
            // Handle events, delegate to panels
        }
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add TUI framework core to etui"
```

---

### Task 4: Move panels to use ApiClient

**Files:**
- Create: `crates/etui/src/panel/mod.rs`
- Create: `crates/etui/src/panel/vocab.rs`
- Create: `crates/etui/src/panel/review.rs`
- Create: `crates/etui/src/panel/read.rs`
- Create: `crates/etui/src/panel/chat.rs`
- Create: `crates/etui/src/panel/stats.rs`

- [ ] **Step 1: Create each panel**

For each panel, adapt from the current `crates/engai/src/tui/panel_*.rs`:

- `vocab.rs`: Replace `state.db.list_words()` → `app.api.list_words().await`, `state.ai_client.explain_word()` → `app.api.explain_word().await`
- `review.rs`: Replace `state.db.get_today_review_words()` → `app.api.today_reviews().await`, SM-2 is now server-side, just call `app.api.submit_review()`
- `read.rs`: Replace `state.db.list_readings()` → `app.api.list_readings().await`
- `chat.rs`: Replace `state.ai_client.chat_completion()` → `app.api.chat().await`
- `stats.rs`: Replace `state.db.word_count()` etc → `app.api.get_stats().await`

- [ ] **Step 2: Create panel/mod.rs**

```rust
pub mod chat;
pub mod read;
pub mod review;
pub mod stats;
pub mod vocab;
```

- [ ] **Step 3: Verify etui compiles**

Run: `cargo check -p etui`

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add panels to etui using ApiClient"
```

---

### Task 5: Create sidebar module

**Files:**
- Create: `crates/etui/src/sidebar/mod.rs`
- Create: `crates/etui/src/sidebar/sidebar.rs`
- Create: `crates/etui/src/sidebar/navigator.rs`

- [ ] **Step 1: Create sidebar components**

Follow zbooks' sidebar pattern with expand/collapse navigation:

`sidebar/sidebar.rs` — Sidebar widget rendering panel list
`sidebar/navigator.rs` — Navigation logic (up/down, select, expand/collapse)
`sidebar/mod.rs` — Re-exports

- [ ] **Step 2: Integrate sidebar into ui.rs**

Update `tui/ui.rs` to render the sidebar alongside the main content area.

- [ ] **Step 3: Verify etui compiles**

Run: `cargo check -p etui`

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add sidebar navigation to etui"
```

---

### Task 6: Update main binary to use etui crate

**Files:**
- Modify: `crates/engai/src/main.rs` — add TUI subcommand
- Modify: `crates/engai/src/Cargo.toml` — add etui dependency
- Delete: `crates/engai/src/tui/` directory

- [ ] **Step 1: Add etui dependency**

In `crates/engai/Cargo.toml`:
```toml
[dependencies]
etui = { path = "../etui" }
```

- [ ] **Step 2: Update main.rs for new run modes**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "engai")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start TUI connecting to a remote server
    Tui {
        /// Server URL
        #[arg(short, long, default_value = "http://127.0.0.1:9000")]
        server: String,
    },
    /// Start HTTP server only
    Svr {
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    // ... existing CLI commands (add, explain, review, etc.)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Tui { server }) => {
            etui::run_tui(&server).await?;
        }
        Some(Commands::Svr { port, host }) => {
            let state = setup_state().await?;
            server::run_server(state, port).await?;
        }
        None => {
            // Integrated mode: spawn server + TUI
            let state = setup_state().await?;
            let port = state.config.server.port;
            let server_handle = tokio::spawn(async move {
                server::run_server(state, port).await
            });
            etui::run_tui(&format!("http://127.0.0.1:{}", port)).await?;
            server_handle.abort();
        }
        // ... handle other CLI commands
    }
    Ok(())
}
```

- [ ] **Step 3: Delete old tui/ directory**

Remove `crates/engai/src/tui/` entirely.

- [ ] **Step 4: Verify full workspace compiles**

Run: `cargo check`

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor: replace inline TUI with etui crate, add tui/svr run modes"
```
