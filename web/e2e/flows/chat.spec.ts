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
