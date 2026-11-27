// Loading the signed-in session, as a state the screen renders (docs/conventions.md:
// view logic outside components).
import type { ApiClient } from '../api/client'

export type SessionState =
  | { kind: 'loading' }
  | {
      kind: 'signed-in'
      email: string
      firstSetupPending: boolean
      schemaVersion: number | null
      revision: string
    }
  /** The API answered with a problem (RFC 9457): not signed in, denied, or unavailable. */
  | { kind: 'refused'; status: number; title: string }
  /** No answer from the API at all. */
  | { kind: 'unreachable' }

export async function loadSession(client: ApiClient): Promise<SessionState> {
  try {
    const { data, error, response } = await client.GET('/api/session')
    if (data) {
      return {
        kind: 'signed-in',
        email: data.email,
        firstSetupPending: data.first_setup_pending,
        schemaVersion: data.schema_version,
        revision: data.revision,
      }
    }
    return {
      kind: 'refused',
      status: response.status,
      title: error?.title ?? `The server answered ${response.status}`,
    }
  } catch {
    return { kind: 'unreachable' }
  }
}
