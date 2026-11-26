// The walking skeleton's round trip, browser to database and back, behind the Access
// check (S-014, ADR-0037, ADR-0043).
import { expect, test, type Page } from '@playwright/test'
import { devToken } from '../devToken.ts'

/** Console errors and uncaught exceptions, which include violations of the CSP. */
function collectErrors(page: Page): string[] {
  const errors: string[] = []
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })
  page.on('pageerror', (error) => errors.push(error.message))
  return errors
}

test('shows who is signed in and the schema version from the database', async ({ page }) => {
  const errors = collectErrors(page)

  await page.goto('/')

  await expect(page.getByRole('heading', { level: 1, name: 'Iasion' })).toBeVisible()
  await expect(page.getByText('Signed in as first.member@example.com')).toBeVisible()
  await expect(page.getByText('No household exists yet. First setup comes next.')).toBeVisible()
  await expect(page.getByRole('definition')).toHaveText(/^\d{14}$/)
  expect(errors).toEqual([])
})

test('shows why another email is refused before first setup', async ({ page }) => {
  await page.setExtraHTTPHeaders({
    'cf-access-jwt-assertion': devToken('alex@example.com'),
  })

  await page.goto('/')

  await expect(page.getByRole('alert')).toHaveText('This sign-in may not use Iasion')
  await expect(page.getByText(/Signed in as/)).toHaveCount(0)
})

test('fits the width of the window', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByText(/Signed in as/)).toBeVisible()

  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  )
  expect(overflow).toBe(0)
})
