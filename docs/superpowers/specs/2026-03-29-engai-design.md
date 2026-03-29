# Engai — AI + Engineering + Knowledge Management English Learning System

## Overview

Engai is a local-first English learning system that combines AI-powered explanations, structured Markdown notes, and an Anki-style spaced repetition engine. It ships as a single binary that provides a Web UI (React), a terminal UI (ratatui), and a CLI — all backed by SQLite with bidirectional Markdown sync.

## Architecture

### Single Binary, Three Interfaces

```
engai                          # Start Web + TUI
engai -s / --server            # Web server only (default port 3000)
engai <subcommand> [args]      # CLI mode
```

The frontend static assets are compiled into the binary at build time via `rust-embed`. The Axum server serves the React SPA from memory.

### Cargo Workspace

```
engai/
├── Cargo.toml                 # workspace root
├── crates/
│   ├── engai-core/            # shared library
│   │   └── src/
│   │       ├── db.rs          # SQLite operations (sqlx)
│   │       ├── markdown.rs    # Markdown parsing/generation
│   │       ├── sync.rs        # bidirectional sync engine
│   │       ├── review.rs      # SM-2 spaced repetition
│   │       ├── ai.rs          # LLM API calls (Kimi/OpenAI)
│   │       └── models.rs      # data models
│   └── engai/                 # main binary (CLI + Server + TUI)
│       ├── build.rs           # triggers npm build, embeds frontend
│       └── src/
│           ├── main.rs        # entry: arg routing
│           ├── cli.rs         # CLI subcommands
│           ├── server.rs      # Axum web server
│           └── tui.rs         # ratatui TUI
├── frontend/                  # React + Shadcn
│   └── src/
│       ├── pages/
│       │   ├── Dashboard.tsx
│       │   ├── Vocabulary.tsx
│       │   ├── WordCard.tsx
│       │   ├── Review.tsx
│       │   ├── Reading.tsx
│       │   └── Chat.tsx
│       ├── components/
│       │   ├── ui/            # Shadcn base components
│       │   ├── Layout.tsx
│       │   ├── FlashCard.tsx
│       │   ├── FamiliarityBadge.tsx
│       │   └── ReviewCalendar.tsx
│       ├── hooks/
│       │   ├── useWebSocket.ts
│       │   └── useApi.ts
│       ├── lib/
│       │   └── api.ts
│       ├── App.tsx
│       └── main.tsx
├── docs/                      # Markdown notes
│   ├── 01_vocab/
│   ├── 02_phrases/
│   ├── 03_reading/
│   └── 99_review/
├── prompts/                   # AI prompt templates
│   ├── explain_word.md
│   ├── reading_analyze.md
│   └── chat_english.md
└── scripts/                   # automation scripts
```

### Data Flow

```
Markdown files ←→ sync engine ←→ SQLite ←→ Axum API ←→ React SPA
                                              ↕
                                         AI (Kimi/OpenAI)
                                              ↕
                                         CLI / TUI (independent AI calls)
```

AI is called independently from both the backend (Axum → engai-core::ai) and the CLI, as per user requirement.

## Data Layer

### SQLite Schema

```sql
CREATE TABLE words (
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

CREATE TABLE phrases (
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

CREATE TABLE examples (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    target_type TEXT NOT NULL,          -- "word" | "phrase"
    target_id   INTEGER NOT NULL,
    sentence    TEXT NOT NULL,
    source      TEXT
);

CREATE TABLE reviews (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    target_type TEXT NOT NULL,          -- "word" | "phrase"
    target_id   INTEGER NOT NULL,
    quality     INTEGER,
    reviewed_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE readings (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT,
    content    TEXT,
    source     TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE notes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    target_type TEXT NOT NULL,          -- "word" | "phrase" | "reading"
    target_id   INTEGER NOT NULL,
    content     TEXT,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE chat_history (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    role       TEXT,
    content    TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### Bidirectional Markdown ↔ SQLite Sync

Sync rules based on `updated_at` timestamps:

1. **Markdown → SQLite**: Parse Markdown (frontmatter + structured sections). If file mtime > DB `updated_at`, update DB.
2. **SQLite → Markdown**: If DB `updated_at` > file mtime, regenerate Markdown file.
3. **Conflict**: Both modified → keep newer, backup older as `<filename>.<timestamp>.bak`.

Markdown files include YAML frontmatter for sync metadata:

```markdown
---
word: abandon
familiarity: 3
interval: 7
next_review: 2026-04-05
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
> ...

## My Notes
- context-specific notes

## Review
- 2026-03-29 ⭐
```

### Phrase Markdown Format

```markdown
---
phrase: take off
familiarity: 2
interval: 3
next_review: 2026-04-01
synced_at: 2026-03-29T10:00:00
---

# take off

## Meaning
to remove; to become successful

## Examples
- She took off her coat.
- The business really took off this year.

## AI Explanation
> ...

## My Notes
- Two very different meanings depending on context

## Review
- 2026-03-29 ⭐
```

### Reading Markdown Format

```markdown
---
title: "The Art of Learning"
source: https://example.com/article
imported_at: 2026-03-29T10:00:00
synced_at: 2026-03-29T10:00:00
---

# The Art of Learning

## Content
[Full article text]

## Vocabulary
- abandon: to leave behind
- derive: to obtain from

## Key Sentences
- "The most important skill is learning how to learn."

## Summary (AI)
> ...

## My Notes
- relates to chapter 3 concepts
```

### Spaced Repetition (SM-2 Simplified)

- Quality 0-2: Reset familiarity to 0, interval back to 1 day.
- Quality 3: Interval unchanged, review again.
- Quality 4-5: `interval = interval × ease_factor`, ease_factor adjusted.

Interval progression: 1 → 3 → 7 → 14 → 30 days (approximate, varies by ease_factor).

## Web API

### Routes

```
# Words
GET    /api/words                    # list (pagination, search, filter by familiarity)
GET    /api/words/:word              # detail
POST   /api/words                    # create
PUT    /api/words/:word              # update
DELETE /api/words/:word              # delete
POST   /api/words/:word/explain      # AI explain (streaming SSE)

# Review
GET    /api/review/today             # today's review queue
POST   /api/review/:word             # submit review result (quality 0-5)
GET    /api/review/stats             # review statistics

# Reading
GET    /api/readings                  # list reading materials
POST   /api/readings                  # add reading material
POST   /api/readings/:id/analyze      # AI reading analysis (streaming SSE)

# AI Chat
WS     /api/chat                      # WebSocket English conversation

# Sync
POST   /api/sync                      # trigger Markdown ↔ SQLite sync

# Stats
GET    /api/stats                     # learning dashboard data

# Static (SPA fallback)
GET    /*                             # React SPA (rust-embed)
```

### WebSocket Chat Flow

```
Browser ←WS→ Axum → engai-core::ai → Kimi/OpenAI API
                ↕
          chat_history (SQLite)
```

Messages are persisted to `chat_history` in real-time.

## Frontend

### Tech Stack

| Library | Version | Purpose |
|---------|---------|---------|
| React | 19 | UI framework |
| Vite | latest | Build tool |
| Shadcn/ui | latest | Component library |
| Tailwind CSS | 4 | Styling |
| React Router | 7 | Routing |
| Recharts | latest | Charts (dashboard) |
| TanStack Query | latest | Server state management |
| lucide-react | latest | Icons |

### Pages

- **Dashboard**: Learning progress, review calendar heatmap, stats charts
- **Vocabulary**: Word list with search, filter by familiarity, bulk operations
- **WordCard**: Word detail with AI explanation, examples, notes
- **Review**: Anki-style flip cards with quality rating (0-5)
- **Reading**: Reading mode with vocabulary annotation, sentence analysis
- **Chat**: AI English conversation via WebSocket

## TUI

Built with ratatui + crossterm. Layout:

```
┌─────────────────────────────────────────────────┐
│ Engai - AI English Learning System    [Web: ✅]  │
├──────────┬──────────────────────────────────────┤
│ Vocab    │  Word detail / content area           │
│ Review   │                                       │
│ Read     │                                       │
│ Chat     │                                       │
│ Stats    │                                       │
├──────────┴──────────────────────────────────────┤
│ [a]dd  [e]xplain  [r]eview  [s]ync  [q]uit     │
└─────────────────────────────────────────────────┘
```

Left sidebar for navigation, main area for content, bottom bar for quick actions.

## CLI Commands

```
engai                              Start Web + TUI
engai -s / --server                Web server only (port configurable via -p)
engai add <word>                   Add word (Markdown + SQLite)
engai explain <word>               AI explain word, write to Markdown
engai review                       Today's review (interactive terminal)
engai review --all                 View all pending reviews
engai sync                         Bidirectional Markdown ↔ SQLite sync
engai read <file>                  Import reading material + AI analysis
engai import <dir-or-file>         Batch import Markdown notes
engai export [--word <w>|--all]    Export to Markdown
engai stats                        Learning statistics summary
engai config set <key> <value>     Set config
engai config get <key>             Get config
engai config init                  Interactive config initialization
```

## Configuration

File: `~/.engai/config.toml`

```toml
[server]
port = 3000
host = "127.0.0.1"

[ai]
provider = "kimi"
api_key = ""
model = ""
base_url = ""

[learning]
daily_new_words = 20
daily_review_limit = 100
default_deck = "01_vocab"

[storage]
db_path = "~/.engai/engai.db"
docs_path = "./docs"
```

API key can also be read from `ENGAI_AI_API_KEY` environment variable.

## AI Prompt Templates

Located in `prompts/`. Support variable interpolation: `{{word}}`, `{{level}}`, `{{context}}`.

| Template | Purpose |
|----------|---------|
| explain_word.md | Word explanation: meaning, 3 examples, synonyms comparison |
| reading_analyze.md | Reading breakdown: vocabulary, grammar, summary |
| chat_english.md | English conversation system prompt |

## Dependencies

### Rust (latest stable versions)

| Crate | Version | Purpose |
|-------|---------|---------|
| axum | 0.8.8 | Web framework |
| sqlx | 0.8.6 | SQLite async driver |
| ratatui | 0.30.0 | TUI framework |
| rust-embed | 8.11.0 | Static file embedding |
| tokio | 1.50.0 | Async runtime |
| clap | 4.6.0 | CLI argument parsing |
| serde | 1.0.228 | Serialization |
| reqwest | 0.13.2 | HTTP client (AI API) |
| crossterm | latest | Terminal control (TUI backend) |
| toml | latest | Config parsing |
| chrono | latest | Time handling |
| pulldown-cmark | latest | Markdown parsing |
| gray_matter | latest | Frontmatter parsing |
| tower-http | latest | Axum middleware (CORS, compression) |
| tracing | latest | Logging |
| anyhow | latest | Error handling |
| thiserror | latest | Custom error types |

### Frontend

- React 19, Vite, Shadcn/ui, Tailwind CSS 4, React Router 7, Recharts, TanStack Query, lucide-react

## Build Process

1. `cd frontend && npm install && npm run build` → produces `frontend/dist/`
2. `cargo build` → `build.rs` checks for `frontend/dist/`, auto-triggers npm build if missing → `rust-embed` embeds static files → single `engai` binary
3. `engai config init` → generates `~/.engai/config.toml`
4. `engai` → starts Web + TUI

## Deployment

Local machine only. Single binary + SQLite file + Markdown docs directory. No external services required beyond AI API access.

## Implementation Phases

This project is large enough to warrant phased delivery. Each phase produces a usable system.

### Phase 1: Core + CLI (Foundation)

- Cargo workspace + `engai-core` library skeleton
- SQLite schema + migrations + CRUD operations
- Markdown parsing and generation (words, phrases, readings)
- Bidirectional sync engine
- SM-2 spaced repetition algorithm
- AI integration (reqwest → Kimi/OpenAI, streaming SSE)
- Prompt templates with variable interpolation
- Configuration system (`config.toml`)
- CLI subcommands: `add`, `explain`, `review`, `sync`, `read`, `import`, `export`, `stats`, `config`
- `build.rs` for frontend embedding

**Result**: Fully functional CLI-based learning system with AI and sync.

### Phase 2: Web Server + React Frontend

- Axum server with all API routes
- Static file serving via `rust-embed`
- WebSocket chat endpoint
- React SPA: Dashboard, Vocabulary, WordCard, Review, Reading, Chat pages
- Shadcn/ui components, Tailwind CSS, TanStack Query
- Streaming AI responses in frontend (SSE for explain/analyze, WS for chat)

**Result**: Full web-based learning interface at `localhost:3000`.

### Phase 3: TUI + Polish

- ratatui TUI with sidebar navigation
- Terminal-based review, word browsing, stats
- Dual-mode launch (Web + TUI concurrently)
- Error handling, logging, edge case hardening
- Documentation and scripts
