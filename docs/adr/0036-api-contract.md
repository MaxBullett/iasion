# 0036: API contract: OpenAPI generated from the code, a typed client, both committed

- Status: accepted
- Date: 2026-11-18
- Refines: ADR-0003, ADR-0035
- Refined by: ADR-0037, ADR-0043

## Context

ADR-0003 generates the frontend's API client from the backend's API description so
types cannot drift. The API contract enabler in `docs/releases/r1.md` left the tooling
open, and `docs/conventions.md` left the frontend's mocking tool open. ADR-0001 says
specs are never generated from code. In review, the maintainer stated what ADR-0001
means: specs in the product sense, such as stories and ADRs, whose purpose is to plan
work before it is implemented. The backend uses axum (ADR-0035).

## Decision

### Where the description comes from

1. **Code-first.** The OpenAPI description is generated from the backend code with
   utoipa and utoipa-axum, as OpenAPI 3.1. It is the code's interface, a generated
   artifact, not a spec in the sense of ADR-0001. The specs that plan the API are the
   stories, ADRs and glossary.
2. **The description is committed** as `crates/api/openapi.json`. A command
   regenerates it, and CI fails when the committed file differs from what the code
   generates. Every API change is visible in review.

### Frontend

3. **Types are generated with openapi-typescript** into `web/src/api/generated/` and
   committed. CI regenerates them and fails on a difference, so backend and client
   change in the same commit.
4. **The client is openapi-fetch,** typed by those generated types.
5. **Mocks are MSW handlers** typed by the same generated types. Their fixture data is
   written by hand from the reference household (`fixtures/README.md`), never random.

### Style

6. **JSON over HTTP.** Queries are GET. Commands are POST to resource paths.
7. **Corrections and voids carry the version they replace** (ADR-0031).
8. **The acting member never comes from the request body.** It comes from the verified
   Access identity (ADR-0024, ADR-0028).
9. **Typed errors are RFC 9457 problem details** (`application/problem+json`). Each
   typed error has a stable `type`, a relative URI under `/problems/`, listed in the
   API description.
10. **No version in the URL.** Frontend and backend are built and deployed together. A
    breaking change is marked with `!` in its commit (AGENTS.md).
11. **Field names use glossary terms** (`docs/conventions.md`).

### Values

12. **Amounts are decimal strings,** e.g. `"54.20"`, never JSON numbers, which
    JavaScript reads as floating point. utoipa's `decimal` feature describes
    `rust_decimal::Decimal` as a string.
13. **Dates are `YYYY-MM-DD`.** Instants (recorded time, occurred-at) are RFC 3339 in
    UTC.
14. **Ownership shares are `{ "numerator": 1, "denominator": 2 }`** (ADR-0019).
15. **Figure flags are fields of their own** (approximate, partial with its named
    exclusions, unknown; ADR-0021). A flag is never encoded in the number, and an
    unknown figure has no number.
16. **IDs are opaque strings** to the client.

### Tests

17. **API tests live in `crates/api/tests/`.** They cover routes, status codes and
    problem types. The domain rules are covered by the acceptance tests in `app`
    (ADR-0034).

## Alternatives considered

- **Spec-first, a hand-written OpenAPI file:** each change is written twice, in the file
  and in the code, and needs a check that both match. It plans nothing the stories and
  ADRs do not already plan.
- **aide** instead of utoipa: smaller community.
- **Generating at build time without committing:** API changes would not show in review,
  and a frontend build would need the backend's toolchain.
- **orval:** generates client and mocks, but with random data, which is useless for
  stories about the reference household.
- **@hey-api/openapi-ts:** still before 1.0.
- **Amounts as JSON numbers:** JavaScript parses them as floating point.
- **A version in the URL:** nothing consumes the API except the frontend deployed with
  it.
- **GraphQL or gRPC-web:** more tooling, and nothing in R1 needs what they add.

## Consequences

- `docs/releases/r1.md` links the API contract enabler to this ADR.
  `docs/conventions.md` no longer lists the mocking tool as open. `architecture.md`
  lists the API contract in its stack table.
- The walking skeleton sets up both diff checks in CI.
- A helper for typing MSW handlers, if one is used, is chosen when the frontend is
  scaffolded.
- Exposing the API to other clients later would need versioning; that is decided then.
