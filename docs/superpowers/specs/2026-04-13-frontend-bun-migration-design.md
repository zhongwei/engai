# engai Frontend Bun Migration Design

## Overview

Migrate engai's frontend build system from Vite + npm to Bun native bundler, aligning with the zbooks project architecture. This includes switching from TanStack file-based routing to manual routing, and adopting the `app/` + `src/` directory separation pattern.

## Motivation

- Unify build tooling across projects (zbooks uses Bun native bundler)
- Faster builds with Bun's native bundler
- Simpler dev server (Bun.serve replaces Vite dev server)
- Brotli compression for production assets
- Feature-flagged static embedding (`embed-static` feature)

## Directory Structure

### New `web/app/` Directory

```
web/app/
├── app.css           # Tailwind entry CSS (moved from src/index.css)
├── entry.tsx         # React mount point (replaces src/main.tsx)
├── root.tsx          # Root layout (replaces src/routes/__root.tsx)
├── router.tsx        # Manual route definitions (replaces routeTree.gen.ts)
└── routes/           # Route page components (lazy loaded)
    ├── _index.tsx
    ├── vocabulary.tsx
    ├── review.tsx
    ├── readings.tsx
    ├── chat.tsx
    └── words.$word.tsx
```

### Unchanged `web/src/` Directory

```
web/src/
├── components/       # UI components (unchanged)
├── features/         # Feature modules (unchanged)
├── hooks/            # Custom hooks (unchanged)
├── lib/              # Utilities (unchanged)
└── stores/           # Zustand stores (unchanged)
```

### Files to Delete

- `web/vite.config.ts`
- `web/tsconfig.node.json`
- `web/tsconfig.app.json`
- `web/eslint.config.js`
- `web/src/routeTree.gen.ts`
- `web/src/routes/` (migrated to `app/routes/`)
- `web/src/main.tsx` (migrated to `app/entry.tsx`)
- `web/.tanstack/`

## Build System

### Development (`bun run dev` → `dev.ts`)

Bun.serve dev server on port 3000:
- CSS: `@tailwindcss/cli --watch` compiles `app/app.css`
- JS: On-demand Bun.build per browser request
- `/api/*` → proxy to `localhost:9000` (backend default port)
- WebSocket HMR: file watcher triggers page reload
- SPA: non-API/non-asset routes serve `index.html`

### Production (`bun run build` → `build.ts`)

1. CSS: `@tailwindcss/cli` compiles + content hashes
2. JS: `Bun.build` minifies + code splits + hashes filenames
3. Output: `../apps/engai/static/` (direct output, no intermediate copy)
4. Brotli: compress `.js` and `.css` files, remove uncompressed originals
5. Generate `index.html` with hashed asset references

## Rust Backend Changes

### `apps/engai/Cargo.toml`

- Add `embed-static` feature flag (makes `rust-embed` and `mime_guess` optional)
- Add `[profile.release]` optimization settings (lto, strip, codegen-units=1, opt-level=2, panic=abort)

### `apps/engai/build.rs`

- Only build frontend when `embed-static` feature is active
- Run `bun install` (if node_modules missing) then `bun run build`
- Build output goes directly to `static/` (build.ts handles this)

### `apps/engai/src/server.rs`

- `#[cfg(embed_static)]`: use `rust_embed` for static serving + Brotli decompression + SPA fallback
- `#[cfg(not(embed_static))]`: return JSON message prompting to start dev server
- Cache headers for hashed assets (`public, max-age=31536000, immutable`)

### `.cargo/config.toml`

Add alias: `cargo release` → `cargo run --release --features embed-static --package engai`

## Routing

Manual route definitions in `app/router.tsx`:

| Path | Component | Loading |
|------|-----------|---------|
| `/` | `_index.tsx` | lazy |
| `/vocabulary` | `vocabulary.tsx` | lazy |
| `/review` | `review.tsx` | lazy |
| `/readings` | `readings.tsx` | lazy |
| `/chat` | `chat.tsx` | lazy |
| `/words/$word` | `words.$word.tsx` | lazy |

All routes use `lazyRouteComponent` for code splitting.

## Package Changes

### Remove

- `vite`, `@vitejs/plugin-react`, `@tailwindcss/vite`
- `@tanstack/router-plugin`, `@tanstack/react-router-devtools`
- `eslint`, `eslint-plugin-react-hooks`, `eslint-plugin-react-refresh`, `typescript-eslint`, `@eslint/js`, `globals`
- `package-lock.json` → replace with `bun.lock`

### Add

- `@tailwindcss/cli`, `@tailwindcss/typography`
- `sonner` (toast notifications)
- `@playwright/test`, `@testing-library/react`, `@testing-library/jest-dom`, `happy-dom` (testing, matching zbooks)

### Scripts

- `"dev": "bun run dev.ts"`
- `"build": "bun run build.ts"`
- Remove `"preview"`, `"lint"`

## Unchanged

- Backend API routes and handlers
- TUI + Server integrated mode (default launch)
- `crates/etui` and `crates/esync`
- `web/src/components/`, `web/src/features/`, `web/src/hooks/`, `web/src/lib/`, `web/src/stores/`
- CLI subcommands
- `apps/engai/migrations/`

## Migration Order

1. Create `web/app/` directory structure with CSS, entry, root, router, routes
2. Create `web/build.ts` and `web/dev.ts`
3. Update `web/package.json` and create `web/bunfig.toml`
4. Update `web/tsconfig.json`
5. Update `web/components.json` (css path)
6. Remove old Vite/routing files
7. Update `apps/engai/Cargo.toml` (feature flag, release profile)
8. Rewrite `apps/engai/build.rs`
9. Refactor `apps/engai/src/server.rs` (conditional static serving)
10. Create `.cargo/config.toml` with release alias
11. Run `bun install` and verify dev + build
