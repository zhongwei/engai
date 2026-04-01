# Engai Architecture Alignment with ZBooks

**Date:** 2026-04-01
**Status:** Approved
**Approach:** Full alignment — mirror zbooks' architecture, patterns, and conventions

## Summary

Restructure engai to match zbooks' architecture: 3-tier backend (handlers/services/repositories), separate TUI crate as HTTP-only client, extracted sync/markdown crate, and frontend modernization with TanStack Router + Zustand.

## Decisions

| Decision | Choice |
|----------|--------|
| Scope | Full alignment with zbooks architecture |
| Schema | Preserve existing SQLite schema and `sqlx::migrate!` |
| AI features | Refactored into dedicated service layer |
| Markdown sync | Extracted to separate `esync` crate |
| TUI crate name | `etui` |
| Sync crate name | `esync` |
| Frontend | Renamed to `web/`, TanStack Router + Zustand |

## 1. Workspace & Crate Structure

### New Layout

```
engai/
├── Cargo.toml                    # Workspace root (resolver = "2")
├── apps/
│   └── engai/                    # Main binary crate
│       ├── Cargo.toml
│       ├── build.rs              # npm build → static/ embedding
│       └── src/
│           ├── main.rs           # CLI dispatch → integrated/svr/tui modes
│           ├── cli.rs            # Clap subcommands
│           ├── error.rs          # AppError enum + Result<T>
│           ├── svr.rs            # Axum server setup
│           ├── integrated.rs     # Combined mode (server + TUI)
│           ├── state.rs          # AppState struct
│           ├── models/           # Data structures & DTOs
│           │   ├── mod.rs
│           │   ├── word.rs       # Word, CreateWord, UpdateWord, WordQuery
│           │   ├── phrase.rs     # Phrase, CreatePhrase, UpdatePhrase, PhraseQuery
│           │   ├── example.rs    # Example, CreateExample
│           │   ├── review.rs     # Review, ReviewSubmit, ReviewStats
│           │   ├── reading.rs    # Reading, CreateReading, ReadingQuery
│           │   ├── note.rs       # Note, CreateNote, UpdateNote
│           │   ├── chat.rs       # ChatEntry, ChatMessage
│           │   └── query.rs      # Pagination, SearchQuery
│           ├── db/               # Database layer
│           │   ├── mod.rs
│           │   ├── pool.rs       # SqlitePool creation (WAL, max 5 conns)
│           │   ├── migrate.rs    # sqlx::migrate! (preserved)
│           │   └── repositories/
│           │       ├── mod.rs
│           │       ├── word_repository.rs
│           │       ├── phrase_repository.rs
│           │       ├── example_repository.rs
│           │       ├── review_repository.rs
│           │       ├── reading_repository.rs
│           │       ├── note_repository.rs
│           │       └── chat_repository.rs
│           ├── services/         # Business logic layer
│           │   ├── mod.rs
│           │   ├── word_service.rs
│           │   ├── phrase_service.rs
│           │   ├── review_service.rs
│           │   ├── reading_service.rs
│           │   ├── note_service.rs
│           │   ├── chat_service.rs
│           │   ├── ai_service.rs         # AI as a dedicated service
│           │   ├── stats_service.rs
│           │   └── sync_service.rs       # Delegates to esync crate
│           ├── handlers/         # Axum HTTP handlers
│           │   ├── mod.rs
│           │   ├── words.rs
│           │   ├── phrases.rs
│           │   ├── reviews.rs
│           │   ├── readings.rs
│           │   ├── notes.rs
│           │   ├── chat.rs
│           │   ├── stats.rs
│           │   └── sync.rs
│           └── cli/              # CLI subcommands
│               ├── mod.rs
│               ├── cmd_add.rs
│               ├── cmd_explain.rs
│               ├── cmd_review.rs
│               ├── cmd_sync.rs
│               ├── cmd_read.rs
│               ├── cmd_import.rs
│               ├── cmd_export.rs
│               ├── cmd_stats.rs
│               ├── cmd_config.rs
│               └── cmd_note.rs
├── crates/
│   ├── etui/                     # TUI library crate (HTTP-only client)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs         # TUI-specific config (TOML)
│   │       ├── api/              # HTTP client for backend
│   │       │   ├── mod.rs
│   │       │   ├── client.rs     # ApiClient (reqwest, 30s timeout)
│   │       │   └── models.rs     # API response DTOs
│   │       ├── tui/              # TUI framework core
│   │       │   ├── mod.rs
│   │       │   ├── app.rs        # App struct
│   │       │   ├── event.rs      # EventHandler (key/mouse/tick via mpsc)
│   │       │   ├── focus.rs      # Focus enum (Sidebar | Content)
│   │       │   └── ui.rs         # ratatui rendering
│   │       ├── panel/            # Feature panels
│   │       │   ├── mod.rs
│   │       │   ├── vocab.rs
│   │       │   ├── review.rs
│   │       │   ├── read.rs
│   │       │   ├── chat.rs
│   │       │   └── stats.rs
│   │       └── sidebar/          # Sidebar navigation
│   │           ├── mod.rs
│   │           ├── sidebar.rs
│   │           └── navigator.rs
│   └── esync/                    # Sync + markdown engine
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── sync.rs           # Bidirectional sync engine
│           ├── markdown.rs       # Markdown parser/generator
│           └── models.rs         # Sync-specific types
├── web/                          # React SPA (renamed from frontend/)
│   ├── package.json
│   ├── vite.config.ts
│   └── src/
│       ├── main.tsx
│       ├── index.css
│       ├── routeTree.gen.ts
│       ├── components/
│       │   ├── layout/           # Header, Sidebar, ThemeToggle
│       │   ├── reader/           # FlashCard, FamiliarityBadge, MarkdownRenderer
│       │   └── ui/               # shadcn components
│       ├── features/             # Feature module pattern
│       │   ├── vocab/            # types.ts, api.ts, queries.ts
│       │   ├── phrases/          # types.ts, api.ts, queries.ts
│       │   ├── review/           # types.ts, api.ts, queries.ts
│       │   ├── reading/          # types.ts, api.ts, queries.ts
│       │   └── chat/             # types.ts, api.ts, queries.ts, hooks/
│       ├── routes/               # TanStack Router file-based routes
│       │   ├── __root.tsx
│       │   ├── index.tsx
│       │   ├── vocabulary.$word.tsx
│       │   ├── review.tsx
│       │   ├── readings.tsx
│       │   ├── readings.$id.tsx
│       │   └── chat.tsx
│       ├── stores/
│       │   ├── sidebar-store.ts  # Zustand + localStorage
│       │   └── theme-store.ts    # Zustand + localStorage
│       └── lib/
│           ├── api-client.ts     # Generic fetch wrapper
│           ├── sse-client.ts     # SSE streaming helper
│           ├── use-websocket.ts  # WebSocket hook
│           └── utils.ts
├── prompts/                      # AI prompt templates (unchanged)
├── docs/                         # User notes + design docs
└── migrations/                   # SQLite migrations (preserved)
```

### Inter-Crate Dependencies

```
engai (apps/engai)
 ├── etui (crates/etui)           # TUI as HTTP-only client
 └── esync (crates/esync)         # Sync + markdown engine

etui (crates/etui)
 └── standalone (no dependency on engai)

esync (crates/esync)
 └── standalone (sqlx, gray_matter, pulldown-cmark)
```

## 2. Backend Layering (3-Tier Architecture)

### 2.1 Models Layer (`models/`)

Pure data structures with no business logic. Each entity has its own file with entity, Create, Update, and Query DTOs. Structs keep `sqlx::FromRow` and `serde::Serialize/Deserialize` derives.

### 2.2 Repositories Layer (`db/repositories/`)

Raw SQL data access only. Each repository wraps `SqlitePool` and provides CRUD operations with no validation. Mirrors zbooks' pattern of dynamic SQL construction (conditional WHERE/ORDER BY/LIMIT clauses).

`db/pool.rs` handles `SqlitePool::connect` with WAL mode, max 5 connections (preserved from current behavior). `db/migrate.rs` keeps `sqlx::migrate!` unchanged.

### 2.3 Services Layer (`services/`)

All business logic lives in services. Services are `Clone`-able for sharing via Axum state. Input validation happens here (not in handlers or repositories).

**AI Service** (`services/ai_service.rs`): Encapsulates the LLM client, prompt engine, and both batch and streaming modes. Other services call it for word/phrase explanations, reading analysis, and chat completion.

**Sync Service** (`services/sync_service.rs`): Thin wrapper that delegates to the `esync` crate for bidirectional Markdown <-> SQLite sync.

### 2.4 Handlers Layer (`handlers/`)

Axum HTTP handlers extract State/Path/Query, delegate to services, return JSON. The `AppState` struct (defined in `state.rs`) holds all services.

### 2.5 Error Handling

Replace `anyhow` with a structured `AppError` enum:

```rust
pub enum AppError {
    Database(sqlx::Error),
    NotFound(String),
    ValidationError(String),
    AiError(String),
    Internal(String),
}
```

Implements `IntoResponse` with JSON `{"error": "..."}` and appropriate HTTP status codes. Custom `Result<T>` type alias throughout.

### 2.6 Data Flow

```
Request → Handler → Service → Repository → SQLite
                      ↓
                 AI Service → LLM API (explain/chat/analyze)
                 Sync Service → esync crate (sync operations)
```

## 3. Frontend Architecture

### 3.1 Technology Stack

| Technology | Version | Purpose |
|-----------|---------|---------|
| React | 19 | UI framework |
| TypeScript | 5 | Type safety |
| Vite | 6+ | Build tool + dev server |
| TanStack Router | 1.x | File-based routing |
| TanStack Query | 5.x | Server state management |
| Zustand | 5.x | Client state management |
| Tailwind CSS | 4.x | Styling |
| shadcn/ui | -- | UI component library |
| Recharts | 3.x | Charts (retained) |

### 3.2 Feature Module Pattern

Each feature directory contains:
- **`types.ts`** — TypeScript interfaces mirroring Rust DTOs
- **`api.ts`** — API functions using the generic `api<T>()` fetch wrapper
- **`queries.ts`** — TanStack Query hooks (`useQuery`, `useMutation`) with automatic cache invalidation

Features: `vocab`, `phrases`, `review`, `reading`, `chat`.

### 3.3 TanStack Router (File-Based Routes)

```
/                              → Dashboard (stats)
/vocabulary/                   → Word/phrase list with search
/vocabulary/$word              → Word detail + AI explain (SSE)
/review                        → Anki-style flash cards
/readings/                     → Reading list
/readings/$id                  → Reading detail + AI analysis (SSE)
/chat                          → AI English conversation (WebSocket)
```

### 3.4 State Management (Zustand)

- `sidebar-store.ts` — Sidebar expanded sections, collapsed state. Persisted to localStorage.
- `theme-store.ts` — light/dark/system theme preference. Persisted to localStorage.

### 3.5 API Client

Generic `api<T>(path, options)` wrapper over `fetch`:
- Base path: `/api`
- Auto `Content-Type: application/json` for requests with body
- Custom `ApiError` class with status code
- Handles 204 No Content

Plus `fetchSSE()` for EventSource-based AI streaming and `useWebSocket` hook for real-time chat.

### 3.6 Build Integration

New `build.rs` in `apps/engai/`:
- Runs `npm install && npm run build` in `web/`
- Outputs to `apps/engai/static/`
- `rust-embed` serves these at runtime
- SPA fallback: non-API, non-file routes serve `index.html`
- Vite dev server proxies `/api` to `http://localhost:9000`

## 4. TUI Crate (`etui`)

### 4.1 Architecture

Standalone TUI library that communicates with the server exclusively via HTTP (using `reqwest`). No direct database or core library access.

Components:
- `ApiClient` — HTTP client for all backend API calls
- `App` — Main application struct integrating all TUI components
- `EventHandler` — Key/Mouse/Tick events via mpsc channel (60fps tick rate)
- `Focus` — Sidebar | Content focus state
- Panels: vocab, review, read, chat, stats
- Sidebar: section navigation with expand/collapse

### 4.2 AI Operations in TUI

For AI explain/analyze, the TUI uses synchronous HTTP calls (full response, not SSE streaming). For chat, uses the REST chat endpoint rather than WebSocket.

### 4.3 Run Modes

| Mode | Command | Description |
|------|---------|-------------|
| Integrated (default) | `engai` | Spawns HTTP server on 127.0.0.1:9000, then starts TUI |
| Server | `engai svr -p PORT --host HOST` | HTTP server only |
| TUI | `engai tui -s URL` | TUI only, connects to remote server |
| CLI | `engai add/explain/review/...` | One-off CLI commands |

### 4.4 CLI Commands

CLI commands remain in `apps/engai/src/cli/`. They use services/repositories directly (no HTTP round-trip), preserving current functionality.

## 5. Migration Strategy

### What Changes
- Directory structure: `crates/` → `apps/` + `crates/`
- `frontend/` renamed to `web/`
- `engai-core` dissolved into `apps/engai/src/` (models, db, services)
- TUI extracted from binary into `crates/etui/`
- Sync + markdown extracted into `crates/esync/`
- Frontend routing: React Router DOM → TanStack Router
- Frontend state: no client state → Zustand
- Error handling: `anyhow` → custom `AppError` enum

### What Stays
- SQLite schema and migrations (`sqlx::migrate!`)
- All API routes and their semantics
- All CLI subcommands
- AI prompt templates in `prompts/`
- User Markdown notes in `docs/`
- Core dependencies: axum, sqlx, ratatui, reqwest, serde, tokio
- SM-2 spaced repetition algorithm
- Bidirectional sync logic (moved, not rewritten)
- AI streaming (SSE + WebSocket)

### Execution Order
1. Backend restructuring (models, repositories, services, handlers)
2. Extract `esync` crate
3. Extract `etui` crate
4. Workspace reorganization (`apps/` layout, build.rs)
5. Frontend modernization (TanStack Router, Zustand, feature modules)
6. Integration testing and polish
