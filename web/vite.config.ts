import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { devToken } from './devToken.ts'

// The backend that `iasion serve` runs in development (IASION_LISTEN_ADDR).
const backend = process.env['IASION_DEV_BACKEND'] ?? 'http://localhost:8080'

// Issued on the first proxied request, not when the config loads (Vitest loads it too),
// and again after an hour: a dev token lasts 12 hours, and `./dev up` runs for days.
const tokenLifetime = 60 * 60 * 1000
let token: { value: string; issued: number } | undefined
function currentToken(): string {
  if (!token || Date.now() - token.issued > tokenLifetime) {
    token = { value: devToken(), issued: Date.now() }
  }
  return token.value
}

export default defineConfig(({ command }) => ({
  plugins: [vue()],
  server: {
    port: 5173,
    strictPort: true,
    // Where the repository's mount passes no file events (ADR-0045, decision 2).
    watch: process.env['IASION_DEV_POLL'] ? { usePolling: true, interval: 1000 } : undefined,
    proxy:
      command === 'serve'
        ? {
            // Behind Access in production, the token arrives with every request. In
            // development the proxy adds a dev token (ADR-0038, decision 9), and
            // presents the dev server's own origin as the app's origin, so that the
            // backend's Origin check (ADR-0037, decision 11) passes for this page
            // and still fails for any other.
            '/api': {
              target: backend,
              configure: (proxy) => {
                proxy.on('proxyReq', (request, incoming) => {
                  request.setHeader('cf-access-jwt-assertion', currentToken())
                  if (incoming.headers.origin === 'http://localhost:5173') {
                    request.setHeader('origin', process.env['IASION_PUBLIC_ORIGIN'] ?? '')
                  }
                })
              },
            },
          }
        : undefined,
  },
  test: {
    include: ['src/**/*.test.ts'],
    environment: 'node',
  },
}))
