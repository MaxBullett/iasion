// A dev Access token from `cargo xtask dev-token` (ADR-0038, decision 7), for the
// Vite dev server and the end-to-end tests. IASION_DEV_TOKEN overrides it, e.g. in CI.
import { execFileSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

export function devToken(email?: string): string {
  const preset = process.env['IASION_DEV_TOKEN']
  if (preset && !email) return preset.trim()
  const args = ['xtask', 'dev-token', ...(email ? ['--email', email] : [])]
  const root = fileURLToPath(new URL('..', import.meta.url))
  return execFileSync('cargo', args, { cwd: root, encoding: 'utf8' }).trim()
}
