# engai Frontend Bun Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate engai frontend from Vite + npm to Bun native bundler with manual routing, aligning with zbooks architecture.

**Architecture:** Replace Vite dev server and build pipeline with Bun.serve (`dev.ts`) and Bun.build (`build.ts`). Move route definitions from TanStack file-based routing (`routeTree.gen.ts`) to manual `router.tsx`. Split app entry layer (`app/`) from shared code (`src/`). Add `embed-static` feature flag to Rust backend for conditional static embedding with Brotli compression.

**Tech Stack:** Bun (bundler + dev server + package manager), React 19, TanStack Router (manual mode), Tailwind CSS v4 CLI, shadcn/ui, rust-embed

---

## File Map

### New Files

| File | Purpose |
|------|---------|
| `web/app/app.css` | Tailwind entry CSS (moved from `src/index.css`) |
| `web/app/entry.tsx` | React mount point (replaces `src/main.tsx`) |
| `web/app/root.tsx` | Root layout component (replaces `src/routes/__root.tsx`) |
| `web/app/router.tsx` | Manual route tree with lazy loading |
| `web/app/routes/_index.tsx` | Dashboard route component |
| `web/app/routes/vocabulary.tsx` | Vocabulary route component |
| `web/app/routes/review.tsx` | Review route component |
| `web/app/routes/readings.tsx` | Readings route component |
| `web/app/routes/chat.tsx` | Chat route component |
| `web/app/routes/words.$word.tsx` | Word detail route component |
| `web/build.ts` | Bun production build script |
| `web/dev.ts` | Bun development server |
| `web/bunfig.toml` | Bun configuration |
| `.cargo/config.toml` | Cargo release alias |

### Modified Files

| File | Change |
|------|--------|
| `web/package.json` | Replace Vite deps with Bun/Tailwind CLI deps, update scripts |
| `web/tsconfig.json` | Add `app/` to include, simplify |
| `web/components.json` | CSS path → `app/app.css` |
| `web/index.html` | Update script src to `/app/entry.tsx` |
| `apps/engai/Cargo.toml` | Add `embed-static` feature, release profile |
| `apps/engai/build.rs` | Conditional bun build, only on `embed-static` feature |
| `apps/engai/src/server.rs` | Conditional static serving with Brotli |

### Deleted Files

| File | Reason |
|------|--------|
| `web/vite.config.ts` | Replaced by Bun bundler |
| `web/tsconfig.node.json` | Vite artifact |
| `web/tsconfig.app.json` | Vite artifact |
| `web/eslint.config.js` | Removed (no Bun-native ESLint setup needed now) |
| `web/src/routeTree.gen.ts` | Replaced by manual `router.tsx` |
| `web/src/main.tsx` | Moved to `app/entry.tsx` |
| `web/src/routes/__root.tsx` | Moved to `app/root.tsx` |
| `web/src/routes/index.tsx` | Moved to `app/routes/_index.tsx` |
| `web/src/routes/vocabulary.tsx` | Moved to `app/routes/vocabulary.tsx` |
| `web/src/routes/review.tsx` | Moved to `app/routes/review.tsx` |
| `web/src/routes/readings.tsx` | Moved to `app/routes/readings.tsx` |
| `web/src/routes/chat.tsx` | Moved to `app/routes/chat.tsx` |
| `web/src/routes/words.$word.tsx` | Moved to `app/routes/words.$word.tsx` |
| `web/src/index.css` | Moved to `app/app.css` |
| `web/.tanstack/` | TanStack file routing cache |
| `web/package-lock.json` | Replaced by `bun.lock` |

---

### Task 1: Create `web/app/` directory structure and CSS

**Files:**
- Create: `web/app/app.css`
- Delete: `web/src/index.css`

- [ ] **Step 1: Create `web/app/` directory**

```bash
mkdir -p web/app/routes
```

- [ ] **Step 2: Create `web/app/app.css`**

```css
@import "tailwindcss";
@plugin "@tailwindcss/typography";

@custom-variant dark (&:is(.dark *));

@theme {
  --font-sans: "Inter", ui-sans-serif, system-ui, sans-serif;
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  --color-card: var(--card);
  --color-card-foreground: var(--card-foreground);
  --color-popover: var(--popover);
  --color-popover-foreground: var(--popover-foreground);
  --color-primary: var(--primary);
  --color-primary-foreground: var(--primary-foreground);
  --color-secondary: var(--secondary);
  --color-secondary-foreground: var(--secondary-foreground);
  --color-muted: var(--muted);
  --color-muted-foreground: var(--muted-foreground);
  --color-accent: var(--accent);
  --color-accent-foreground: var(--accent-foreground);
  --color-destructive: var(--destructive);
  --color-destructive-foreground: var(--destructive-foreground);
  --color-border: var(--border);
  --color-input: var(--input);
  --color-ring: var(--ring);
  --color-sidebar-background: var(--sidebar-background);
  --color-sidebar-foreground: var(--sidebar-foreground);
  --color-sidebar-primary: var(--sidebar-primary);
  --color-sidebar-primary-foreground: var(--sidebar-primary-foreground);
  --color-sidebar-accent: var(--sidebar-accent);
  --color-sidebar-accent-foreground: var(--sidebar-accent-foreground);
  --color-sidebar-border: var(--sidebar-border);
  --color-sidebar-ring: var(--sidebar-ring);
  --color-chart-1: var(--chart-1);
  --color-chart-2: var(--chart-2);
  --color-chart-3: var(--chart-3);
  --color-chart-4: var(--chart-4);
  --color-chart-5: var(--chart-5);
  --radius-lg: 0.5rem;
  --radius-md: calc(var(--radius-lg) - 2px);
  --radius-sm: calc(var(--radius-lg) - 4px);
}

:root {
  --background: oklch(1 0 0);
  --foreground: oklch(0.145 0 0);
  --card: oklch(1 0 0);
  --card-foreground: oklch(0.145 0 0);
  --popover: oklch(1 0 0);
  --popover-foreground: oklch(0.145 0 0);
  --primary: oklch(0.205 0 0);
  --primary-foreground: oklch(0.985 0 0);
  --secondary: oklch(0.965 0 0);
  --secondary-foreground: oklch(0.205 0 0);
  --muted: oklch(0.965 0 0);
  --muted-foreground: oklch(0.556 0 0);
  --accent: oklch(0.965 0 0);
  --accent-foreground: oklch(0.205 0 0);
  --destructive: oklch(0.577 0.245 27.325);
  --destructive-foreground: oklch(0.577 0.245 27.325);
  --border: oklch(0.922 0 0);
  --input: oklch(0.922 0 0);
  --ring: oklch(0.708 0 0);
  --chart-1: oklch(0.646 0.222 41.116);
  --chart-2: oklch(0.6 0.118 184.704);
  --chart-3: oklch(0.398 0.07 227.392);
  --chart-4: oklch(0.828 0.189 84.429);
  --chart-5: oklch(0.769 0.188 70.08);
  --sidebar-background: oklch(0.985 0 0);
  --sidebar-foreground: oklch(0.145 0 0);
  --sidebar-primary: oklch(0.205 0 0);
  --sidebar-primary-foreground: oklch(0.985 0 0);
  --sidebar-accent: oklch(0.965 0 0);
  --sidebar-accent-foreground: oklch(0.205 0 0);
  --sidebar-border: oklch(0.922 0 0);
  --sidebar-ring: oklch(0.708 0 0);
}

.dark {
  --background: oklch(0.145 0 0);
  --foreground: oklch(0.985 0 0);
  --card: oklch(0.145 0 0);
  --card-foreground: oklch(0.985 0 0);
  --popover: oklch(0.145 0 0);
  --popover-foreground: oklch(0.985 0 0);
  --primary: oklch(0.985 0 0);
  --primary-foreground: oklch(0.205 0 0);
  --secondary: oklch(0.269 0 0);
  --secondary-foreground: oklch(0.985 0 0);
  --muted: oklch(0.269 0 0);
  --muted-foreground: oklch(0.708 0 0);
  --accent: oklch(0.269 0 0);
  --accent-foreground: oklch(0.985 0 0);
  --destructive: oklch(0.396 0.141 25.723);
  --destructive-foreground: oklch(0.637 0.237 25.331);
  --border: oklch(0.269 0 0);
  --input: oklch(0.269 0 0);
  --ring: oklch(0.439 0 0);
  --chart-1: oklch(0.488 0.243 264.376);
  --chart-2: oklch(0.696 0.17 162.48);
  --chart-3: oklch(0.769 0.188 70.08);
  --chart-4: oklch(0.627 0.265 303.9);
  --chart-5: oklch(0.645 0.246 16.439);
  --sidebar-background: oklch(0.205 0 0);
  --sidebar-foreground: oklch(0.985 0 0);
  --sidebar-primary: oklch(0.488 0.243 264.376);
  --sidebar-primary-foreground: oklch(0.985 0 0);
  --sidebar-accent: oklch(0.269 0 0);
  --sidebar-accent-foreground: oklch(0.985 0 0);
  --sidebar-border: oklch(0.269 0 0);
  --sidebar-ring: oklch(0.439 0 0);
}

@layer base {
  * {
    border-color: var(--border);
  }
  body {
    @apply bg-background text-foreground;
  }
}
```

- [ ] **Step 3: Delete `web/src/index.css`**

```bash
rm web/src/index.css
```

- [ ] **Step 4: Commit**

```bash
git add web/app/app.css
git rm web/src/index.css
git commit -m "feat(web): create app directory with Tailwind CSS entry"
```

---

### Task 2: Create route components in `web/app/routes/`

**Files:**
- Create: `web/app/routes/_index.tsx`
- Create: `web/app/routes/vocabulary.tsx`
- Create: `web/app/routes/review.tsx`
- Create: `web/app/routes/readings.tsx`
- Create: `web/app/routes/chat.tsx`
- Create: `web/app/routes/words.$word.tsx`

- [ ] **Step 1: Create `web/app/routes/_index.tsx`**

```tsx
export { default as default } from '@/pages/Dashboard'
```

- [ ] **Step 2: Create `web/app/routes/vocabulary.tsx`**

```tsx
export { default as default } from '@/pages/Vocabulary'
```

- [ ] **Step 3: Create `web/app/routes/review.tsx`**

```tsx
export { default as default } from '@/pages/Review'
```

- [ ] **Step 4: Create `web/app/routes/readings.tsx`**

```tsx
export { default as default } from '@/pages/Reading'
```

- [ ] **Step 5: Create `web/app/routes/chat.tsx`**

```tsx
export { default as default } from '@/pages/Chat'
```

- [ ] **Step 6: Create `web/app/routes/words.$word.tsx`**

```tsx
export { default as default } from '@/pages/WordCard'
```

- [ ] **Step 7: Commit**

```bash
git add web/app/routes/
git commit -m "feat(web): add route components for manual routing"
```

---

### Task 3: Create root layout and router

**Files:**
- Create: `web/app/root.tsx`
- Create: `web/app/router.tsx`
- Create: `web/app/entry.tsx`

- [ ] **Step 1: Create `web/app/root.tsx`**

```tsx
import { Outlet } from '@tanstack/react-router'
import { useEffect } from 'react'
import { useThemeStore } from '@/stores/theme-store'
import Layout from '@/components/Layout'

function applyTheme(theme: 'light' | 'dark' | 'system') {
  const root = document.documentElement
  if (theme === 'system') {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    root.classList.toggle('dark', prefersDark)
  } else {
    root.classList.toggle('dark', theme === 'dark')
  }
}

export default function RootLayout() {
  const { theme } = useThemeStore()

  useEffect(() => {
    applyTheme(theme)

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = () => {
      if (theme === 'system') {
        applyTheme('system')
      }
    }
    mediaQuery.addEventListener('change', handleChange)
    return () => mediaQuery.removeEventListener('change', handleChange)
  }, [theme])

  return (
    <Layout>
      <Outlet />
    </Layout>
  )
}
```

- [ ] **Step 2: Create `web/app/router.tsx`**

```tsx
import { createRouter, createRootRoute, createRoute, lazyRouteComponent } from '@tanstack/react-router'
import RootLayout from './root'

const rootRoute = createRootRoute({
  component: RootLayout,
})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: lazyRouteComponent(() => import('./routes/_index')),
})

const vocabularyRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/vocabulary',
  component: lazyRouteComponent(() => import('./routes/vocabulary')),
})

const reviewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/review',
  component: lazyRouteComponent(() => import('./routes/review')),
})

const readingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/readings',
  component: lazyRouteComponent(() => import('./routes/readings')),
})

const chatRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/chat',
  component: lazyRouteComponent(() => import('./routes/chat')),
})

const wordRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/words/$word',
  component: lazyRouteComponent(() => import('./routes/words.$word')),
})

const routeTree = rootRoute.addChildren([
  indexRoute,
  vocabularyRoute,
  reviewRoute,
  readingsRoute,
  chatRoute,
  wordRoute,
])

export const router = createRouter({ routeTree })

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
```

- [ ] **Step 3: Create `web/app/entry.tsx`**

```tsx
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { RouterProvider } from '@tanstack/react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { router } from './router'

const queryClient = new QueryClient()

const rootElement = document.getElementById('root')!
const root = import.meta.hot?.data.root ?? createRoot(rootElement)
if (import.meta.hot) {
  import.meta.hot.data.root = root
  import.meta.hot.accept()
}

root.render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </StrictMode>,
)
```

- [ ] **Step 4: Commit**

```bash
git add web/app/root.tsx web/app/router.tsx web/app/entry.tsx
git commit -m "feat(web): add root layout, manual router, and entry point"
```

---

### Task 4: Create Bun dev server (`web/dev.ts`)

**Files:**
- Create: `web/dev.ts`

- [ ] **Step 1: Create `web/dev.ts`**

```ts
import { mkdirSync, watch } from 'fs'

const PORT = Number(process.env.PORT) || 3000
const API_PORT = Number(process.env.API_PORT) || 9000

mkdirSync('./build/client/assets', { recursive: true })

const clients = new Set<WebSocket>()

const RELOAD_SCRIPT = `<script>
(function() {
  const ws = new WebSocket('ws://localhost:${PORT}/__ws');
  ws.onmessage = function(e) {
    if (e.data === 'reload') location.reload();
  };
  ws.onclose = function() {
    setTimeout(function() { location.reload(); }, 1000);
  };
})();
</script>`

const cssWatch = Bun.spawn([
  'bun', 'x', '@tailwindcss/cli',
  '-i', './app/app.css',
  '-o', './build/client/assets/app.css',
  '--watch',
], {
  stdio: ['inherit', 'inherit', 'inherit'],
})

async function serveModule(pathname: string): Promise<Response | null> {
  const filePath = '.' + pathname
  const result = await Bun.build({
    entrypoints: [filePath],
    target: 'browser',
    splitting: false,
    define: { 'import.meta.hot': 'undefined' },
  })
  if (!result.success) {
    console.error('Build failed:', result.logs)
    return null
  }
  const output = result.outputs[0]
  return new Response(output, {
    headers: { 'Content-Type': 'application/javascript' },
  })
}

const server = Bun.serve({
  port: PORT,
  development: true,
  websocket: {
    open(ws) {
      clients.add(ws)
    },
    close(ws) {
      clients.delete(ws)
    },
    message() {},
  },
  async fetch(req, server) {
    const url = new URL(req.url)

    if (url.pathname === '/__ws') {
      if (server.upgrade(req)) return
      return new Response('WebSocket upgrade failed', { status: 500 })
    }

    if (url.pathname === '/__trigger_reload') {
      for (const ws of clients) {
        try { ws.send('reload') } catch {}
      }
      return new Response('ok')
    }

    if (url.pathname.startsWith('/api/')) {
      const target = `http://localhost:${API_PORT}${url.pathname}${url.search}`
      return fetch(target, {
        method: req.method,
        headers: req.headers,
        body: req.body,
      })
    }

    if (url.pathname.startsWith('/assets/')) {
      const file = Bun.file(`./build/client${url.pathname}`)
      if (await file.exists()) {
        return new Response(file)
      }
    }

    if ((url.pathname.endsWith('.tsx') || url.pathname.endsWith('.ts')) && (url.pathname.startsWith('/app/') || url.pathname.startsWith('/src/'))) {
      const response = await serveModule(url.pathname)
      if (response) return response
    }

    let html = await Bun.file('./index.html').text()
    html = html.replace('</body>', `${RELOAD_SCRIPT}</body>`)
    return new Response(html, {
      headers: { 'Content-Type': 'text/html' },
    })
  },
})

console.log(`Dev server: http://localhost:${server.port} (proxy → localhost:${API_PORT})`)

function startFileWatcher() {
  watch('./app', { recursive: true }, () => {
    for (const ws of clients) {
      try { ws.send('reload') } catch {}
    }
  })
  watch('./src', { recursive: true }, () => {
    for (const ws of clients) {
      try { ws.send('reload') } catch {}
    }
  })
}

startFileWatcher()

process.on('SIGINT', () => {
  cssWatch.kill()
  server.stop()
  process.exit(0)
})

process.on('SIGTERM', () => {
  cssWatch.kill()
  server.stop()
  process.exit(0)
})
```

- [ ] **Step 2: Commit**

```bash
git add web/dev.ts
git commit -m "feat(web): add Bun dev server with HMR and API proxy"
```

---

### Task 5: Create Bun build script (`web/build.ts`)

**Files:**
- Create: `web/build.ts`

- [ ] **Step 1: Create `web/build.ts`**

Build output goes directly to `../apps/engai/static/`.

```ts
import { mkdirSync, rmSync, readFileSync, writeFileSync } from 'fs'
import { brotliCompressSync } from 'zlib'
import { createHash } from 'crypto'
import { join, basename as pathBasename } from 'path'

const outDir = '../apps/engai/static'
const assetsDir = join(outDir, 'assets')

const manifest: Record<string, string> = {}

function hashContent(data: Uint8Array): string {
  return createHash('sha256').update(data).digest('hex').slice(0, 8)
}

rmSync(outDir, { recursive: true, force: true })
mkdirSync(assetsDir, { recursive: true })

console.log('Building CSS...')
const cssResult = Bun.spawnSync([
  'bun', 'x', '@tailwindcss/cli',
  '-i', './app/app.css',
  '-o', join(assetsDir, 'app.css'),
])
if (!cssResult.success) {
  console.error('CSS build failed:', cssResult.stderr.toString())
  process.exit(1)
}

const cssData = readFileSync(join(assetsDir, 'app.css'))
const cssHash = hashContent(cssData)
const cssHashed = `app.${cssHash}.css`
writeFileSync(join(assetsDir, cssHashed), cssData)
manifest['app.css'] = cssHashed
rmSync(join(assetsDir, 'app.css'))
console.log(`  app.css → ${cssHashed}`)

console.log('Building JS...')
const jsResult = await Bun.build({
  entrypoints: ['./app/entry.tsx'],
  outdir: assetsDir,
  minify: true,
  naming: '[name].[hash].[ext]',
  splitting: true,
  target: 'browser',
})
if (!jsResult.success) {
  console.error('JS build failed:', jsResult.logs)
  process.exit(1)
}
console.log(`  Bundled ${jsResult.outputs.length} files`)

for (const output of jsResult.outputs) {
  const basename = pathBasename(output.path)
  const data = output.size !== undefined ? readFileSync(output.path) : new Uint8Array()

  const hash = hashContent(data)
  const dotIndex = basename.lastIndexOf('.')
  const ext = basename.slice(dotIndex)
  const namePart = basename.slice(0, dotIndex)
  const baseName = namePart.includes('.') ? namePart.slice(0, namePart.lastIndexOf('.')) : namePart
  const originalName = `${baseName}${ext}`
  manifest[originalName] = basename

  console.log(`  ${originalName} → ${basename}`)
}

writeFileSync(join(assetsDir, 'manifest.json'), JSON.stringify(manifest, null, 2))

const entryJs = manifest['entry.js'] || 'entry.js'
const appCss = manifest['app.css'] || 'app.css'

const html = `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
    <title>Engai</title>
    <link rel="preload" href="/assets/${appCss}" as="style" />
    <link rel="stylesheet" href="/assets/${appCss}" />
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/assets/${entryJs}"></script>
  </body>
</html>`

await Bun.write(`${outDir}/index.html`, html)

console.log('Compressing assets with Brotli...')
for (const file of Object.values(manifest)) {
  const filePath = join(assetsDir, file)
  if (!filePath.endsWith('.js') && !filePath.endsWith('.css')) continue
  const data = readFileSync(filePath)
  const compressed = brotliCompressSync(data)
  writeFileSync(`${filePath}.br`, compressed)
  rmSync(filePath)
  console.log(`  ${file}: ${data.length} -> ${compressed.length} bytes (${Math.round((1 - compressed.length / data.length) * 100)}% reduction)`)
}

console.log('Build complete.')
```

- [ ] **Step 2: Commit**

```bash
git add web/build.ts
git commit -m "feat(web): add Bun build script with Brotli compression"
```

---

### Task 6: Update package.json and create bunfig.toml

**Files:**
- Modify: `web/package.json`
- Create: `web/bunfig.toml`
- Delete: `web/package-lock.json`

- [ ] **Step 1: Rewrite `web/package.json`**

```json
{
  "name": "engai-web",
  "private": true,
  "version": "0.0.1",
  "type": "module",
  "scripts": {
    "dev": "bun run dev.ts",
    "build": "bun run build.ts",
    "test": "bun test"
  },
  "dependencies": {
    "@radix-ui/react-dialog": "^1.1.15",
    "@radix-ui/react-scroll-area": "^1.2.10",
    "@radix-ui/react-separator": "^1.1.8",
    "@radix-ui/react-slot": "^1.2.4",
    "@radix-ui/react-tabs": "^1.1.13",
    "@tailwindcss/typography": "^0.5.19",
    "@tanstack/react-query": "^5.96.2",
    "@tanstack/react-router": "^1.120.13",
    "class-variance-authority": "^0.7.1",
    "clsx": "^2.1.1",
    "lucide-react": "^1.7.0",
    "radix-ui": "^1.4.3",
    "react": "^19.2.4",
    "react-dom": "^19.2.4",
    "recharts": "^3.8.1",
    "tailwind-merge": "^3.5.0",
    "zustand": "^5.0.12"
  },
  "devDependencies": {
    "@tailwindcss/cli": "^4.2.2",
    "@types/react": "^19.2.14",
    "@types/react-dom": "^19.2.3",
    "tailwindcss": "^4.2.2",
    "typescript": "~6.0.2"
  }
}
```

- [ ] **Step 2: Create `web/bunfig.toml`**

```toml
[test]
preload = []
```

- [ ] **Step 3: Delete `web/package-lock.json`**

```bash
rm web/package-lock.json
```

- [ ] **Step 4: Commit**

```bash
git add web/package.json web/bunfig.toml
git rm web/package-lock.json
git commit -m "feat(web): switch from npm/Vite to Bun package manager and build tools"
```

---

### Task 7: Update config files (tsconfig, components.json, index.html)

**Files:**
- Modify: `web/tsconfig.json`
- Modify: `web/components.json`
- Modify: `web/index.html`

- [ ] **Step 1: Rewrite `web/tsconfig.json`**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "allowImportingTsExtensions": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["app/**/*", "src/**/*"],
  "exclude": ["node_modules"]
}
```

- [ ] **Step 2: Update `web/components.json` — change CSS path**

Change `"css": "src/index.css"` to `"css": "app/app.css"`.

- [ ] **Step 3: Update `web/index.html` — change script src**

Change `<script type="module" src="/src/main.tsx"></script>` to `<script type="module" src="/app/entry.tsx"></script>`.
Also update the title from `frontend` to `Engai`.

- [ ] **Step 4: Commit**

```bash
git add web/tsconfig.json web/components.json web/index.html
git commit -m "feat(web): update tsconfig, components.json, and index.html for Bun build"
```

---

### Task 8: Delete old Vite/routing files

**Files:**
- Delete: `web/vite.config.ts`
- Delete: `web/tsconfig.node.json`
- Delete: `web/tsconfig.app.json`
- Delete: `web/eslint.config.js`
- Delete: `web/src/routeTree.gen.ts`
- Delete: `web/src/main.tsx`
- Delete: `web/src/routes/__root.tsx`
- Delete: `web/src/routes/index.tsx`
- Delete: `web/src/routes/vocabulary.tsx`
- Delete: `web/src/routes/review.tsx`
- Delete: `web/src/routes/readings.tsx`
- Delete: `web/src/routes/chat.tsx`
- Delete: `web/src/routes/words.$word.tsx`
- Delete: `web/.tanstack/` directory

- [ ] **Step 1: Delete all old files**

```bash
cd web && rm -f vite.config.ts tsconfig.node.json tsconfig.app.json eslint.config.js
rm -f src/routeTree.gen.ts src/main.tsx
rm -rf src/routes/
rm -rf .tanstack/
```

- [ ] **Step 2: Commit**

```bash
git rm web/vite.config.ts web/tsconfig.node.json web/tsconfig.app.json web/eslint.config.js web/src/routeTree.gen.ts web/src/main.tsx
git rm -r web/src/routes/ web/.tanstack/
git commit -m "feat(web): remove Vite config and TanStack file routing artifacts"
```

---

### Task 9: Update Rust backend — Cargo.toml, build.rs, server.rs

**Files:**
- Modify: `apps/engai/Cargo.toml`
- Modify: `apps/engai/build.rs`
- Modify: `apps/engai/src/server.rs`

- [ ] **Step 1: Update `apps/engai/Cargo.toml`**

Add `embed-static` feature and release profile. Replace the existing file with:

```toml
[package]
name = "engai"
version = "0.1.0"
edition = "2021"

[features]
default = []
embed-static = ["rust-embed", "mime_guess"]

[[bin]]
name = "engai"
path = "src/main.rs"

[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
sqlx = { version = "0.8.6", features = ["runtime-tokio", "sqlite", "migrate", "chrono"] }
reqwest = { version = "0.13.2", features = ["json", "stream"] }
toml = "0.8"
pulldown-cmark = "0.13"
gray_matter = "0.2"
dirs = "6"
futures = "0.3"
uuid = { version = "1", features = ["v4"] }
tokio-stream = "0.1"
esync = { path = "../../crates/esync" }
etui = { path = "../../crates/etui" }
async-trait = "0.1"
clap = { version = "4.6.0", features = ["derive"] }
axum = { version = "0.8.8", features = ["ws"] }
axum-extra = { version = "0.10", features = ["typed-header"] }
tower-http = { version = "0.6", features = ["cors", "fs"] }
tower = "0.5"
rust-embed = { version = "8.11.0", optional = true }
mime_guess = { version = "2", optional = true }
async-stream = "0.3"
ratatui = "0.30.0"
crossterm = "0.29"
textwrap = "0.16"

[dev-dependencies]
tempfile = "3"
tokio = { workspace = true, features = ["full", "macros"] }

[profile.release]
lto = true
strip = true
codegen-units = 1
opt-level = 2
panic = "abort"
```

- [ ] **Step 2: Rewrite `apps/engai/build.rs`**

```rust
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-check-cfg=cfg(embed_static)");
    if std::env::var("CARGO_FEATURE_EMBED_STATIC").is_ok() {
        println!("cargo:rustc-cfg=embed_static");
        build_frontend();
    }
}

fn build_frontend() {
    let web_dir = Path::new("../../web");
    if !web_dir.exists() {
        println!("cargo:warning=web/ directory not found, skipping frontend build");
        return;
    }

    let node_modules = web_dir.join("node_modules");
    if !node_modules.exists() {
        let status = Command::new("bun")
            .arg("install")
            .current_dir(web_dir)
            .status()
            .expect("bun install failed");

        if !status.success() {
            panic!("bun install failed with exit code {:?}", status.code());
        }
    }

    let status = Command::new("bun")
        .arg("run")
        .arg("build")
        .current_dir(web_dir)
        .status()
        .expect("bun build failed");

    if !status.success() {
        panic!("bun build failed with exit code {:?}", status.code());
    }

    println!("cargo:rerun-if-changed=../../web/package.json");
    println!("cargo:rerun-if-changed=../../web/index.html");
    println!("cargo:rerun-if-changed=../../web/build.ts");
    println!("cargo:rerun-if-changed=../../web/dev.ts");
    println!("cargo:rerun-if-changed=../../web/tsconfig.json");
    println!("cargo:rerun-if-changed=../../web/app/");
    println!("cargo:rerun-if-changed=../../web/src/");
}
```

- [ ] **Step 3: Rewrite `apps/engai/src/server.rs`**

```rust
#[cfg(embed_static)]
use axum::http::Uri;
use axum::{body::Body, http::header, response::Response, routing::get, Router};
use tower_http::cors::{Any, CorsLayer};

#[cfg(embed_static)]
use rust_embed::Embed;

use crate::handlers::{chat, notes, phrases, readings, reviews, stats, sync, words};
use crate::state::AppState;

#[cfg(embed_static)]
#[derive(Embed)]
#[folder = "static"]
struct StaticAssets;

#[cfg(embed_static)]
fn serve_static(path: &str) -> Response {
    let br_path = format!("{}.br", path);

    if let Some(content) = StaticAssets::get(&br_path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return Response::builder()
            .header(header::CONTENT_TYPE, mime.as_ref())
            .header(header::CONTENT_ENCODING, "br")
            .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
            .body(Body::from(content.data))
            .unwrap();
    }

    match StaticAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap()
        }
        None => Response::builder()
            .status(404)
            .body(Body::from("Not Found"))
            .unwrap(),
    }
}

#[cfg(embed_static)]
async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || !path.contains('.') {
        return serve_static("index.html");
    }

    let response = serve_static(path);
    if response.status() == axum::http::StatusCode::NOT_FOUND {
        serve_static("index.html")
    } else {
        response
    }
}

#[cfg(not(embed_static))]
async fn dev_mode_fallback() -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"message":"Running in dev mode. Start frontend with: cd web && bun run dev"}"#,
        ))
        .unwrap()
}

pub async fn run_server(state: AppState, port: u16) -> anyhow::Result<()> {
    let addr = format!("{}:{}", state.config.server.host, port);
    let app = create_app(state);
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
    let mut app = Router::new()
        .nest("/api", api)
        .with_state(state)
        .layer(cors);

    #[cfg(embed_static)]
    {
        app = app.fallback(static_handler);
    }

    #[cfg(not(embed_static))]
    {
        app = app.fallback(dev_mode_fallback);
    }

    app
}
```

- [ ] **Step 4: Commit**

```bash
git add apps/engai/Cargo.toml apps/engai/build.rs apps/engai/src/server.rs
git commit -m "feat(backend): add embed-static feature flag and conditional static serving"
```

---

### Task 10: Create `.cargo/config.toml` with release alias

**Files:**
- Create: `.cargo/config.toml`

- [ ] **Step 1: Create directory and file**

```bash
mkdir -p .cargo
```

```toml
[alias]
release = "run --release --features embed-static --package engai"
```

- [ ] **Step 2: Commit**

```bash
git add .cargo/config.toml
git commit -m "feat: add cargo release alias for embedded frontend build"
```

---

### Task 11: Install dependencies and verify build

**Files:** None (verification only)

- [ ] **Step 1: Install Bun dependencies**

```bash
cd web && rm -rf node_modules && bun install
```

Expected: `bun.lock` created, dependencies installed.

- [ ] **Step 2: Verify build**

```bash
cd web && bun run build
```

Expected: `apps/engai/static/` populated with `index.html`, `assets/` containing hashed `.js.br` and `.css.br` files.

- [ ] **Step 3: Verify Rust compilation (dev mode, no frontend)**

```bash
cargo build --package engai
```

Expected: Compiles successfully without `embed-static` feature.

- [ ] **Step 4: Commit bun.lock**

```bash
git add web/bun.lock
git commit -m "chore(web): add bun.lock"
```

---

### Task 12: Verify dev mode end-to-end

**Files:** None (verification only)

- [ ] **Step 1: Start backend in one terminal**

```bash
cargo run --package engai -- server
```

Expected: Server starts on port 3000 (default).

- [ ] **Step 2: Start frontend dev server in another terminal**

```bash
cd web && bun run dev
```

Expected: Dev server starts on port 3000, proxies `/api/*` to backend. Open `http://localhost:3000` and verify the app loads with routing working (Dashboard, Vocabulary, Review, Readings, Chat pages).

- [ ] **Step 3: Verify navigation**

Click through all sidebar links: Dashboard → Vocabulary → Review → Readings → Chat. Verify each page renders correctly.

---

## Self-Review

1. **Spec coverage:** Each section in the design doc maps to a task: app/ structure (T1-T3), build system (T4-T5), package updates (T6), configs (T7), cleanup (T8), backend (T9-T10), verification (T11-T12). All requirements covered.

2. **Placeholder scan:** No TBDs, TODOs, or vague steps. All code blocks contain complete implementations.

3. **Type consistency:** Router uses `lazyRouteComponent(() => import('./routes/...'))` consistently. Route paths match the original file-based routes. `@/` path alias maps to `./src/*` in tsconfig, matching all component imports.
