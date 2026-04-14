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
