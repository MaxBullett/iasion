// End-to-end tests against the built backend serving the built frontend (ADR-0043,
// decision 10): Chromium and Firefox, each at desktop width and at 360 px.
import { defineConfig, devices } from '@playwright/test'
import { devToken } from './devToken.ts'

const baseURL = process.env['IASION_E2E_BASE_URL'] ?? 'http://localhost:8080'
const phone = { width: 360, height: 740 }

export default defineConfig({
  testDir: 'e2e',
  forbidOnly: !!process.env['CI'],
  reporter: process.env['CI'] ? [['list'], ['html', { open: 'never' }]] : 'list',
  use: {
    baseURL,
    // What Cloudflare Access adds to every request it lets through (ADR-0037).
    extraHTTPHeaders: { 'cf-access-jwt-assertion': devToken() },
    trace: 'retain-on-failure',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'chromium-phone', use: { ...devices['Desktop Chrome'], viewport: phone } },
    { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
    { name: 'firefox-phone', use: { ...devices['Desktop Firefox'], viewport: phone } },
  ],
  // Started unless one is running already; it needs a migrated database and
  // access-dev (README, Development).
  webServer: {
    command: 'cargo run --quiet --package iasion-server -- serve',
    cwd: '..',
    url: `${baseURL}/`,
    reuseExistingServer: !process.env['CI'],
    timeout: 180_000,
  },
})
