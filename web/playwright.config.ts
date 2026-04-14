import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: process.env.CI ? 'list' : 'html',
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
