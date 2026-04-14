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
