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
