import { test, expect } from '@playwright/test'

test('readings page loads with list or empty state', async ({ page }) => {
  await page.goto('/readings')
  await page.waitForLoadState('networkidle')
  await expect(page.locator('role=heading[name="Readings"]')).toBeVisible()
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
