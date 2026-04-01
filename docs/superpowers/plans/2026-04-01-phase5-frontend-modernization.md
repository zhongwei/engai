# Phase 5: Frontend Modernization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Modernize frontend to match zbooks patterns — TanStack Router, Zustand, feature modules.

**Architecture:** File-based routing with TanStack Router. Feature modules with types/api/queries separation. Zustand for client state with localStorage persistence. shadcn/ui components retained.

**Tech Stack:** React 19, TanStack Router 1.x, TanStack Query 5, Zustand 5, Vite 8, Tailwind 4, shadcn/ui

---

### Task 1: Install new dependencies

- [ ] **Step 1: Install TanStack Router + Zustand**

```bash
cd web
npm install @tanstack/react-router @tanstack/react-router-devtools zustand
```

- [ ] **Step 2: Install TanStack Router Vite plugin**

```bash
npm install -D @tanstack/router-plugin
```

- [ ] **Step 3: Remove react-router-dom**

```bash
npm uninstall react-router-dom
```

- [ ] **Step 4: Verify install**

Run: `npm ls`
Expected: no errors

- [ ] **Step 5: Commit**

```bash
git add web/package.json web/package-lock.json
git commit -m "chore: install tanstack router + zustand, remove react-router-dom"
```

---

### Task 2: Create lib/api-client.ts

**Files:**
- Create: `web/src/lib/api-client.ts`
- Create: `web/src/lib/sse-client.ts`

- [ ] **Step 1: Create api-client.ts**

```typescript
export class ApiError extends Error {
  status: number;
  data: unknown;

  constructor(status: number, data: unknown) {
    super(`API Error ${status}`);
    this.status = status;
    this.data = data;
  }
}

export async function api<T>(path: string, options?: RequestInit & { params?: Record<string, string | number | undefined> }): Promise<T> {
  let url = `/api${path}`;

  if (options?.params) {
    const searchParams = new URLSearchParams();
    Object.entries(options.params).forEach(([key, value]) => {
      if (value !== undefined) {
        searchParams.set(key, String(value));
      }
    });
    const qs = searchParams.toString();
    if (qs) url += `?${qs}`;
  }

  const init: RequestInit = { ...options };
  if (init.body && typeof init.body === 'object' && !(init.body instanceof FormData)) {
    init.headers = { 'Content-Type': 'application/json', ...init.headers };
    init.body = JSON.stringify(init.body);
  }

  const response = await fetch(url, init);

  if (!response.ok) {
    const data = await response.json().catch(() => null);
    throw new ApiError(response.status, data);
  }

  if (response.status === 204) return undefined as T;

  return response.json();
}
```

- [ ] **Step 2: Create sse-client.ts**

```typescript
export function fetchSSE(
  url: string,
  onChunk: (text: string) => void,
  onDone: () => void,
  onError: (error: Error) => void
): EventSource {
  const source = new EventSource(url);

  source.onmessage = (event) => {
    if (event.data === '[DONE]') {
      source.close();
      onDone();
      return;
    }
    onChunk(event.data);
  };

  source.onerror = () => {
    source.close();
    onError(new Error('SSE connection error'));
  };

  return source;
}
```

- [ ] **Step 3: Commit**

```bash
git add web/src/lib/
git commit -m "feat: create api-client and sse-client"
```

---

### Task 3: Create feature modules

**Files:**
- Create: `web/src/features/vocab/types.ts`
- Create: `web/src/features/vocab/api.ts`
- Create: `web/src/features/vocab/queries.ts`
- (Same pattern for phrases, review, reading, chat)

- [ ] **Step 1: Create vocab feature**

`features/vocab/types.ts`:
```typescript
export interface Word {
  id: number;
  word: string;
  phonetic: string | null;
  meaning: string;
  familiarity: number;
  next_review: string | null;
  interval: number;
  ease_factor: number;
  created_at: string;
  updated_at: string;
}

export interface CreateWord {
  word: string;
  phonetic?: string;
  meaning: string;
}

export interface WordQuery {
  search?: string;
  familiarity_gte?: number;
  limit?: number;
  offset?: number;
}
```

`features/vocab/api.ts`:
```typescript
import { api } from "@/lib/api-client";
import type { Word, CreateWord, WordQuery } from "./types";

export const listWords = (query?: WordQuery) =>
  api<Word[]>("/words", { params: query as Record<string, string | number | undefined> });

export const getWord = (word: string) =>
  api<Word>(`/words/${word}`);

export const createWord = (data: CreateWord) =>
  api<Word>("/words", { method: "POST", body: data as unknown as BodyInit });

export const deleteWord = (word: string) =>
  api<void>(`/words/${word}`, { method: "DELETE" });
```

`features/vocab/queries.ts`:
```typescript
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { listWords, getWord, createWord } from "./api";
import type { WordQuery } from "./types";

export const useWords = (query?: WordQuery) =>
  useQuery({
    queryKey: ["words", query],
    queryFn: () => listWords(query),
  });

export const useWord = (word: string) =>
  useQuery({
    queryKey: ["word", word],
    queryFn: () => getWord(word),
    enabled: !!word,
  });

export const useCreateWord = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: createWord,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["words"] }),
  });
};
```

- [ ] **Step 2: Create phrases feature**

Same pattern. Types: `Phrase`, `CreatePhrase`, `PhraseQuery`. API: `listPhrases`, `getPhrase`, `createPhrase`. Queries: `usePhrases`, `usePhrase`, `useCreatePhrase`.

- [ ] **Step 3: Create review feature**

Types: `ReviewItem`, `ReviewSubmit`, `ReviewStats`.
API: `todayReviews`, `submitReview`, `reviewStats`.
Queries: `useTodayReviews`, `useSubmitReview`, `useReviewStats`.

- [ ] **Step 4: Create reading feature**

Types: `Reading`, `NewReading`.
API: `listReadings`, `getReading`, `createReading`, `deleteReading`.
Queries: `useReadings`, `useReading`, `useCreateReading`.

- [ ] **Step 5: Create chat feature**

Types: `ChatMessage`.
API: (uses WebSocket, so minimal REST API).
Keep `useWebSocket` hook in this feature.

- [ ] **Step 6: Commit**

```bash
git add web/src/features/
git commit -m "feat: create feature modules (vocab, phrases, review, reading, chat)"
```

---

### Task 4: Create Zustand stores

**Files:**
- Create: `web/src/stores/sidebar-store.ts`
- Create: `web/src/stores/theme-store.ts`

- [ ] **Step 1: Create sidebar-store.ts**

```typescript
import { create } from "zustand";
import { persist } from "zustand/middleware";

interface SidebarState {
  expandedSections: string[];
  collapsed: boolean;
  toggleSection: (section: string) => void;
  toggleCollapsed: () => void;
}

export const useSidebarStore = create<SidebarState>()(
  persist(
    (set) => ({
      expandedSections: [],
      collapsed: false,
      toggleSection: (section) =>
        set((state) => ({
          expandedSections: state.expandedSections.includes(section)
            ? state.expandedSections.filter((s) => s !== section)
            : [...state.expandedSections, section],
        })),
      toggleCollapsed: () => set((state) => ({ collapsed: !state.collapsed })),
    }),
    { name: "engai-sidebar" }
  )
);
```

- [ ] **Step 2: Create theme-store.ts**

```typescript
import { create } from "zustand";
import { persist } from "zustand/middleware";

type Theme = "light" | "dark" | "system";

interface ThemeState {
  theme: Theme;
  setTheme: (theme: Theme) => void;
}

export const useThemeStore = create<ThemeState>()(
  persist(
    (set) => ({
      theme: "system",
      setTheme: (theme) => set({ theme }),
    }),
    { name: "engai-theme" }
  )
);
```

- [ ] **Step 3: Commit**

```bash
git add web/src/stores/
git commit -m "feat: add Zustand sidebar and theme stores"
```

---

### Task 5: Set up TanStack Router

**Files:**
- Modify: `web/vite.config.ts` — add router plugin
- Create: `web/src/routes/__root.tsx`
- Create: `web/src/routes/index.tsx`
- Create: `web/src/routes/vocabulary.index.tsx`
- Create: `web/src/routes/vocabulary.$word.tsx`
- Create: `web/src/routes/review.tsx`
- Create: `web/src/routes/readings.index.tsx`
- Create: `web/src/routes/readings.$id.tsx`
- Create: `web/src/routes/chat.tsx`

- [ ] **Step 1: Update vite.config.ts**

```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { TanStackRouterVite } from '@tanstack/router-plugin/vite'
import path from 'path'

export default defineConfig({
  plugins: [TanStackRouterVite({ autoCodeSplitting: true }), react(), tailwindcss()],
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

- [ ] **Step 2: Create __root.tsx**

```tsx
import { createRootRoute, Outlet } from "@tanstack/react-router";

export const Route = createRootRoute({
  component: () => {
    // Import Layout component with sidebar
    return (
      <div className="flex h-screen">
        {/* Sidebar */}
        {/* Main content */}
        <Outlet />
      </div>
    );
  },
});
```

- [ ] **Step 3: Create route files**

Convert each page from `pages/` into a route file:

`routes/index.tsx` — Dashboard (stats cards)
`routes/vocabulary.index.tsx` — Word/phrase list
`routes/vocabulary.$word.tsx` — Word detail + AI explain
`routes/review.tsx` — Flashcard review
`routes/readings.index.tsx` — Reading list
`routes/readings.$id.tsx` — Reading detail + AI analysis
`routes/chat.tsx` — WebSocket chat

Each route file uses `createFileRoute`:
```tsx
import { createFileRoute } from "@tanstack/react-router";
import { useWords } from "@/features/vocab/queries";

export const Route = createFileRoute("/vocabulary/")({
  component: VocabularyPage,
});

function VocabularyPage() {
  const { data: words } = useWords();
  // ... render
}
```

- [ ] **Step 4: Update main.tsx**

```tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { router } from "./routeTree.gen";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { staleTime: 5 * 60 * 1000 },
  },
});

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      {/* TanStack Router provider */}
    </QueryClientProvider>
  </StrictMode>
);
```

- [ ] **Step 5: Run code generation**

```bash
npm run dev
```

This triggers TanStack Router's file-based route generation, creating `routeTree.gen.ts`.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: set up TanStack Router with file-based routes"
```

---

### Task 6: Create layout components

**Files:**
- Create: `web/src/components/layout/Header.tsx`
- Create: `web/src/components/layout/Sidebar.tsx`
- Create: `web/src/components/layout/ThemeToggle.tsx`
- Move: `web/src/components/FlashCard.tsx` → `web/src/components/reader/FlashCard.tsx`
- Move: `web/src/components/FamiliarityBadge.tsx` → `web/src/components/reader/FamiliarityBadge.tsx`
- Move: `web/src/components/MarkdownRender.tsx` → `web/src/components/reader/MarkdownRenderer.tsx`

- [ ] **Step 1: Create layout components**

`components/layout/Sidebar.tsx` — Navigation sidebar using Zustand store for expand/collapse state
`components/layout/Header.tsx` — Top bar with app title and theme toggle
`components/layout/ThemeToggle.tsx` — Light/dark/system toggle using Zustand theme store

- [ ] **Step 2: Move reader components**

Move FlashCard, FamiliarityBadge, MarkdownRender into `components/reader/`. Rename `MarkdownRender.tsx` to `MarkdownRenderer.tsx`.

- [ ] **Step 3: Update imports in route files**

Update all references from old component paths to new paths.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: create layout components, reorganize reader components"
```

---

### Task 7: Remove old files

**Files:**
- Delete: `web/src/App.tsx`
- Delete: `web/src/pages/` directory
- Delete: `web/src/components/Layout.tsx` (replaced by layout/ components)
- Delete: `web/src/hooks/` (moved to features)
- Delete: `web/src/lib/api.ts` (replaced by api-client.ts)

- [ ] **Step 1: Remove old files**

```bash
rm web/src/App.tsx
rm -rf web/src/pages/
rm web/src/components/Layout.tsx
rm -rf web/src/hooks/
rm web/src/lib/api.ts
```

- [ ] **Step 2: Verify build**

```bash
cd web && npm run build
```
Expected: success

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "chore: remove old frontend files after migration"
```

---

### Task 8: Final verification

- [ ] **Step 1: Run full frontend build**

```bash
cd web && npm run build
```
Verify output goes to `apps/engai/static/`.

- [ ] **Step 2: Run full Rust build**

```bash
cargo build
```
Verify everything compiles.

- [ ] **Step 3: Test integrated mode**

```bash
cargo run
```
Verify server starts and serves the frontend.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore: finalize frontend modernization"
```
