# Playwright E2E Testing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Playwright E2E tests to the `web/` app with smoke tests and user flow tests, supporting headless/headed/UI modes.

**Architecture:** Playwright test runner configured with dual webServer (Bun frontend + Rust backend auto-start). Tests organized into `e2e/smoke/` for page render/navigation and `e2e/flows/` for real user workflows. Mode switching via npm scripts and env vars.

**Tech Stack:** @playwright/test, Chromium only, Bun runtime for frontend, Cargo for backend.

---

### Task 1: Install Playwright and configure project

**Files:**
- Modify: `web/package.json` (add devDependency + scripts)
- Create: `web/playwright.config.ts`

- [ ] **Step 1: Install @playwright/test**

Run:
```bash
cd web && bun add -d @playwright/test
```

- [ ] **Step 2: Install Chromium browser**

Run:
```bash
cd web && npx playwright install chromium
```

- [ ] **Step 3: Create playwright.config.ts**

Create `web/playwright.config.ts`:

```typescript
import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: process.env.CI ? 'html' : 'list',
  timeout: 30_000,
  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on-first-retry',
    headless: !process.env.HEADED,
  },
  projects: [
    {
      name: 'chromium',
      use: { browserName: 'chromium' },
    },
  ],
  webServer: [
    {
      command: 'cargo run -- server --port 9000',
      port: 9000,
      cwd: '../',
      reuseExistingServer: !process.env.CI,
      timeout: 120_000,
    },
    {
      command: 'bun run dev',
      port: 3000,
      reuseExistingServer: !process.env.CI,
      timeout: 30_000,
    },
  ],
})
```

- [ ] **Step 4: Add e2e scripts to package.json**

Modify `web/package.json` scripts section to add:

```json
"test:e2e": "playwright test",
"test:e2e:headed": "HEADED=1 playwright test",
"test:e2e:ui": "HEADED=1 playwright test --ui"
```

- [ ] **Step 5: Add e2e/ to .gitignore**

Append to `web/.gitignore`:

```
e2e/results/
test-results/
playwright-report/
```

- [ ] **Step 6: Verify config loads**

Run:
```bash
cd web && npx playwright test --list 2>&1 | head -5
```

Expected: No errors (may say "no test files found" — that's fine).

- [ ] **Step 7: Commit**

```bash
git add web/package.json web/bun.lock web/playwright.config.ts web/.gitignore
git commit -m "feat(web): add Playwright config and e2e test infrastructure"
```

---

### Task 2: Create smoke test — navigation

**Files:**
- Create: `web/e2e/smoke/navigation.spec.ts`

- [ ] **Step 1: Write navigation test**

Create `web/e2e/smoke/navigation.spec.ts`:

```typescript
import { test, expect } from '@playwright/test'

const navItems = [
  { label: 'Dashboard', path: '/' },
  { label: 'Vocabulary', path: '/vocabulary' },
  { label: 'Review', path: '/review' },
  { label: 'Reading', path: '/readings' },
  { label: 'Chat', path: '/chat' },
]

for (const { label, path } of navItems) {
  test(`navigate to ${label} via sidebar`, async ({ page }) => {
    await page.goto('/')
    const link = page.locator('nav a', { hasText: label })
    await link.click()
    await expect(page).toHaveURL(new RegExp(`${path === '/' ? '/$' : path}`))
  })
}

test('active nav link is highlighted', async ({ page }) => {
  await page.goto('/vocabulary')
  const activeLink = page.locator('nav a.bg-slate-700')
  await expect(activeLink).toContainText('Vocabulary')
})
```

- [ ] **Step 2: Run the test**

Run:
```bash
cd web && bun run test:e2e -- e2e/smoke/navigation.spec.ts
```

Expected: All 6 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add web/e2e/smoke/navigation.spec.ts
git commit -m "test(web): add navigation smoke test"
```

---

### Task 3: Create smoke test — page render

**Files:**
- Create: `web/e2e/smoke/page-render.spec.ts`

- [ ] **Step 1: Write page render test**

Create `web/e2e/smoke/page-render.spec.ts`:

```typescript
import { test, expect } from '@playwright/test'

test('Dashboard renders stat cards', async ({ page }) => {
  await page.goto('/')
  await expect(page.locator('h1', { hasText: 'Dashboard' })).toBeVisible()
  await expect(page.locator('text=Words')).toBeVisible()
  await expect(page.locator('text=Phrases')).toBeVisible()
  await expect(page.locator('text=Pending Reviews')).toBeVisible()
  await expect(page.locator('text=Reviewed Today')).toBeVisible()
})

test('Vocabulary renders search and tabs', async ({ page }) => {
  await page.goto('/vocabulary')
  await expect(page.locator('h1', { hasText: 'Vocabulary' })).toBeVisible()
  await expect(page.locator('input[placeholder="Search words and phrases..."]')).toBeVisible()
  await expect(page.locator('button[role="tab"]', { hasText: /Words/ })).toBeVisible()
  await expect(page.locator('button[role="tab"]', { hasText: /Phrases/ })).toBeVisible()
})

test('Review renders review card or empty state', async ({ page }) => {
  await page.goto('/review')
  const heading = page.locator('h1, h2')
  await expect(heading.first()).toBeVisible()
})

test('Readings renders reading list area', async ({ page }) => {
  await page.goto('/readings')
  await expect(page.locator('text=Readings')).toBeVisible()
})

test('Chat renders input area', async ({ page }) => {
  await page.goto('/chat')
  await expect(page.locator('text=AI Chat')).toBeVisible()
  await expect(page.locator('input[placeholder="Type a message..."]')).toBeVisible()
})
```

- [ ] **Step 2: Run the test**

Run:
```bash
cd web && bun run test:e2e -- e2e/smoke/page-render.spec.ts
```

Expected: All 5 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add web/e2e/smoke/page-render.spec.ts
git commit -m "test(web): add page render smoke test"
```

---

### Task 4: Create flow test — vocabulary

**Files:**
- Create: `web/e2e/flows/vocabulary.spec.ts`

- [ ] **Step 1: Write vocabulary flow test**

Create `web/e2e/flows/vocabulary.spec.ts`:

```typescript
import { test, expect } from '@playwright/test'

test('word list loads and displays words or empty state', async ({ page }) => {
  await page.goto('/vocabulary')
  await page.waitForLoadState('networkidle')
  const wordTab = page.locator('[role="tab"]', { hasText: /Words/ })
  await expect(wordTab).toBeVisible()
  const body = page.locator('[role="tabpanel"]')
  await expect(body.first()).toBeVisible()
})

test('search filters words', async ({ page }) => {
  await page.goto('/vocabulary')
  await page.waitForLoadState('networkidle')
  const cards = page.locator('[role="tabpanel"] a')
  const countBefore = await cards.count()

  if (countBefore === 0) {
    const emptyMsg = page.locator('text=No words found')
    await expect(emptyMsg).toBeVisible()
    return
  }

  const firstWord = await cards.first().locator('.font-medium').textContent()
  expect(firstWord).toBeTruthy()

  await page.locator('input[placeholder="Search words and phrases..."]').fill(firstWord!)
  const filtered = page.locator('[role="tabpanel"] a')
  await expect(filtered.first()).toBeVisible()
})

test('click word navigates to detail', async ({ page }) => {
  await page.goto('/vocabulary')
  await page.waitForLoadState('networkidle')
  const firstCard = page.locator('[role="tabpanel"] a').first()

  if (!(await firstCard.isVisible())) return

  await firstCard.click()
  await expect(page).toHaveURL(/\/words\//)
  await expect(page.locator('text=Back')).toBeVisible()
})
```

- [ ] **Step 2: Run the test**

Run:
```bash
cd web && bun run test:e2e -- e2e/flows/vocabulary.spec.ts
```

Expected: All 3 tests PASS (adapt to data or empty state).

- [ ] **Step 3: Commit**

```bash
git add web/e2e/flows/vocabulary.spec.ts
git commit -m "test(web): add vocabulary flow test"
```

---

### Task 5: Create flow test — review

**Files:**
- Create: `web/e2e/flows/review.spec.ts`

- [ ] **Step 1: Write review flow test**

Create `web/e2e/flows/review.spec.ts`:

```typescript
import { test, expect } from '@playwright/test'

test('review page shows queue or empty state', async ({ page }) => {
  await page.goto('/review')
  await page.waitForLoadState('networkidle')
  const heading = page.locator('h1, h2')
  await expect(heading.first()).toBeVisible()
})

test('review card can be flipped if items exist', async ({ page }) => {
  await page.goto('/review')
  await page.waitForLoadState('networkidle')

  const noReviews = page.locator('text=No reviews due!')
  if (await noReviews.isVisible()) return

  const card = page.locator('.cursor-pointer')
  await expect(card).toBeVisible()
  await card.click()

  const ratingButtons = page.locator('button', { hasText: /Again|Hard|Difficult|Good|Easy|Perfect/ })
  await expect(ratingButtons.first()).toBeVisible()
})

test('review progress bar is visible when items exist', async ({ page }) => {
  await page.goto('/review')
  await page.waitForLoadState('networkidle')

  const noReviews = page.locator('text=No reviews due!')
  if (await noReviews.isVisible()) return

  const progressBar = page.locator('.bg-primary.h-2.rounded-full')
  await expect(progressBar).toBeVisible()
})
```

- [ ] **Step 2: Run the test**

Run:
```bash
cd web && bun run test:e2e -- e2e/flows/review.spec.ts
```

Expected: All 3 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add web/e2e/flows/review.spec.ts
git commit -m "test(web): add review flow test"
```

---

### Task 6: Create flow test — chat

**Files:**
- Create: `web/e2e/flows/chat.spec.ts`

- [ ] **Step 1: Write chat flow test**

Create `web/e2e/flows/chat.spec.ts`:

```typescript
import { test, expect } from '@playwright/test'

test('chat page shows connection status', async ({ page }) => {
  await page.goto('/chat')
  await page.waitForLoadState('networkidle')
  const status = page.locator('text=Connected').or(page.locator('text=Disconnected'))
  await expect(status).toBeVisible()
})

test('chat input is disabled when disconnected', async ({ page }) => {
  await page.goto('/chat')
  await page.waitForLoadState('networkidle')
  const disconnected = page.locator('text=Disconnected')
  if (await disconnected.isVisible()) {
    const input = page.locator('input[placeholder="Type a message..."]')
    await expect(input).toBeDisabled()
  }
})

test('chat shows empty state message', async ({ page }) => {
  await page.goto('/chat')
  await expect(page.locator('text=Start a conversation')).toBeVisible()
})
```

- [ ] **Step 2: Run the test**

Run:
```bash
cd web && bun run test:e2e -- e2e/flows/chat.spec.ts
```

Expected: All 3 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add web/e2e/flows/chat.spec.ts
git commit -m "test(web): add chat flow test"
```

---

### Task 7: Create flow test — readings

**Files:**
- Create: `web/e2e/flows/readings.spec.ts`

- [ ] **Step 1: Write readings flow test**

Create `web/e2e/flows/readings.spec.ts`:

```typescript
import { test, expect } from '@playwright/test'

test('readings page loads with list or empty state', async ({ page }) => {
  await page.goto('/readings')
  await page.waitForLoadState('networkidle')
  await expect(page.locator('text=Readings')).toBeVisible()
  const content = page.locator('button, .text-muted-foreground')
  await expect(content.first()).toBeVisible()
})

test('click reading shows detail view', async ({ page }) => {
  await page.goto('/readings')
  await page.waitForLoadState('networkidle')

  const emptyMsg = page.locator('text=No readings yet')
  if (await emptyMsg.isVisible()) return

  const firstReading = page.locator('button.text-left').first()
  await expect(firstReading).toBeVisible()
  await firstReading.click()

  const analyzeBtn = page.locator('button', { hasText: /AI Analysis|Analyzing/ })
  await expect(analyzeBtn).toBeVisible()
})

test('readings detail shows content area', async ({ page }) => {
  await page.goto('/readings')
  await page.waitForLoadState('networkidle')

  const emptyMsg = page.locator('text=No readings yet')
  if (await emptyMsg.isVisible()) return

  const firstReading = page.locator('button.text-left').first()
  await firstReading.click()

  const contentCard = page.locator('.card, [class*="card"]')
  await expect(contentCard.first()).toBeVisible()
})
```

- [ ] **Step 2: Run the test**

Run:
```bash
cd web && bun run test:e2e -- e2e/flows/readings.spec.ts
```

Expected: All 3 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add web/e2e/flows/readings.spec.ts
git commit -m "test(web): add readings flow test"
```

---

### Task 8: Run full e2e suite and verify all modes

- [ ] **Step 1: Run full test suite headless**

Run:
```bash
cd web && bun run test:e2e
```

Expected: All tests PASS (17 total: 6 navigation + 5 page-render + 3 vocab + 3 review + 3 chat + 3 readings = 17, minus the 3 that overlap with chat empty-state-only tests).

- [ ] **Step 2: Verify headed mode launches browser**

Run:
```bash
cd web && HEADED=1 npx playwright test e2e/smoke/navigation.spec.ts --reporter=list
```

Expected: Tests PASS, Chromium window visible briefly.

- [ ] **Step 3: Verify UI mode command is valid**

Run:
```bash
cd web && npx playwright test --ui --list 2>&1 | head -3
```

Expected: No errors (UI mode starts or lists tests).
