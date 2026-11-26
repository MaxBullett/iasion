// MSW handlers typed by the generated API types (ADR-0036, decision 5). The data is
// the reference household's (fixtures/README.md), written by hand.
import { http, HttpResponse } from 'msw'
import type { Problem, Session } from '../api/client'

export const setupSession: Session = {
  email: 'first.member@example.com',
  first_setup_pending: true,
  schema_version: 20261126000004,
}

export function sessionHandler(baseUrl: string, session: Session = setupSession) {
  return http.get<never, never, Session>(`${baseUrl}/api/session`, () => HttpResponse.json(session))
}

export function problemHandler(baseUrl: string, problem: Problem) {
  return http.get<never, never, Problem>(`${baseUrl}/api/session`, () =>
    HttpResponse.json(problem, {
      status: problem.status,
      headers: { 'content-type': 'application/problem+json' },
    }),
  )
}

export function unreachableHandler(baseUrl: string) {
  return http.get(`${baseUrl}/api/session`, () => HttpResponse.error())
}
