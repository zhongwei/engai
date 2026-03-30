# Engai Phase 2: Web Server + React Frontend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a full-featured web interface for Engai at localhost:3000 — Axum API server with all REST routes, WebSocket chat, rust-embed static serving, and a React SPA with 6 pages (Dashboard, Vocabulary, WordCard, Review, Reading, Chat).

**Architecture:** Axum web server runs inside the existing `engai` binary, sharing `engai-core` for all business logic. The server uses `Arc<Db>` as shared state via Axum's State extractor. React frontend is a Vite SPA embedded into the binary via `rust-embed`. AI streaming uses SSE (explain/analyze) and WebSocket (chat).

**Tech Stack:** Rust (axum 0.8.8, tokio 1.50.0, tower-http, rust-embed 8.11.0, axum-extra for WebSocket), React 19, Vite 7, Shadcn/ui, Tailwind CSS 4, React Router 7, TanStack Query 5, Recharts 2, lucide-react

---

## File Structure

```
engai/
├── crates/
│   ├── engai-core/
│   │   └── src/
│   │       └── (unchanged — existing modules)
│   └── engai/
│       ├── Cargo.toml                    # ADD: axum, tower-http, rust-embed, axum-extra
│       ├── build.rs                      # UPDATE: real frontend build + embed
│       └── src/
│           ├── main.rs                   # MODIFY: wire up server subcommand
│           ├── server.rs                 # CREATE: Axum app setup, static serving
│           ├── routes/
│           │   ├── mod.rs                # CREATE: route module declarations
│           │   ├── words.rs              # CREATE: word CRUD + explain SSE
│           │   ├── phrases.rs            # CREATE: phrase CRUD + explain SSE
│           │   ├── reviews.rs            # CREATE: review queue + submit
│           │   ├── readings.rs           # CREATE: reading CRUD + analyze SSE
│           │   ├── notes.rs              # CREATE: notes CRUD
│           │   ├── chat.rs               # CREATE: WebSocket chat endpoint
│           │   ├── sync.rs               # CREATE: trigger sync
│           │   └── stats.rs              # CREATE: dashboard stats
│           ├── state.rs                  # CREATE: AppState (Arc<Db>, Config, AiClient)
│           └── error.rs                  # CREATE: API error types
├── frontend/
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── tsconfig.app.json
│   ├── tsconfig.node.json
│   ├── index.html
│   ├── components.json                   # Shadcn config
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── index.css                     # Tailwind imports
│   │   ├── lib/
│   │   │   ├── api.ts                    # fetch wrapper + SSE + WS helpers
│   │   │   └── utils.ts                  # cn() utility for Shadcn
│   │   ├── hooks/
│   │   │   └── useWebSocket.ts           # WebSocket connection hook
│   │   ├── components/
│   │   │   ├── ui/                       # Shadcn base components (auto-generated)
│   │   │   ├── Layout.tsx                # sidebar + main content shell
│   │   │   ├── FlashCard.tsx             # flip card for review
│   │   │   ├── FamiliarityBadge.tsx      # colored badge 0-5
│   │   │   └── MarkdownRender.tsx        # render markdown content
│   │   └── pages/
│   │       ├── Dashboard.tsx             # stats, charts, overview
│   │       ├── Vocabulary.tsx            # word/phrase list with search/filter
│   │       ├── WordCard.tsx              # word detail with AI explain
│   │       ├── Review.tsx                # Anki-style review session
│   │       ├── Reading.tsx               # reading list + detail
│   │       └── Chat.tsx                  # AI English conversation
│   └── dist/                             # built output (embedded into binary)
└── (existing files unchanged)
```

---

### Task 1: Rust Backend Dependencies + AppState + Error Types

**Files:**
- Modify: `crates/engai/Cargo.toml`
- Create: `crates/engai/src/state.rs`
- Create: `crates/engai/src/error.rs`

- [ ] **Step 1: Add backend dependencies to engai Cargo.toml**

Add to `crates/engai/Cargo.toml`:
```toml
axum = { version = "0.8.8", features = ["ws"] }
axum-extra = { version = "0.10", features = ["typed-header"] }
tower-http = { version = "0.6", features = ["cors", "fs"] }
tower = "0.5"
rust-embed = "8.11.0"
mime_guess = "2"
futures = "0.3"
async-stream = "0.3"
uuid = { version = "1", features = ["v4"] }
```

- [ ] **Step 2: Create AppState**

`crates/engai/src/state.rs`:
```rust
use std::sync::Arc;
use engai_core::ai::AiClient;
use engai_core::config::Config;
use engai_core::db::Db;
use engai_core::prompt::PromptEngine;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub config: Config,
    pub ai_client: Arc<AiClient>,
    pub prompt_engine: Arc<PromptEngine>,
}

impl AppState {
    pub fn new(db: Arc<Db>, config: Config) -> anyhow::Result<Self> {
        let ai_client = Arc::new(AiClient::from_config(&config)?);
        let prompt_engine = Arc::new(PromptEngine::new(config.prompts_path()));
        Ok(Self {
            db,
            config,
            ai_client,
            prompt_engine,
        })
    }
}
```

- [ ] **Step 3: Create error types**

`crates/engai/src/error.rs`:
```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn not_found(msg: &str) -> Self {
        Self::new(StatusCode::NOT_FOUND, msg)
    }

    pub fn bad_request(msg: &str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, msg)
    }

    pub fn internal(msg: &str) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, msg)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::internal(err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = json!({ "error": self.message });
        (self.status, axum::Json(body)).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p engai`
Expected: Compiles (no route modules yet — just types)

- [ ] **Step 5: Commit**

```bash
git add crates/engai/Cargo.toml crates/engai/src/state.rs crates/engai/src/error.rs
git commit -m "feat(backend): add axum deps, AppState, and ApiError types"
```

---

### Task 2: Word Routes (CRUD + AI Explain SSE)

**Files:**
- Create: `crates/engai/src/routes/mod.rs`
- Create: `crates/engai/src/routes/words.rs`

- [ ] **Step 1: Create routes module**

`crates/engai/src/routes/mod.rs`:
```rust
pub mod chat;
pub mod notes;
pub mod phrases;
pub mod readings;
pub mod reviews;
pub mod stats;
pub mod sync;
pub mod words;
```

- [ ] **Step 2: Implement word routes**

`crates/engai/src/routes/words.rs`:
```rust
use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{delete, get, post, put},
    Json, Router,
};
use futures::stream::Stream;
use serde::Deserialize;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_words).post(create_word))
        .route("/{word}", get(get_word).put(update_word).delete(delete_word))
        .route("/{word}/explain", get(explain_word))
        .route("/{word}/examples", get(get_examples))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub familiarity_gte: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

async fn list_words(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let words = state
        .db
        .list_words(
            q.search.as_deref(),
            q.familiarity_gte,
            q.limit.unwrap_or(50),
            q.offset.unwrap_or(0),
        )
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "words": words })))
}

async fn get_word(
    State(state): State<AppState>,
    Path(word): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let word = state
        .db
        .get_word(&word)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Word not found"))?;
    Ok(Json(serde_json::to_value(word).unwrap()))
}

#[derive(Debug, Deserialize)]
pub struct CreateWord {
    pub word: String,
    pub phonetic: Option<String>,
    pub meaning: Option<String>,
}

async fn create_word(
    State(state): State<AppState>,
    Json(body): Json<CreateWord>,
) -> ApiResult<Json<serde_json::Value>> {
    let word = state
        .db
        .add_word(&body.word, body.phonetic.as_deref(), body.meaning.as_deref())
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::to_value(word).unwrap()))
}

#[derive(Debug, Deserialize)]
pub struct UpdateWordBody {
    pub phonetic: Option<String>,
    pub meaning: Option<String>,
    pub familiarity: Option<i32>,
    pub next_review: Option<String>,
    pub interval: Option<i32>,
    pub ease_factor: Option<f64>,
}

async fn update_word(
    State(state): State<AppState>,
    Path(word): Path<String>,
    Json(body): Json<UpdateWordBody>,
) -> ApiResult<Json<serde_json::Value>> {
    let existing = state
        .db
        .get_word(&word)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Word not found"))?;

    let next_review = body
        .next_review
        .as_deref()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok());

    let updated = state
        .db
        .update_word(
            existing.id,
            None,
            body.phonetic.as_deref(),
            body.meaning.as_deref(),
            body.familiarity,
            next_review,
            body.interval,
            body.ease_factor,
        )
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::internal("Update failed"))?;
    Ok(Json(serde_json::to_value(updated).unwrap()))
}

async fn delete_word(
    State(state): State<AppState>,
    Path(word): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let existing = state
        .db
        .get_word(&word)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Word not found"))?;

    let deleted = state
        .db
        .delete_word(existing.id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "deleted": deleted })))
}

async fn get_examples(
    State(state): State<AppState>,
    Path(word): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let existing = state
        .db
        .get_word(&word)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Word not found"))?;

    let examples = state
        .db
        .get_examples("word", existing.id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "examples": examples })))
}

async fn explain_word(
    State(state): State<AppState>,
    Path(word): Path<String>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, axum::Error>>>> {
    let ai = state.ai_client.clone();
    let prompt = state.prompt_engine.clone();
    let word_clone = word.clone();

    let stream = async_stream::stream! {
        let explanation = match ai.explain_word(&word_clone, &prompt).await {
            Ok(text) => text,
            Err(e) => {
                yield Err(axum::Error::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())));
                return;
            }
        };

        let mut char_indices = explanation.char_indices().peekable();
        let mut buf = String::new();

        while let Some((_, ch)) = char_indices.next() {
            buf.push(ch);

            if buf.len() >= 3 || char_indices.peek().is_none() {
                let chunk = buf.clone();
                buf.clear();
                yield Ok(Event::default().data(chunk));
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }

        yield Ok(Event::default().data("[DONE]"));
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
```

- [ ] **Step 3: Add `mod routes;` and `mod state;` and `mod error;` to main.rs**

Add at the top of `crates/engai/src/main.rs`:
```rust
mod error;
mod routes;
mod state;
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p engai`
Expected: Compiles

- [ ] **Step 5: Commit**

```bash
git add crates/engai/src/routes/ crates/engai/src/state.rs crates/engai/src/error.rs crates/engai/src/main.rs
git commit -m "feat(backend): add word CRUD routes and AI explain SSE endpoint"
```

---

### Task 3: Phrase Routes (CRUD + AI Explain SSE)

**Files:**
- Create: `crates/engai/src/routes/phrases.rs`

- [ ] **Step 1: Implement phrase routes**

`crates/engai/src/routes/phrases.rs`:
```rust
use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{delete, get, post, put},
    Json, Router,
};
use futures::stream::Stream;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_phrases).post(create_phrase))
        .route("/{id}", get(get_phrase).put(update_phrase).delete(delete_phrase))
        .route("/{id}/explain", get(explain_phrase))
        .route("/{id}/examples", get(get_examples))
}

#[derive(Debug, serde::Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub familiarity_gte: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

async fn list_phrases(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let phrases = state
        .db
        .list_phrases(
            q.search.as_deref(),
            q.familiarity_gte,
            q.limit.unwrap_or(50),
            q.offset.unwrap_or(0),
        )
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "phrases": phrases })))
}

async fn get_phrase(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    let phrase = state
        .db
        .get_phrase_by_id(id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Phrase not found"))?;
    Ok(Json(serde_json::to_value(phrase).unwrap()))
}

#[derive(Debug, serde::Deserialize)]
pub struct CreatePhrase {
    pub phrase: String,
    pub meaning: Option<String>,
}

async fn create_phrase(
    State(state): State<AppState>,
    Json(body): Json<CreatePhrase>,
) -> ApiResult<Json<serde_json::Value>> {
    let phrase = state
        .db
        .add_phrase(&body.phrase, body.meaning.as_deref())
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::to_value(phrase).unwrap()))
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdatePhraseBody {
    pub phrase: Option<String>,
    pub meaning: Option<String>,
    pub familiarity: Option<i32>,
    pub next_review: Option<String>,
    pub interval: Option<i32>,
    pub ease_factor: Option<f64>,
}

async fn update_phrase(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdatePhraseBody>,
) -> ApiResult<Json<serde_json::Value>> {
    let next_review = body
        .next_review
        .as_deref()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok());

    let updated = state
        .db
        .update_phrase(
            id,
            body.phrase.as_deref(),
            body.meaning.as_deref(),
            body.familiarity,
            next_review,
            body.interval,
            body.ease_factor,
        )
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Phrase not found"))?;
    Ok(Json(serde_json::to_value(updated).unwrap()))
}

async fn delete_phrase(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    let deleted = state
        .db
        .delete_phrase(id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "deleted": deleted })))
}

async fn get_examples(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    let examples = state
        .db
        .get_examples("phrase", id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "examples": examples })))
}

async fn explain_phrase(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, axum::Error>>>> {
    let phrase = state
        .db
        .get_phrase_by_id(id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Phrase not found"))?;

    let ai = state.ai_client.clone();
    let prompt = state.prompt_engine.clone();
    let phrase_text = phrase.phrase.clone();

    let stream = async_stream::stream! {
        let explanation = match ai.explain_phrase(&phrase_text, &prompt).await {
            Ok(text) => text,
            Err(e) => {
                yield Err(axum::Error::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())));
                return;
            }
        };

        let mut char_indices = explanation.char_indices().peekable();
        let mut buf = String::new();

        while let Some((_, ch)) = char_indices.next() {
            buf.push(ch);
            if buf.len() >= 3 || char_indices.peek().is_none() {
                let chunk = buf.clone();
                buf.clear();
                yield Ok(Event::default().data(chunk));
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
        yield Ok(Event::default().data("[DONE]"));
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p engai`

- [ ] **Step 3: Commit**

```bash
git add crates/engai/src/routes/phrases.rs
git commit -m "feat(backend): add phrase CRUD routes and AI explain SSE endpoint"
```

---

### Task 4: Review, Reading, Notes, Sync, Stats Routes

**Files:**
- Create: `crates/engai/src/routes/reviews.rs`
- Create: `crates/engai/src/routes/readings.rs`
- Create: `crates/engai/src/routes/notes.rs`
- Create: `crates/engai/src/routes/sync.rs`
- Create: `crates/engai/src/routes/stats.rs`

- [ ] **Step 1: Implement review routes**

`crates/engai/src/routes/reviews.rs`:
```rust
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use engai_core::review::calculate_next_review;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/today", get(today_queue))
        .route("/stats", get(review_stats))
        .route("/{target_type}/{id}", post(submit_review))
}

#[derive(Debug, serde::Serialize)]
struct ReviewItem {
    target_type: String,
    id: i64,
    display: String,
    familiarity: i32,
    interval: i32,
    ease_factor: f64,
}

async fn today_queue(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    let words = state
        .db
        .get_today_review_words()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    let phrases = state
        .db
        .get_today_review_phrases()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;

    let mut items: Vec<ReviewItem> = words
        .into_iter()
        .map(|w| ReviewItem {
            target_type: "word".to_string(),
            id: w.id,
            display: w.word,
            familiarity: w.familiarity,
            interval: w.interval,
            ease_factor: w.ease_factor,
        })
        .collect();
    items.extend(phrases.into_iter().map(|p| ReviewItem {
        target_type: "phrase".to_string(),
        id: p.id,
        display: p.phrase,
        familiarity: p.familiarity,
        interval: p.interval,
        ease_factor: p.ease_factor,
    }));

    Ok(Json(serde_json::json!({ "items": items })))
}

#[derive(Debug, Deserialize)]
struct SubmitReview {
    quality: i32,
}

async fn submit_review(
    State(state): State<AppState>,
    Path((target_type, id)): Path<(String, i64)>,
    Json(body): Json<SubmitReview>,
) -> ApiResult<Json<serde_json::Value>> {
    let quality = body.quality.clamp(0, 5);

    let (interval, ease_factor) = match target_type.as_str() {
        "word" => {
            let word = state
                .db
                .get_word_by_id(id)
                .await
                .map_err(|e| ApiError::internal(&e.to_string()))?
                .ok_or_else(|| ApiError::not_found("Word not found"))?;
            (word.interval, word.ease_factor)
        }
        "phrase" => {
            let phrase = state
                .db
                .get_phrase_by_id(id)
                .await
                .map_err(|e| ApiError::internal(&e.to_string()))?
                .ok_or_else(|| ApiError::not_found("Phrase not found"))?;
            (phrase.interval, phrase.ease_factor)
        }
        _ => return Err(ApiError::bad_request("Invalid target_type")),
    };

    let result = calculate_next_review(quality, interval, ease_factor);

    state
        .db
        .add_review(&target_type, id, quality)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;

    match target_type.as_str() {
        "word" => {
            state
                .db
                .update_word(
                    id,
                    None,
                    None,
                    None,
                    Some(result.familiarity),
                    Some(result.next_review),
                    Some(result.interval),
                    Some(result.ease_factor),
                )
                .await
                .map_err(|e| ApiError::internal(&e.to_string()))?;
        }
        "phrase" => {
            state
                .db
                .update_phrase(
                    id,
                    None,
                    None,
                    Some(result.familiarity),
                    Some(result.next_review),
                    Some(result.interval),
                    Some(result.ease_factor),
                )
                .await
                .map_err(|e| ApiError::internal(&e.to_string()))?;
        }
        _ => {}
    }

    Ok(Json(serde_json::json!({
        "quality": quality,
        "next_review": result.next_review.to_string(),
        "interval": result.interval,
        "ease_factor": result.ease_factor,
        "familiarity": result.familiarity,
    })))
}

async fn review_stats(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    let pending = state
        .db
        .pending_review_count()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    let today = state
        .db
        .review_count_today()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({
        "pending": pending,
        "reviewed_today": today,
    })))
}
```

- [ ] **Step 2: Implement reading routes**

`crates/engai/src/routes/readings.rs`:
```rust
use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{delete, get, post},
    Json, Router,
};
use futures::stream::Stream;
use serde::Deserialize;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_readings).post(create_reading))
        .route("/{id}", get(get_reading).delete(delete_reading))
        .route("/{id}/analyze", get(analyze_reading))
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

async fn list_readings(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let readings = state
        .db
        .list_readings(q.limit.unwrap_or(50), q.offset.unwrap_or(0))
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "readings": readings })))
}

async fn get_reading(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    let reading = state
        .db
        .get_reading(id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Reading not found"))?;
    Ok(Json(serde_json::to_value(reading).unwrap()))
}

#[derive(Debug, Deserialize)]
struct CreateReading {
    pub title: Option<String>,
    pub content: String,
    pub source: Option<String>,
}

async fn create_reading(
    State(state): State<AppState>,
    Json(body): Json<CreateReading>,
) -> ApiResult<Json<serde_json::Value>> {
    let reading = state
        .db
        .add_reading(body.title.as_deref(), &body.content, body.source.as_deref())
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::to_value(reading).unwrap()))
}

async fn delete_reading(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    let deleted = state
        .db
        .delete_reading(id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "deleted": deleted })))
}

async fn analyze_reading(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, axum::Error>>>> {
    let reading = state
        .db
        .get_reading(id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Reading not found"))?;

    let ai = state.ai_client.clone();
    let prompt = state.prompt_engine.clone();

    let stream = async_stream::stream! {
        let analysis = match ai.analyze_reading(&reading.content, &prompt).await {
            Ok(text) => text,
            Err(e) => {
                yield Err(axum::Error::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())));
                return;
            }
        };
        let mut char_indices = analysis.char_indices().peekable();
        let mut buf = String::new();
        while let Some((_, ch)) = char_indices.next() {
            buf.push(ch);
            if buf.len() >= 3 || char_indices.peek().is_none() {
                let chunk = buf.clone();
                buf.clear();
                yield Ok(Event::default().data(chunk));
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
        yield Ok(Event::default().data("[DONE]"));
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
```

- [ ] **Step 3: Implement notes routes**

`crates/engai/src/routes/notes.rs`:
```rust
use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_notes).post(create_note))
        .route("/{id}", put(update_note).delete(delete_note))
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    pub target_type: Option<String>,
    pub target_id: Option<i64>,
}

async fn list_notes(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let notes = state
        .db
        .get_notes(
            q.target_type.as_deref().unwrap_or("word"),
            q.target_id.unwrap_or(0),
        )
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "notes": notes })))
}

#[derive(Debug, Deserialize)]
struct CreateNote {
    pub target_type: String,
    pub target_id: i64,
    pub content: String,
}

async fn create_note(
    State(state): State<AppState>,
    Json(body): Json<CreateNote>,
) -> ApiResult<Json<serde_json::Value>> {
    let note = state
        .db
        .add_note(&body.target_type, body.target_id, &body.content)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::to_value(note).unwrap()))
}

#[derive(Debug, Deserialize)]
struct UpdateNoteBody {
    pub content: String,
}

async fn update_note(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateNoteBody>,
) -> ApiResult<Json<serde_json::Value>> {
    let _existing = state
        .db
        .get_notes("word", id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;

    let note = state
        .db
        .add_note("word", id, &body.content)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::to_value(note).unwrap()))
}

async fn delete_note(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    let deleted = state
        .db
        .delete_note(id)
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "deleted": deleted })))
}
```

- [ ] **Step 4: Implement sync route**

`crates/engai/src/routes/sync.rs`:
```rust
use axum::{extract::State, routing::post, Json, Router};
use std::sync::Arc;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use engai_core::sync::SyncEngine;

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(trigger_sync))
}

async fn trigger_sync(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    let engine = SyncEngine::new(
        state.db.clone(),
        &state.config.docs_path(),
        &state.config.prompts_path(),
    );
    engine
        .sync_all()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}
```

- [ ] **Step 5: Implement stats route**

`crates/engai/src/routes/stats.rs`:
```rust
use axum::{extract::State, routing::get, Json, Router};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_stats))
}

async fn get_stats(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    let word_count = state
        .db
        .word_count()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    let phrase_count = state
        .db
        .phrase_count()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    let pending = state
        .db
        .pending_review_count()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;
    let reviewed_today = state
        .db
        .review_count_today()
        .await
        .map_err(|e| ApiError::internal(&e.to_string()))?;

    Ok(Json(serde_json::json!({
        "word_count": word_count,
        "phrase_count": phrase_count,
        "pending_reviews": pending,
        "reviewed_today": reviewed_today,
    })))
}
```

- [ ] **Step 6: Verify compilation**

Run: `cargo check -p engai`

- [ ] **Step 7: Commit**

```bash
git add crates/engai/src/routes/
git commit -m "feat(backend): add review, reading, notes, sync, and stats routes"
```

---

### Task 5: WebSocket Chat Route + Chat History DB

**Files:**
- Create: `crates/engai/src/routes/chat.rs`
- Modify: `crates/engai-core/src/db.rs` (add chat_history methods)
- Modify: `crates/engai-core/src/models.rs` (add ChatMessage model)

- [ ] **Step 1: Add ChatMessage model to engai-core**

Add to `crates/engai-core/src/models.rs`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatEntry {
    pub id: i64,
    pub role: String,
    pub content: String,
    pub created_at: NaiveDateTime,
}
```

- [ ] **Step 2: Add chat_history DB methods to db.rs**

Add to `crates/engai-core/src/db.rs`:
```rust
pub async fn add_chat_message(&self, role: &str, content: &str) -> Result<ChatEntry> {
    let row = sqlx::query_as::<_, ChatEntry>(
        "INSERT INTO chat_history (role, content) VALUES (?, ?) RETURNING *",
    )
    .bind(role)
    .bind(content)
    .fetch_one(&self.pool)
    .await
    .context("Failed to add chat message")?;
    Ok(row)
}

pub async fn get_recent_chat(&self, limit: i64) -> Result<Vec<ChatEntry>> {
    let rows = sqlx::query_as::<_, ChatEntry>(
        "SELECT * FROM chat_history ORDER BY created_at DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&self.pool)
    .await?;
    Ok(rows.into_iter().rev().collect())
}

pub async fn clear_chat(&self) -> Result<u64> {
    let result = sqlx::query("DELETE FROM chat_history")
        .execute(&self.pool)
        .await?;
    Ok(result.rows_affected())
}
```

Add `use crate::models::ChatEntry;` to the imports at top of db.rs.

- [ ] **Step 3: Implement WebSocket chat route**

`crates/engai/src/routes/chat.rs`:
```rust
use axum::{
    extract::{State, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use engai_core::ai::ChatMessage;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(ws_handler))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

#[derive(Debug, Serialize, Deserialize)]
struct WsMessage {
    role: String,
    content: String,
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(_) => break,
        };

        let text = match msg.to_text() {
            Ok(t) => t.to_string(),
            Err(_) => continue,
        };

        let parsed: WsMessage = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(_) => {
                let _ = sender
                    .send(axum::extract::ws::Message::Text(
                        serde_json::json!({"error": "invalid message"}).to_string().into(),
                    ))
                    .await;
                continue;
            }
        };

        if parsed.role != "user" {
            continue;
        }

        if let Err(e) = state.db.add_chat_message("user", &parsed.content).await {
            let _ = sender
                .send(
                    axum::extract::ws::Message::Text(
                        serde_json::json!({"error": e.to_string()}).to_string().into(),
                    ),
                )
                .await;
            continue;
        }

        let recent = match state.db.get_recent_chat(20).await {
            Ok(r) => r,
            Err(e) => {
                let _ = sender
                    .send(
                        axum::extract::ws::Message::Text(
                            serde_json::json!({"error": e.to_string()}).to_string().into(),
                        ),
                    )
                    .await;
                continue;
            }
        };

        let messages: Vec<ChatMessage> = recent
            .iter()
            .map(|r| ChatMessage {
                role: r.role.clone(),
                content: r.content.clone(),
            })
            .collect();

        let mut stream = match state.ai_client.chat_completion_stream(messages).await {
            Ok(s) => s,
            Err(e) => {
                let _ = sender
                    .send(
                        axum::extract::ws::Message::Text(
                            serde_json::json!({"error": e.to_string()}).to_string().into(),
                        ),
                    )
                    .await;
                continue;
            }
        };

        let mut full_response = String::new();
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(text) => {
                    full_response.push_str(&text);
                    let _ = sender
                        .send(
                            axum::extract::ws::Message::Text(
                                serde_json::json!({"role": "assistant", "content": text})
                                    .to_string()
                                    .into(),
                            ),
                        )
                        .await;
                }
                Err(e) => {
                    let _ = sender
                        .send(
                            axum::extract::ws::Message::Text(
                                serde_json::json!({"error": e.to_string()}).to_string().into(),
                            ),
                        )
                        .await;
                    break;
                }
            }
        }

        let _ = state.db.add_chat_message("assistant", &full_response).await;
    }
}
```

- [ ] **Step 4: Run existing tests to verify nothing broke**

Run: `cargo test -p engai-core`
Expected: All 27 tests pass

- [ ] **Step 5: Verify compilation**

Run: `cargo check -p engai`

- [ ] **Step 6: Commit**

```bash
git add crates/engai/src/routes/chat.rs crates/engai-core/src/db.rs crates/engai-core/src/models.rs
git commit -m "feat(backend): add WebSocket chat route and chat_history DB methods"
```

---

### Task 6: Axum Server Setup + Static File Serving + build.rs

**Files:**
- Create: `crates/engai/src/server.rs`
- Modify: `crates/engai/build.rs`
- Modify: `crates/engai/src/main.rs` (wire server)

- [ ] **Step 1: Implement server.rs**

`crates/engai/src/server.rs`:
```rust
use axum::{routing::get, Router};
use rust_embed::Embed;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::routes::{chat, notes, phrases, readings, reviews, stats, sync, words};
use crate::state::AppState;

#[derive(Embed)]
#[folder = "../../frontend/dist"]
struct Assets;

pub async fn run_server(state: AppState, port: u16) -> anyhow::Result<()> {
    let app = create_app(state);

    let addr = format!("{}:{}", state.config.server.host, port);
    tracing::info!("Engai server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .nest("/words", words::router())
        .nest("/phrases", phrases::router())
        .nest("/review", reviews::router())
        .nest("/readings", readings::router())
        .nest("/notes", notes::router())
        .nest("/chat", chat::router())
        .nest("/sync", sync::router())
        .nest("/stats", stats::router());

    let app = Router::new()
        .nest("/api", api)
        .fallback_service(get(serve_static));

    app.with_state(state).layer(cors)
}

async fn serve_static(axum::extract::Path(path): axum::extract::Path<String>) -> impl axum::response::IntoResponse {
    let path = if path.is_empty() { "index.html".to_string() } else { path };

    if let Some(content) = Assets::get(&path) {
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        axum::response::Response::builder()
            .header("Content-Type", mime.as_ref())
            .body(axum::body::Body::from(content.data.to_vec()))
            .unwrap()
            .into_response()
    } else {
        if let Some(content) = Assets::get("index.html") {
            axum::response::Response::builder()
                .header("Content-Type", "text/html")
                .body(axum::body::Body::from(content.data.to_vec()))
                .unwrap()
                .into_response()
        } else {
            axum::response::Response::builder()
                .status(404)
                .body(axum::body::Body::from("Not found"))
                .unwrap()
                .into_response()
        }
    }
}
```

- [ ] **Step 2: Update build.rs to build frontend**

`crates/engai/build.rs`:
```rust
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../../frontend/dist");
    println!("cargo:rerun-if-changed=../../frontend/package.json");
    println!("cargo:rerun-if-changed=../../frontend/src");

    let frontend_dist = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../frontend/dist");

    if !frontend_dist.exists() {
        let frontend_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../frontend");
        println!("cargo:warning=frontend/dist not found, running npm build...");

        let output = Command::new("npm")
            .args(["run", "build"])
            .current_dir(&frontend_dir)
            .output();

        match output {
            Ok(o) if o.status.success() => {
                println!("cargo:warning=frontend build succeeded");
            }
            Ok(o) => {
                println!("cargo:warning=frontend build failed: {}", String::from_utf8_lossy(&o.stderr));
            }
            Err(e) => {
                println!("cargo:warning=failed to run npm build: {}", e);
            }
        }
    }
}
```

- [ ] **Step 3: Create minimal frontend/dist placeholder**

For now, create a minimal `frontend/dist/index.html` so `cargo check` succeeds before we build the real frontend:

```bash
mkdir -p frontend/dist
```

Create `frontend/dist/index.html`:
```html
<!DOCTYPE html>
<html><body>Engai - placeholder</body></html>
```

- [ ] **Step 4: Wire server into main.rs**

Update the `Server` match arm in `crates/engai/src/main.rs`:
```rust
Some(Commands::Server { port }) => {
    let config = engai_core::config::Config::load_global()?;
    let db_path = config.db_path();
    let db = engai_core::db::Db::new(&db_path).await?;
    let state = crate::state::AppState::new(std::sync::Arc::new(db), config)?;
    crate::server::run_server(state, port).await?;
}
```

- [ ] **Step 5: Verify compilation**

Run: `cargo check -p engai`

- [ ] **Step 6: Commit**

```bash
git add crates/engai/src/server.rs crates/engai/build.rs crates/engai/src/main.rs frontend/dist/
git commit -m "feat(backend): add Axum server with static serving, API routes, and build.rs"
```

---

### Task 7: React Frontend — Project Scaffold + Vite + Tailwind CSS 4 + Shadcn

**Files:**
- Create: `frontend/package.json`
- Create: `frontend/vite.config.ts`
- Create: `frontend/tsconfig.json`
- Create: `frontend/tsconfig.app.json`
- Create: `frontend/tsconfig.node.json`
- Create: `frontend/index.html`
- Create: `frontend/components.json`
- Create: `frontend/src/main.tsx`
- Create: `frontend/src/App.tsx`
- Create: `frontend/src/index.css`
- Create: `frontend/src/lib/utils.ts`
- Create: `frontend/src/lib/api.ts`
- Create: `frontend/src/hooks/useWebSocket.ts`

- [ ] **Step 1: Initialize frontend with Vite + React + TypeScript**

```bash
cd frontend && npm create vite@latest . -- --template react-ts
npm install
```

If the directory is not empty, use `--force` or create the files manually.

- [ ] **Step 2: Install dependencies**

```bash
cd frontend
npm install react-router-dom @tanstack/react-query recharts lucide-react
npm install -D tailwindcss @tailwindcss/vite
npx shadcn@latest init -d
```

Shadcn init should create `components.json`, `src/lib/utils.ts`, and modify `src/index.css`.

- [ ] **Step 3: Configure Tailwind CSS 4**

`frontend/vite.config.ts`:
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
    outDir: 'dist',
  },
})
```

- [ ] **Step 4: Configure index.css for Tailwind CSS 4**

`frontend/src/index.css`:
```css
@import "tailwindcss";
@import "tw-animate-css";

@custom-variant dark (&:is(.dark *));

@theme inline {
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  --font-sans: var(--font-sans);
  --color-sidebar-ring: var(--sidebar-ring);
  --color-sidebar-border: var(--sidebar-border);
  --color-sidebar-accent-foreground: var(--sidebar-accent-foreground);
  --color-sidebar-accent: var(--sidebar-accent);
  --color-sidebar-primary-foreground: var(--sidebar-primary-foreground);
  --color-sidebar-primary: var(--sidebar-primary);
  --color-sidebar-foreground: var(--sidebar-foreground);
  --color-sidebar: var(--sidebar);
  --color-chart-5: var(--chart-5);
  --color-chart-4: var(--chart-4);
  --color-chart-3: var(--chart-3);
  --color-chart-2: var(--chart-2);
  --color-chart-1: var(--chart-1);
  --color-ring: var(--ring);
  --color-input: var(--input);
  --color-border: var(--border);
  --color-destructive: var(--destructive);
  --color-accent-foreground: var(--accent-foreground);
  --color-accent: var(--accent);
  --color-muted-foreground: var(--muted-foreground);
  --color-muted: var(--muted);
  --color-secondary-foreground: var(--secondary-foreground);
  --color-secondary: var(--secondary);
  --color-primary-foreground: var(--primary-foreground);
  --color-primary: var(--primary);
  --color-popover-foreground: var(--popover-foreground);
  --color-popover: var(--popover);
  --color-card-foreground: var(--card-foreground);
  --color-card: var(--card);
  --radius-sm: calc(var(--radius) - 4px);
  --radius-md: calc(var(--radius) - 2px);
  --radius-lg: var(--radius);
  --radius-xl: calc(var(--radius) + 4px);
}

:root {
  --radius: 0.625rem;
  --background: oklch(1 0 0);
  --foreground: oklch(0.145 0 0);
  --card: oklch(1 0 0);
  --card-foreground: oklch(0.145 0 0);
  --popover: oklch(1 0 0);
  --popover-foreground: oklch(0.145 0 0);
  --primary: oklch(0.205 0 0);
  --primary-foreground: oklch(0.985 0 0);
  --secondary: oklch(0.97 0 0);
  --secondary-foreground: oklch(0.205 0 0);
  --muted: oklch(0.97 0 0);
  --muted-foreground: oklch(0.556 0 0);
  --accent: oklch(0.97 0 0);
  --accent-foreground: oklch(0.205 0 0);
  --destructive: oklch(0.577 0.245 27.325);
  --border: oklch(0.922 0 0);
  --input: oklch(0.922 0 0);
  --ring: oklch(0.708 0 0);
  --chart-1: oklch(0.646 0.222 41.116);
  --chart-2: oklch(0.6 0.118 184.714);
  --chart-3: oklch(0.398 0.07 227.392);
  --chart-4: oklch(0.828 0.189 84.429);
  --chart-5: oklch(0.769 0.188 70.08);
  --sidebar: oklch(0.985 0 0);
  --sidebar-foreground: oklch(0.145 0 0);
  --sidebar-primary: oklch(0.205 0 0);
  --sidebar-primary-foreground: oklch(0.985 0 0);
  --sidebar-accent: oklch(0.97 0 0);
  --sidebar-accent-foreground: oklch(0.205 0 0);
  --sidebar-border: oklch(0.922 0 0);
  --sidebar-ring: oklch(0.708 0 0);
}

.dark {
  --background: oklch(0.145 0 0);
  --foreground: oklch(0.985 0 0);
  --card: oklch(0.205 0 0);
  --card-foreground: oklch(0.985 0 0);
  --popover: oklch(0.205 0 0);
  --popover-foreground: oklch(0.985 0 0);
  --primary: oklch(0.985 0 0);
  --primary-foreground: oklch(0.205 0 0);
  --secondary: oklch(0.269 0 0);
  --secondary-foreground: oklch(0.985 0 0);
  --muted: oklch(0.269 0 0);
  --muted-foreground: oklch(0.708 0 0);
  --accent: oklch(0.269 0 0);
  --accent-foreground: oklch(0.985 0 0);
  --destructive: oklch(0.704 0.191 22.216);
  --border: oklch(0.269 0 0);
  --input: oklch(0.269 0 0);
  --ring: oklch(0.556 0 0);
  --chart-1: oklch(0.488 0.243 264.376);
  --chart-2: oklch(0.696 0.17 162.48);
  --chart-3: oklch(0.769 0.188 70.08);
  --chart-4: oklch(0.627 0.265 303.9);
  --chart-5: oklch(0.645 0.246 16.439);
  --sidebar: oklch(0.205 0 0);
  --sidebar-foreground: oklch(0.985 0 0);
  --sidebar-primary: oklch(0.488 0.243 264.376);
  --sidebar-primary-foreground: oklch(0.985 0 0);
  --sidebar-accent: oklch(0.269 0 0);
  --sidebar-accent-foreground: oklch(0.985 0 0);
  --sidebar-border: oklch(0.269 0 0);
  --sidebar-ring: oklch(0.556 0 0);
}

@layer base {
  * {
    @apply border-border;
  }
  body {
    @apply bg-background text-foreground;
  }
}
```

- [ ] **Step 5: Create API utility**

`frontend/src/lib/api.ts`:
```typescript
const BASE = '/api';

export async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json', ...init?.headers },
    ...init,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(body.error || 'Request failed');
  }
  return res.json();
}

export function fetchSSE(
  url: string,
  onChunk: (text: string) => void,
  onDone: () => void,
  onError: (err: Error) => void,
) {
  const evtSource = new EventSource(`${BASE}${url}`);
  evtSource.onmessage = (e) => {
    if (e.data === '[DONE]') {
      evtSource.close();
      onDone();
      return;
    }
    onChunk(e.data);
  };
  evtSource.onerror = () => {
    evtSource.close();
    onError(new Error('SSE connection error'));
  };
  return () => evtSource.close();
}
```

- [ ] **Step 6: Create WebSocket hook**

`frontend/src/hooks/useWebSocket.ts`:
```typescript
import { useCallback, useEffect, useRef, useState } from 'react';

interface UseWebSocketOptions {
  onMessage: (data: any) => void;
  onError?: (err: Event) => void;
}

export function useWebSocket(url: string, { onMessage, onError }: UseWebSocketOptions) {
  const wsRef = useRef<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);

  const connect = useCallback(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const ws = new WebSocket(`${protocol}//${window.location.host}${url}`);
    wsRef.current = ws;

    ws.onopen = () => setConnected(true);
    ws.onclose = () => {
      setConnected(false);
      setTimeout(connect, 3000);
    };
    ws.onerror = (e) => onError?.(e);
    ws.onmessage = (e) => {
      try {
        onMessage(JSON.parse(e.data));
      } catch {
        // ignore parse errors
      }
    };
  }, [url, onMessage, onError]);

  useEffect(() => {
    connect();
    return () => wsRef.current?.close();
  }, [connect]);

  const send = useCallback((data: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(data));
    }
  }, []);

  return { connected, send };
}
```

- [ ] **Step 7: Create main.tsx and App.tsx**

`frontend/src/main.tsx`:
```tsx
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter } from 'react-router-dom'
import App from './App'
import './index.css'

const queryClient = new QueryClient()

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </QueryClientProvider>
  </StrictMode>,
)
```

`frontend/src/App.tsx`:
```tsx
import { Routes, Route } from 'react-router-dom'
import Layout from './components/Layout'
import Dashboard from './pages/Dashboard'
import Vocabulary from './pages/Vocabulary'
import WordCard from './pages/WordCard'
import Review from './pages/Review'
import Reading from './pages/Reading'
import Chat from './pages/Chat'

export default function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/" element={<Dashboard />} />
        <Route path="/vocabulary" element={<Vocabulary />} />
        <Route path="/words/:word" element={<WordCard />} />
        <Route path="/review" element={<Review />} />
        <Route path="/readings" element={<Reading />} />
        <Route path="/chat" element={<Chat />} />
      </Route>
    </Routes>
  )
}
```

- [ ] **Step 8: Install Shadcn components**

```bash
cd frontend
npx shadcn@latest add button card input badge separator sheet scroll-area tabs textarea skeleton
```

- [ ] **Step 9: Verify frontend builds**

```bash
cd frontend && npm run build
```
Expected: `dist/` directory created

- [ ] **Step 10: Commit**

```bash
git add frontend/
git commit -m "feat(frontend): scaffold React + Vite + Tailwind CSS 4 + Shadcn + API utils"
```

---

### Task 8: Layout + FamiliarityBadge + MarkdownRender Components

**Files:**
- Create: `frontend/src/components/Layout.tsx`
- Create: `frontend/src/components/FamiliarityBadge.tsx`
- Create: `frontend/src/components/MarkdownRender.tsx`

- [ ] **Step 1: Create Layout with sidebar navigation**

`frontend/src/components/Layout.tsx`:
```tsx
import { NavLink, Outlet } from 'react-router-dom'
import { BookOpen, BarChart3, RefreshCw, MessageCircle, FileText } from 'lucide-react'

const navItems = [
  { to: '/', label: 'Dashboard', icon: BarChart3 },
  { to: '/vocabulary', label: 'Vocabulary', icon: BookOpen },
  { to: '/review', label: 'Review', icon: RefreshCw },
  { to: '/readings', label: 'Reading', icon: FileText },
  { to: '/chat', label: 'Chat', icon: MessageCircle },
]

export default function Layout() {
  return (
    <div className="flex h-screen">
      <aside className="w-60 border-r bg-sidebar flex flex-col">
        <div className="p-4 font-bold text-xl tracking-tight">Engai</div>
        <nav className="flex-1 px-2">
          {navItems.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              end={to === '/'}
              className={({ isActive }) =>
                `flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors ${
                  isActive
                    ? 'bg-sidebar-accent text-sidebar-accent-foreground font-medium'
                    : 'text-sidebar-foreground/70 hover:bg-sidebar-accent/50'
                }`
              }
            >
              <Icon className="h-4 w-4" />
              {label}
            </NavLink>
          ))}
        </nav>
      </aside>
      <main className="flex-1 overflow-auto">
        <div className="mx-auto max-w-4xl p-6">
          <Outlet />
        </div>
      </main>
    </div>
  )
}
```

- [ ] **Step 2: Create FamiliarityBadge**

`frontend/src/components/FamiliarityBadge.tsx`:
```tsx
import { Badge } from '@/components/ui/badge'

const colors = [
  'bg-gray-200 text-gray-700',
  'bg-red-100 text-red-700',
  'bg-orange-100 text-orange-700',
  'bg-yellow-100 text-yellow-700',
  'bg-green-100 text-green-700',
  'bg-blue-100 text-blue-700',
]

export default function FamiliarityBadge({ level }: { level: number }) {
  const clamped = Math.max(0, Math.min(5, level))
  return (
    <Badge variant="secondary" className={colors[clamped]}>
      Lv.{clamped}
    </Badge>
  )
}
```

- [ ] **Step 3: Create MarkdownRender**

`frontend/src/components/MarkdownRender.tsx`:
```tsx
interface Props {
  content: string
}

export default function MarkdownRender({ content }: Props) {
  return (
    <div className="prose prose-sm max-w-none dark:prose-invert">
      {content.split('\n').map((line, i) => {
        if (line.startsWith('## '))
          return <h2 key={i} className="text-lg font-semibold mt-4 mb-2">{line.slice(3)}</h2>
        if (line.startsWith('# '))
          return <h1 key={i} className="text-xl font-bold mt-4 mb-2">{line.slice(2)}</h1>
        if (line.startsWith('- '))
          return <li key={i} className="ml-4">{line.slice(2)}</li>
        if (line.startsWith('> '))
          return <blockquote key={i} className="border-l-4 pl-4 italic text-muted-foreground">{line.slice(2)}</blockquote>
        if (line.startsWith('**') && line.endsWith('**'))
          return <p key={i}><strong>{line.slice(2, -2)}</strong></p>
        if (line.trim() === '')
          return <br key={i} />
        return <p key={i}>{line}</p>
      })}
    </div>
  )
}
```

- [ ] **Step 4: Verify frontend builds**

```bash
cd frontend && npm run build
```

- [ ] **Step 5: Commit**

```bash
git add frontend/src/components/
git commit -m "feat(frontend): add Layout, FamiliarityBadge, MarkdownRender components"
```

---

### Task 9: Dashboard Page

**Files:**
- Create: `frontend/src/pages/Dashboard.tsx`

- [ ] **Step 1: Implement Dashboard**

`frontend/src/pages/Dashboard.tsx`:
```tsx
import { useQuery } from '@tanstack/react-query'
import { BookOpen, RefreshCw, CheckCircle, Clock } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { apiFetch } from '@/lib/api'

interface Stats {
  word_count: number
  phrase_count: number
  pending_reviews: number
  reviewed_today: number
}

export default function Dashboard() {
  const { data: stats } = useQuery({
    queryKey: ['stats'],
    queryFn: () => apiFetch<Stats>('/stats'),
  })

  const cards = [
    {
      title: 'Words',
      value: stats?.word_count ?? 0,
      icon: BookOpen,
      color: 'text-blue-500',
    },
    {
      title: 'Phrases',
      value: stats?.phrase_count ?? 0,
      icon: BookOpen,
      color: 'text-purple-500',
    },
    {
      title: 'Pending Reviews',
      value: stats?.pending_reviews ?? 0,
      icon: Clock,
      color: 'text-orange-500',
    },
    {
      title: 'Reviewed Today',
      value: stats?.reviewed_today ?? 0,
      icon: CheckCircle,
      color: 'text-green-500',
    },
  ]

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">Dashboard</h1>
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {cards.map(({ title, value, icon: Icon, color }) => (
          <Card key={title}>
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                {title}
              </CardTitle>
              <Icon className={`h-4 w-4 ${color}`} />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{value}</div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Verify frontend builds**

```bash
cd frontend && npm run build
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/pages/Dashboard.tsx
git commit -m "feat(frontend): add Dashboard page with stats overview"
```

---

### Task 10: Vocabulary Page (Word + Phrase List)

**Files:**
- Create: `frontend/src/pages/Vocabulary.tsx`

- [ ] **Step 1: Implement Vocabulary page**

`frontend/src/pages/Vocabulary.tsx`:
```tsx
import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import { Link } from 'react-router-dom'
import { Search, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent } from '@/components/ui/card'
import { apiFetch } from '@/lib/api'
import FamiliarityBadge from '@/components/FamiliarityBadge'

interface Word {
  id: number
  word: string
  meaning: string | null
  familiarity: number
  next_review: string | null
}

interface Phrase {
  id: number
  phrase: string
  meaning: string | null
  familiarity: number
  next_review: string | null
}

export default function Vocabulary() {
  const [search, setSearch] = useState('')
  const [tab, setTab] = useState<'words' | 'phrases'>('words')

  const { data: wordsData } = useQuery({
    queryKey: ['words', search],
    queryFn: () => apiFetch<{ words: Word[] }>(`/words?search=${encodeURIComponent(search)}`),
  })

  const { data: phrasesData } = useQuery({
    queryKey: ['phrases', search],
    queryFn: () => apiFetch<{ phrases: Phrase[] }>(`/phrases?search=${encodeURIComponent(search)}`),
  })

  const items = tab === 'words' ? (wordsData?.words ?? []) : (phrasesData?.phrases ?? [])

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Vocabulary</h1>
        <Button size="sm">
          <Plus className="h-4 w-4 mr-1" /> Add Word
        </Button>
      </div>

      <div className="flex gap-2">
        <Button
          variant={tab === 'words' ? 'default' : 'outline'}
          size="sm"
          onClick={() => setTab('words')}
        >
          Words
        </Button>
        <Button
          variant={tab === 'phrases' ? 'default' : 'outline'}
          size="sm"
          onClick={() => setTab('phrases')}
        >
          Phrases
        </Button>
        <div className="relative flex-1 max-w-xs ml-auto">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9"
          />
        </div>
      </div>

      <div className="space-y-2">
        {items.length === 0 && (
          <p className="text-muted-foreground text-sm py-8 text-center">
            No {tab} found. Add your first word!
          </p>
        )}
        {items.map((item) => (
          <Link key={item.id} to={tab === 'words' ? `/words/${item.word}` : '#'}>
            <Card className="hover:bg-accent/50 transition-colors cursor-pointer">
              <CardContent className="flex items-center justify-between py-3 px-4">
                <div>
                  <span className="font-medium">
                    {tab === 'words' ? item.word : item.phrase}
                  </span>
                  {item.meaning && (
                    <span className="text-sm text-muted-foreground ml-2">
                      {item.meaning}
                    </span>
                  )}
                </div>
                <FamiliarityBadge level={item.familiarity} />
              </CardContent>
            </Card>
          </Link>
        ))}
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Verify frontend builds**

```bash
cd frontend && npm run build
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/pages/Vocabulary.tsx
git commit -m "feat(frontend): add Vocabulary page with word/phrase list, search, and tabs"
```

---

### Task 11: WordCard Page (Detail + AI Explain)

**Files:**
- Create: `frontend/src/pages/WordCard.tsx`

- [ ] **Step 1: Implement WordCard page**

`frontend/src/pages/WordCard.tsx`:
```tsx
import { useQuery } from '@tanstack/react-query'
import { useParams } from 'react-router-dom'
import { Sparkles, ArrowLeft } from 'lucide-react'
import { useState } from 'react'
import { Link } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { apiFetch, fetchSSE } from '@/lib/api'
import FamiliarityBadge from '@/components/FamiliarityBadge'
import MarkdownRender from '@/components/MarkdownRender'

interface Word {
  id: number
  word: string
  phonetic: string | null
  meaning: string | null
  familiarity: number
  next_review: string | null
  interval: number
  ease_factor: number
  created_at: string
  updated_at: string
}

interface Example {
  id: number
  sentence: string
  source: string | null
}

interface Note {
  id: number
  content: string
  created_at: string
}

export default function WordCard() {
  const { word } = useParams<{ word: string }>()
  const [explanation, setExplanation] = useState('')

  const { data: wordData, isLoading } = useQuery({
    queryKey: ['word', word],
    queryFn: () => apiFetch<Word>(`/words/${word}`),
    enabled: !!word,
  })

  const { data: examplesData } = useQuery({
    queryKey: ['word-examples', word],
    queryFn: () => apiFetch<{ examples: Example[] }>(`/words/${word}/examples`),
    enabled: !!word,
  })

  const { data: notesData } = useQuery({
    queryKey: ['word-notes', word],
    queryFn: () => apiFetch<{ notes: Note[] }>(`/notes?target_type=word&target_id=${wordData?.id}`),
    enabled: !!wordData?.id,
  })

  const handleExplain = () => {
    if (!word) return
    setExplanation('')
    fetchSSE(
      `/words/${word}/explain`,
      (chunk) => setExplanation((prev) => prev + chunk),
      () => {},
      (err) => console.error(err),
    )
  }

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-4 w-64" />
        <Skeleton className="h-32" />
      </div>
    )
  }

  if (!wordData) {
    return <div className="text-muted-foreground">Word not found</div>
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3">
        <Link to="/vocabulary">
          <Button variant="ghost" size="icon">
            <ArrowLeft className="h-4 w-4" />
          </Button>
        </Link>
        <div>
          <h1 className="text-2xl font-bold">{wordData.word}</h1>
          {wordData.phonetic && (
            <p className="text-muted-foreground">{wordData.phonetic}</p>
          )}
        </div>
        <FamiliarityBadge level={wordData.familiarity} />
      </div>

      <Card>
        <CardHeader><CardTitle>Meaning</CardTitle></CardHeader>
        <CardContent>
          <p>{wordData.meaning || 'No meaning added yet.'}</p>
        </CardContent>
      </Card>

      {examplesData && examplesData.examples.length > 0 && (
        <Card>
          <CardHeader><CardTitle>Examples</CardTitle></CardHeader>
          <CardContent>
            <ul className="list-disc pl-5 space-y-1">
              {examplesData.examples.map((ex) => (
                <li key={ex.id}>{ex.sentence}</li>
              ))}
            </ul>
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle>AI Explanation</CardTitle>
          <Button onClick={handleExplain} size="sm">
            <Sparkles className="h-4 w-4 mr-1" /> Explain
          </Button>
        </CardHeader>
        <CardContent>
          {explanation ? (
            <MarkdownRender content={explanation} />
          ) : (
            <p className="text-muted-foreground text-sm">
              Click "Explain" to get an AI-powered explanation.
            </p>
          )}
        </CardContent>
      </Card>

      {notesData && notesData.notes.length > 0 && (
        <Card>
          <CardHeader><CardTitle>Notes</CardTitle></CardHeader>
          <CardContent>
            <ul className="space-y-2">
              {notesData.notes.map((note) => (
                <li key={note.id} className="text-sm border-l-2 pl-3">{note.content}</li>
              ))}
            </ul>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
```

- [ ] **Step 2: Verify frontend builds**

```bash
cd frontend && npm run build
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/pages/WordCard.tsx
git commit -m "feat(frontend): add WordCard page with detail view and AI explain streaming"
```

---

### Task 12: Review Page (Anki-Style Flash Cards)

**Files:**
- Create: `frontend/src/components/FlashCard.tsx`
- Create: `frontend/src/pages/Review.tsx`

- [ ] **Step 1: Create FlashCard component**

`frontend/src/components/FlashCard.tsx`:
```tsx
import { useState } from 'react'
import { Card, CardContent } from '@/components/ui/card'

interface Props {
  front: string
  back: string
}

export default function FlashCard({ front, back }: Props) {
  const [flipped, setFlipped] = useState(false)

  return (
    <div
      className="cursor-pointer perspective-1000 w-full max-w-md mx-auto"
      onClick={() => setFlipped(!flipped)}
    >
      <Card className="min-h-[200px] flex items-center justify-center transition-transform duration-300">
        <CardContent className="text-center p-8">
          {flipped ? (
            <div>
              <p className="text-lg">{back}</p>
              <p className="text-xs text-muted-foreground mt-4">Click to flip back</p>
            </div>
          ) : (
            <div>
              <p className="text-2xl font-bold">{front}</p>
              <p className="text-xs text-muted-foreground mt-4">Click to reveal</p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
```

- [ ] **Step 2: Create Review page**

`frontend/src/pages/Review.tsx`:
```tsx
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { apiFetch } from '@/lib/api'
import FlashCard from '@/components/FlashCard'

interface ReviewItem {
  target_type: string
  id: number
  display: string
  familiarity: number
  interval: number
  ease_factor: number
}

interface ReviewResult {
  quality: number
  next_review: string
  interval: number
  ease_factor: number
  familiarity: number
}

const qualityLabels = [
  { value: 0, label: 'Again', color: 'bg-red-500 hover:bg-red-600' },
  { value: 1, label: 'Hard', color: 'bg-orange-500 hover:bg-orange-600' },
  { value: 2, label: 'Difficult', color: 'bg-yellow-500 hover:bg-yellow-600' },
  { value: 3, label: 'Good', color: 'bg-green-400 hover:bg-green-500' },
  { value: 4, label: 'Easy', color: 'bg-green-500 hover:bg-green-600' },
  { value: 5, label: 'Perfect', color: 'bg-blue-500 hover:bg-blue-600' },
]

export default function Review() {
  const [currentIndex, setCurrentIndex] = useState(0)
  const queryClient = useQueryClient()

  const { data, isLoading } = useQuery({
    queryKey: ['review-queue'],
    queryFn: () => apiFetch<{ items: ReviewItem[] }>('/review/today'),
  })

  const items = data?.items ?? []
  const current = items[currentIndex]

  const submitMutation = useMutation({
    mutationFn: ({ target_type, id, quality }: { target_type: string; id: number; quality: number }) =>
      apiFetch<ReviewResult>(`/review/${target_type}/${id}`, {
        method: 'POST',
        body: JSON.stringify({ quality }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['review-queue'] })
      setCurrentIndex((i) => Math.min(i + 1, items.length - 1))
    },
  })

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-64" />
      </div>
    )
  }

  if (items.length === 0) {
    return (
      <div className="space-y-4">
        <h1 className="text-2xl font-bold">Review</h1>
        <Card>
          <CardContent className="text-center py-12">
            <p className="text-lg text-muted-foreground">No reviews due right now. Great job!</p>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Review</h1>
        <span className="text-sm text-muted-foreground">
          {currentIndex + 1} / {items.length}
        </span>
      </div>

      <FlashCard front={current.display} back="Rate your recall" />

      {submitMutation.isPending ? (
        <div className="text-center text-muted-foreground text-sm">Processing...</div>
      ) : (
        <div className="flex gap-2 justify-center flex-wrap">
          {qualityLabels.map(({ value, label, color }) => (
            <Button
              key={value}
              className={`${color} text-white`}
              onClick={() =>
                submitMutation.mutate({
                  target_type: current.target_type,
                  id: current.id,
                  quality: value,
                })
              }
            >
              {label}
            </Button>
          ))}
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 3: Verify frontend builds**

```bash
cd frontend && npm run build
```

- [ ] **Step 4: Commit**

```bash
git add frontend/src/components/FlashCard.tsx frontend/src/pages/Review.tsx
git commit -m "feat(frontend): add Review page with Anki-style flash cards and quality rating"
```

---

### Task 13: Reading Page

**Files:**
- Create: `frontend/src/pages/Reading.tsx`

- [ ] **Step 1: Implement Reading page**

`frontend/src/pages/Reading.tsx`:
```tsx
import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import { FileText, Sparkles } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { ScrollArea } from '@/components/ui/scroll-area'
import { apiFetch, fetchSSE } from '@/lib/api'
import MarkdownRender from '@/components/MarkdownRender'

interface Reading {
  id: number
  title: string | null
  content: string
  source: string | null
  created_at: string
}

export default function Reading() {
  const [selectedId, setSelectedId] = useState<number | null>(null)
  const [analysis, setAnalysis] = useState('')

  const { data: readingsData, isLoading } = useQuery({
    queryKey: ['readings'],
    queryFn: () => apiFetch<{ readings: Reading[] }>('/readings'),
  })

  const { data: reading } = useQuery({
    queryKey: ['reading', selectedId],
    queryFn: () => apiFetch<Reading>(`/readings/${selectedId}`),
    enabled: !!selectedId,
  })

  const handleAnalyze = () => {
    if (!selectedId) return
    setAnalysis('')
    fetchSSE(
      `/readings/${selectedId}/analyze`,
      (chunk) => setAnalysis((prev) => prev + chunk),
      () => {},
      (err) => console.error(err),
    )
  }

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-64" />
      </div>
    )
  }

  const readings = readingsData?.readings ?? []

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">Reading</h1>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="space-y-2">
          {readings.map((r) => (
            <Card
              key={r.id}
              className={`cursor-pointer transition-colors ${
                selectedId === r.id ? 'border-primary' : 'hover:bg-accent/50'
              }`}
              onClick={() => {
                setSelectedId(r.id)
                setAnalysis('')
              }}
            >
              <CardContent className="py-3 px-4">
                <div className="flex items-center gap-2">
                  <FileText className="h-4 w-4 text-muted-foreground" />
                  <span className="font-medium text-sm">
                    {r.title || `Reading #${r.id}`}
                  </span>
                </div>
              </CardContent>
            </Card>
          ))}
          {readings.length === 0 && (
            <p className="text-sm text-muted-foreground">No readings yet.</p>
          )}
        </div>

        <div className="md:col-span-2">
          {reading ? (
            <div className="space-y-4">
              <Card>
                <CardHeader className="flex flex-row items-center justify-between">
                  <CardTitle>{reading.title || `Reading #${reading.id}`}</CardTitle>
                  {reading.source && (
                    <a
                      href={reading.source}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-sm text-blue-500 hover:underline"
                    >
                      Source
                    </a>
                  )}
                </CardHeader>
                <CardContent>
                  <ScrollArea className="h-64">
                    <p className="whitespace-pre-wrap text-sm">{reading.content}</p>
                  </ScrollArea>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="flex flex-row items-center justify-between">
                  <CardTitle>AI Analysis</CardTitle>
                  <Button onClick={handleAnalyze} size="sm">
                    <Sparkles className="h-4 w-4 mr-1" /> Analyze
                  </Button>
                </CardHeader>
                <CardContent>
                  {analysis ? (
                    <MarkdownRender content={analysis} />
                  ) : (
                    <p className="text-muted-foreground text-sm">
                      Click "Analyze" to get AI-powered reading analysis.
                    </p>
                  )}
                </CardContent>
              </Card>
            </div>
          ) : (
            <Card>
              <CardContent className="py-12 text-center text-muted-foreground">
                Select a reading to view its content.
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Verify frontend builds**

```bash
cd frontend && npm run build
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/pages/Reading.tsx
git commit -m "feat(frontend): add Reading page with list view and AI analysis streaming"
```

---

### Task 14: Chat Page (WebSocket AI Conversation)

**Files:**
- Create: `frontend/src/pages/Chat.tsx`

- [ ] **Step 1: Implement Chat page**

`frontend/src/pages/Chat.tsx`:
```tsx
import { useState, useRef, useEffect, useCallback } from 'react'
import { Send, Trash2, Wifi, WifiOff } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useWebSocket } from '@/hooks/useWebSocket'

interface ChatMsg {
  role: string
  content: string
}

export default function Chat() {
  const [messages, setMessages] = useState<ChatMsg[]>([])
  const [input, setInput] = useState('')
  const bottomRef = useRef<HTMLDivElement>(null)

  const onMessage = useCallback((data: ChatMsg) => {
    if (data.role === 'assistant') {
      setMessages((prev) => {
        const last = prev[prev.length - 1]
        if (last?.role === 'assistant') {
          return [...prev.slice(0, -1), { role: 'assistant', content: last.content + data.content }]
        }
        return [...prev, { role: 'assistant', content: data.content }]
      })
    } else if (data.error) {
      setMessages((prev) => [...prev, { role: 'system', content: `Error: ${data.error}` }])
    }
  }, [])

  const { connected, send } = useWebSocket('/api/chat', { onMessage })

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  const handleSend = () => {
    if (!input.trim()) return
    const userMsg = { role: 'user', content: input.trim() }
    setMessages((prev) => [...prev, userMsg])
    send(userMsg)
    setInput('')
  }

  const handleClear = () => {
    setMessages([])
  }

  return (
    <div className="flex flex-col h-[calc(100vh-8rem)]">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <h1 className="text-2xl font-bold">Chat</h1>
          {connected ? (
            <Wifi className="h-4 w-4 text-green-500" />
          ) : (
            <WifiOff className="h-4 w-4 text-red-500" />
          )}
        </div>
        <Button variant="ghost" size="icon" onClick={handleClear}>
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>

      <Card className="flex-1 flex flex-col">
        <ScrollArea className="flex-1 p-4">
          {messages.length === 0 && (
            <div className="text-center text-muted-foreground py-12">
              Start a conversation to practice English!
            </div>
          )}
          {messages.map((msg, i) => (
            <div
              key={i}
              className={`mb-4 flex ${
                msg.role === 'user' ? 'justify-end' : msg.role === 'system' ? 'justify-center' : 'justify-start'
              }`}
            >
              <div
                className={`max-w-[80%] rounded-lg px-4 py-2 text-sm ${
                  msg.role === 'user'
                    ? 'bg-primary text-primary-foreground'
                    : msg.role === 'system'
                    ? 'bg-destructive/10 text-destructive'
                    : 'bg-muted'
                }`}
              >
                {msg.content}
              </div>
            </div>
          ))}
          <div ref={bottomRef} />
        </ScrollArea>

        <div className="border-t p-4 flex gap-2">
          <Input
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSend()}
            placeholder="Type your message..."
            className="flex-1"
          />
          <Button onClick={handleSend} disabled={!input.trim()}>
            <Send className="h-4 w-4" />
          </Button>
        </div>
      </Card>
    </div>
  )
}
```

- [ ] **Step 2: Verify frontend builds**

```bash
cd frontend && npm run build
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/pages/Chat.tsx
git commit -m "feat(frontend): add Chat page with WebSocket AI conversation"
```

---

### Task 15: Integration — Build Embedding + End-to-End Test

**Files:**
- Verify: `crates/engai/build.rs`
- Verify: `crates/engai/src/server.rs`
- Verify: `crates/engai/src/main.rs`

- [ ] **Step 1: Build the full binary**

```bash
cd frontend && npm run build
cd ../.. && cargo build
```
Expected: Binary compiles successfully with embedded frontend.

- [ ] **Step 2: Add a `sync` note to notes route for `update_note`**

The current `update_note` in notes.rs is a stub. Fix it to use actual DB update via delete+re-insert pattern (since there's no update SQL for notes). This is acceptable for Phase 2.

- [ ] **Step 3: Run all existing tests**

```bash
cargo test
```
Expected: All 27+ tests pass

- [ ] **Step 4: Run clippy**

```bash
cargo clippy -- -D warnings
```
Expected: No warnings

- [ ] **Step 5: Manual smoke test**

```bash
cargo run -- -s
```
Open http://localhost:3000 in browser. Verify:
- Dashboard loads with stats
- Vocabulary page shows
- Navigation works between pages
- WebSocket chat connects (status indicator)

- [ ] **Step 6: Fix any issues found during smoke test**

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: complete Phase 2 — full web server + React frontend integration"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Backend deps + AppState + ApiError | `Cargo.toml`, `state.rs`, `error.rs` |
| 2 | Word CRUD + AI Explain SSE routes | `routes/words.rs` |
| 3 | Phrase CRUD + AI Explain SSE routes | `routes/phrases.rs` |
| 4 | Review, Reading, Notes, Sync, Stats routes | 5 route files |
| 5 | WebSocket Chat + chat_history DB | `routes/chat.rs`, `db.rs`, `models.rs` |
| 6 | Axum server + rust-embed + build.rs | `server.rs`, `build.rs`, `main.rs` |
| 7 | React scaffold + Vite + Tailwind + Shadcn | `frontend/` |
| 8 | Layout + FamiliarityBadge + MarkdownRender | 3 components |
| 9 | Dashboard page | `Dashboard.tsx` |
| 10 | Vocabulary page | `Vocabulary.tsx` |
| 11 | WordCard page | `WordCard.tsx` |
| 12 | Review page + FlashCard | `Review.tsx`, `FlashCard.tsx` |
| 13 | Reading page | `Reading.tsx` |
| 14 | Chat page | `Chat.tsx` |
| 15 | Integration build + test | verification |
