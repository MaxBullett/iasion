// The typed API client (ADR-0036, decision 4), generated types in ./generated.
import createClient from 'openapi-fetch'
import type { components, paths } from './generated/schema'

export type ApiClient = ReturnType<typeof createClient<paths>>
export type Session = components['schemas']['Session']
export type Problem = components['schemas']['Problem']

/**
 * A client for the API at `baseUrl`, by default the page's own origin. It looks up
 * `fetch` on each request rather than once, so that a mock installed later (MSW)
 * still intercepts it.
 */
export function createApiClient(baseUrl: string = window.location.origin): ApiClient {
  return createClient<paths>({ baseUrl, fetch: (request) => globalThis.fetch(request) })
}
