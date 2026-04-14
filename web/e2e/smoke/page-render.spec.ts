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
  await expect(page.locator('h1, h2', { hasText: 'Readings' })).toBeVisible()
})

test('Chat renders input area', async ({ page }) => {
  await page.goto('/chat')
  await expect(page.locator('text=AI Chat')).toBeVisible()
  await expect(page.locator('input[placeholder="Type a message..."]')).toBeVisible()
})
