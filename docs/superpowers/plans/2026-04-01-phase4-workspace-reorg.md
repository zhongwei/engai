# Phase 4: Workspace Reorganization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reorganize workspace to apps/engai + crates/ layout matching zbooks, add build.rs for frontend embedding, rename frontend/ to web/.

**Architecture:** Single binary crate at apps/engai/ containing all backend code. Separate etui and esync crates in crates/. Frontend at web/ builds into apps/engai/static/.

**Tech Stack:** Rust, rust-embed, Vite, npm

---

### Task 1: Create apps/engai directory and move files

**Files:**
- Create: `apps/engai/` directory structure
- Move: All files from `crates/engai-core/src/` and `crates/engai/src/` into `apps/engai/src/`

- [ ] **Step 1: Create directory structure**

```bash
mkdir -p apps/engai/src
mkdir -p apps/engai/migrations
```

- [ ] **Step 2: Move engai-core source files to apps/engai/src/**

Move all module directories and files:
```bash
mv crates/engai-core/src/models apps/engai/src/models
mv crates/engai-core/src/db apps/engai/src/db
mv crates/engai-core/src/services apps/engai/src/services
mv crates/engai-core/src/config.rs apps/engai/src/config.rs
mv crates/engai-core/src/review.rs apps/engai/src/review.rs
mv crates/engai-core/src/lib.rs apps/engai/src/_lib_old.rs  # Will be deleted; binary doesn't need lib.rs
```

Move AI and prompt files:
```bash
mv crates/engai-core/src/ai.rs apps/engai/src/ai.rs
mv crates/engai-core/src/prompt.rs apps/engai/src/prompt.rs
```

- [ ] **Step 3: Move engai binary source files to apps/engai/src/**

```bash
mv crates/engai/src/main.rs apps/engai/src/main.rs
mv crates/engai/src/server.rs apps/engai/src/svr.rs
mv crates/engai/src/state.rs apps/engai/src/state.rs
mv crates/engai/src/error.rs apps/engai/src/error.rs
mv crates/engai/src/routes apps/engai/src/handlers
```

Move CLI commands:
```bash
mv crates/engai/src/cmd_*.rs apps/engai/src/cli/
```
(First create `apps/engai/src/cli/` and move each cmd_*.rs into it with appropriate mod.rs.)

- [ ] **Step 4: Move migrations**

```bash
mv crates/engai-core/migrations/001_init.sql apps/engai/migrations/
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor: move source files to apps/engai directory"
```

---

### Task 2: Create merged Cargo.toml for apps/engai

**Files:**
- Create: `apps/engai/Cargo.toml`

- [ ] **Step 1: Merge dependencies**

Combine all dependencies from old `engai-core/Cargo.toml` and `engai/Cargo.toml`:

```toml
[package]
name = "engai"
version = "0.1.0"
edition = "2021"

[dependencies]
# Workspace deps
tokio = { workspace = true, features = ["full"] }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true, features = ["serde"] }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "migrate"] }

# Web
axum = { version = "0.8", features = ["ws"] }
axum-extra = "0.10"
tower-http = { version = "0.6", features = ["cors"] }
tower = "0.5"
rust-embed = "8"

# CLI
clap = { version = "4", features = ["derive"] }

# AI
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio-stream = "0.1"
futures = "0.3"
async-stream = "0.3"
uuid = { version = "1", features = ["v4"] }

# Markdown / Sync
gray_matter = "0.2"
pulldown-cmark = "0.11"
toml = "0.8"
dirs = "5"

# Local crates
etui = { path = "../../crates/etui" }
esync = { path = "../../crates/esync" }
```

- [ ] **Step 2: Commit**

```bash
git add -A
git commit -m "feat: create merged Cargo.toml for apps/engai"
```

---

### Task 3: Update imports — remove engai_core:: prefix

**Files:**
- Modify: All files in `apps/engai/src/`

- [ ] **Step 1: Replace all `engai_core::` references**

Since everything is now in one crate, replace:
- `use engai_core::models::*` → `use crate::models::*`
- `use engai_core::db::Db` → `use crate::db::Db`
- `use engai_core::config::Config` → `use crate::config::Config`
- `use engai_core::ai::AiClient` → `use crate::ai::AiClient`
- `use engai_core::services::*` → `use crate::services::*`
- `use engai_core::review::calculate_next_review` → `use crate::review::calculate_next_review`

- [ ] **Step 2: Update internal module references**

Rename `routes` module to `handlers` in all relevant files:
- `use crate::routes::*` → `use crate::handlers::*`
- Rename `cmd_*` references to `crate::cli::*`

- [ ] **Step 3: Create apps/engai/src/main.rs**

```rust
mod ai;
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod prompt;
mod review;
mod services;
mod state;
mod svr;
mod cli;

use clap::{Parser, Subcommand};
use cli::*;

#[derive(Parser)]
#[command(name = "engai")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Tui { #[arg(short, long, default_value = "http://127.0.0.1:9000")] server: String },
    Svr { #[arg(short, long, default_value_t = 3000)] port: u16, #[arg(long, default_value = "127.0.0.1")] host: String },
    Add { word_or_phrase: String, meaning: String, phonetic: Option<String> },
    Explain { target: String, r#type: Option<String> },
    Review,
    Sync,
    Read { file: String, analyze: bool },
    Import { dir: String },
    Export { dir: Option<String> },
    Stats,
    Config { key: Option<String>, value: Option<String>, init: bool },
    Note { target_type: String, target_id: i64, content: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Same dispatch logic as current main.rs but using new module structure
    // ...
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check`

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor: update imports to remove engai_core:: prefix"
```

---

### Task 4: Add build.rs for frontend embedding

**Files:**
- Create: `apps/engai/build.rs`
- Modify: `apps/engai/src/svr.rs` — update rust-embed path

- [ ] **Step 1: Create build.rs**

```rust
fn main() {
    if std::env::var("FRONTEND_BUILD").is_ok() || cfg!(not(debug_assertions)) {
        let web_dir = std::path::Path::new("../../web");
        if web_dir.exists() {
            let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };

            let status = std::process::Command::new(npm)
                .args(["install"])
                .current_dir(web_dir)
                .status()
                .expect("Failed to run npm install");
            assert!(status.success(), "npm install failed");

            let status = std::process::Command::new(npm)
                .args(["run", "build"])
                .current_dir(web_dir)
                .status()
                .expect("Failed to run npm build");
            assert!(status.success(), "npm build failed");
        }
    }

    println!("cargo:rerun-if-changed=static/");
}
```

- [ ] **Step 2: Update svr.rs rust-embed path**

```rust
#[derive(rust_embed::RustEmbed)]
#[folder = "static/"]
struct Assets;
```

Change from `../../frontend/dist` to `static/`.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: add build.rs for frontend embedding"
```

---

### Task 5: Rename frontend/ to web/ and update vite config

**Files:**
- Rename: `frontend/` → `web/`
- Modify: `web/vite.config.ts`

- [ ] **Step 1: Rename directory**

```bash
mv frontend web
```

- [ ] **Step 2: Update vite.config.ts output path**

```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import path from 'path'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    outDir: '../apps/engai/static',
    emptyOutDir: true,
  },
  server: {
    proxy: {
      '/api': 'http://localhost:9000',
    },
  },
})
```

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "refactor: rename frontend/ to web/, update vite output path"
```

---

### Task 6: Update workspace Cargo.toml and cleanup

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Delete: `crates/engai-core/` directory
- Delete: `crates/engai/` directory

- [ ] **Step 1: Update workspace Cargo.toml**

```toml
[workspace]
resolver = "2"
members = [
    "apps/engai",
    "crates/etui",
    "crates/esync",
]

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

- [ ] **Step 2: Remove old crate directories**

```bash
rm -rf crates/engai-core
rm -rf crates/engai
```

- [ ] **Step 3: Full build verification**

Run: `cargo build`
Expected: success

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "refactor: finalize workspace reorganization to apps/ + crates/ layout"
```
