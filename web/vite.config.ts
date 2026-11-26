import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { devToken } from './devToken.ts'

// The backend that `iasion serve` runs in development (IASION_LISTEN_ADDR).
const backend = process.env['IASION_DEV_BACKEND'] ?? 'http://localhost:8080'

// Issued on the first proxied request, not when the config loads (Vitest loads it too).
let token: string | undefined

export default defineConfig(({ command }) => ({
  plugins: [vue()],
  server: {
    port: 5173,
    strictPort: true,
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
                  token ??= devToken()
                  request.setHeader('cf-access-jwt-assertion', token)
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
