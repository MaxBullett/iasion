# 0035: axum is the web framework, used only in `api` and `server`

- Status: accepted
- Date: 2026-11-18
- Refines: ADR-0034
- Refined by: ADR-0036

## Context

The backend exposes an HTTP API (`architecture.md`, ADR-0002), and the API contract
enabler in `docs/releases/r1.md` depends on how handlers and types are declared. No
ADR chose a web framework. Candidates, as of this ADR (crates.io): axum 0.8, actix-web
4, Dropshot 0.18, Rocket 0.5, poem 3 with poem-openapi, salvo 1, warp 0.4, Loco 1.

## Decision

1. **axum** (0.8) on the tokio runtime is the web framework.
2. **Middleware comes from tower and tower-http** (tracing, timeouts, compression),
   not from framework-specific crates where a tower one exists.
3. **Only `api` and `server` depend on axum** (ADR-0034). Context crates, `views` and
   `app` know nothing of HTTP. The dependency check of ADR-0034 (decision 7) also
   checks that axum appears only in those two crates.

## Alternatives considered

- **actix-web:** mature, fast, a long-stable major version. It runs its own runtime on
  top of tokio, and tower middleware does not plug in directly. The safe second choice.
- **Dropshot:** OpenAPI generation is built into the framework, the most faithful
  code-first option. Small community, 0.x, made for one company's API servers.
- **Rocket:** ergonomic, but with a history of long gaps between releases.
- **poem with poem-openapi:** the OpenAPI description follows from the types, but there
  has been no release for over a year.
- **salvo:** OpenAPI built in, small community.
- **warp:** filter-based composition that is harder to read.
- **Loco:** a Rails-like framework on axum that brings an ORM, which ADR-0033 rejected.

## Consequences

- axum is still 0.x; its minor releases can break (0.7 to 0.8 did). Its version is
  pinned in `[workspace.dependencies]` (ADR-0034) and upgraded in a change of its own.
- The API contract ADR builds on axum (utoipa-axum).
- `architecture.md` lists axum in its stack table.
