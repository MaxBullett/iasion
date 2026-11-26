// loadSession against a mocked API (ADR-0036, ADR-0043; S-014 for the refusals).
import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest'
import { setupServer } from 'msw/node'
import { createApiClient } from '../api/client'
import { problemHandler, sessionHandler, setupSession, unreachableHandler } from '../mocks/handlers'
import { loadSession } from './session'

const baseUrl = 'http://iasion.test'
const server = setupServer()
const client = createApiClient(baseUrl)

beforeAll(() => server.listen({ onUnhandledFrame: 'error' }))
afterEach(() => server.resetHandlers())
afterAll(() => server.close())

describe('loadSession', () => {
  it('returns the signed-in session', async () => {
    server.use(sessionHandler(baseUrl))

    expect(await loadSession(client)).toEqual({
      kind: 'signed-in',
      email: setupSession.email,
      firstSetupPending: true,
      schemaVersion: 20261126000004,
    })
  })

  it('keeps an unknown schema version unknown', async () => {
    server.use(sessionHandler(baseUrl, { ...setupSession, schema_version: null }))

    expect(await loadSession(client)).toMatchObject({ kind: 'signed-in', schemaVersion: null })
  })

  it('returns the problem title when the API refuses', async () => {
    server.use(
      problemHandler(baseUrl, {
        type: '/problems/access-denied',
        title: 'This sign-in may not use Iasion',
        status: 403,
      }),
    )

    expect(await loadSession(client)).toEqual({
      kind: 'refused',
      status: 403,
      title: 'This sign-in may not use Iasion',
    })
  })

  it('reports an unreachable API', async () => {
    server.use(unreachableHandler(baseUrl))

    expect(await loadSession(client)).toEqual({ kind: 'unreachable' })
  })
})
