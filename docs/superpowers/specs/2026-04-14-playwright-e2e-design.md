# Playwright E2E Testing Design

## Context

The `web/` app is a React 19 SPA using TanStack Router, Zustand, and TailwindCSS. It runs via a custom Bun dev server (`dev.ts`, port 3000) that proxies API requests to a Rust/Axum backend (`engai server`, port 9000). There are 6 routes: Dashboard(`/`), Vocabulary(`/vocabulary`), Review(`/review`), Readings(`/readings`), Chat(`/chat`), Word detail(`/words/$word`).

There are currently no tests in the project.

## Scope

- **Smoke tests**: Verify every page loads, navigation works, key UI elements are visible.
- **User flow tests**: Simulate real user operations (vocabulary browsing, review, chat, readings) against a real backend.

## Architecture

### Project Structure

```
web/
├── e2e/
│   ├── smoke/
│   │   ├── navigation.spec.ts    # Sidebar navigation across all routes
│   │   └── page-render.spec.ts   # Key element visibility per page
│   └── flows/
│       ├── vocabulary.spec.ts    # Word list, search, detail view
│       ├── review.spec.ts        # Review session flow
│       ├── chat.spec.ts          # Send message, receive reply
│       └── readings.spec.ts      # Reading list, reading detail
├── playwright.config.ts
└── package.json                  # New e2e scripts
```

### Playwright Configuration

File: `web/playwright.config.ts`

- **Browser**: Chromium only
- **webServer**: Two servers auto-started
  - Frontend: `bun run dev` (port 3000, cwd `web/`)
  - Backend: `cargo run -- server --port 9000` (cwd project root `../`)
- **headless**: Controlled by `HEADED` env var, defaults to `true`
- **baseURL**: `http://localhost:3000`
- **timeout**: 30s per test
- **retries**: 1 on CI, 0 locally

### Running Modes

| Command | Description |
|---------|-------------|
| `bun run test:e2e` | Headless mode (CI) |
| `bun run test:e2e:headed` | Headed mode, visible browser |
| `bun run test:e2e:ui` | Playwright UI mode with time-travel debugging |

Mode switching uses `HEADED=1` env var and `--ui` flag.

### Dependencies

- `@playwright/test` (devDependency)
- Chromium browser installed via `npx playwright install chromium`

## Smoke Tests

### navigation.spec.ts

Tests sidebar navigation from any page to all routes:
- Click each nav item (Dashboard, Vocabulary, Review, Reading, Chat)
- Verify URL changes correctly
- Verify active link is highlighted

### page-render.spec.ts

Verifies each page renders its key elements:
- Dashboard: stats/title area visible
- Vocabulary: word list area visible
- Review: review card area visible
- Readings: reading list visible
- Chat: chat input visible
- Handles empty states gracefully (checks for empty state message if no data)

## User Flow Tests

All flow tests run against the real backend. Pages wait for API responses before asserting. Empty data states are handled gracefully (assert empty state UI rather than failing).

### vocabulary.spec.ts

- Word list loads from API
- Search/filter words
- Click word to navigate to `/words/$word`
- Word detail page shows word info

### review.spec.ts

- Enter review mode
- Review card displays correctly
- Navigate through review cards

### chat.spec.ts

- Chat input accepts text
- Send a message
- Response appears in chat

### readings.spec.ts

- Reading list loads from API
- Click to view reading detail

## Flow Test Strategy

- `beforeEach`: Navigate to target page, wait for network idle
- Assertions check for either data-loaded state or empty-state UI
- No hard-coded test data dependencies — tests adapt to whatever the backend returns
