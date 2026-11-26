# 0043: Frontend tooling: Vite, ESLint and Prettier, served by the backend

- Status: accepted
- Date: 2026-11-26
- Refines: ADR-0003, ADR-0036, ADR-0037, ADR-0038, ADR-0039, ADR-0040

## Context

ADR-0003 chose Vue 3 and TypeScript and deferred build tooling and state management.
ADR-0036 fixed the API client (openapi-typescript, openapi-fetch) and the mocks (MSW),
and ADR-0038 fixed Node 26 and pnpm. `docs/conventions.md` (Frontend) binds where view
logic lives, testing by role and accessible name, accessibility and phone width, and
leaves the story tool open. ADR-0040 puts the built frontend into the backend's image,
served by the backend. ADR-0037 verifies every API request, and ADR-0038 has the Vite
dev server add a dev token.

As of this ADR: Vue 3.5, Vite 8, vue-router 5, TypeScript 7.0 is the latest release
while typescript-eslint supports TypeScript below 6.1 and openapi-typescript 5.x,
Node 26 no longer ships corepack, and pnpm 12 can refuse versions younger than a set
age (`minimumReleaseAge`), allows dependency build scripts only by name
(`allowBuilds`), and can refuse a version published with weaker provenance than its
predecessors (`trustPolicy`). Histoire 1.0 is in beta.

## Decision

### Package and build

1. **`web/` holds the frontend** as one pnpm package, not a workspace of several. It is
   built with Vite and `@vitejs/plugin-vue` and routed with vue-router.
2. **TypeScript 5.9,** pinned exactly, until the lint and client generator support a
   newer major. `vue-tsc` type-checks, including templates.
3. **No state library for now.** Screens load their data through openapi-fetch in small
   composables. A store (Pinia) is introduced when state is shared across screens.

### Lint and format

4. **ESLint** with eslint-plugin-vue, typescript-eslint and
   eslint-plugin-vuejs-accessibility, in a flat config. The accessibility rules check
   part of `docs/conventions.md` mechanically, e.g. that interactive elements have an
   accessible name.
5. **Prettier** formats. ESLint does not check formatting.

### Packages and supply chain

6. **pnpm's version is pinned in `packageManager`** of `web/package.json`. The
   devcontainer and CI install exactly that version, since Node no longer provides
   corepack.
7. **`web/pnpm-workspace.yaml` sets:**
   - `minimumReleaseAge`: versions younger than three days are not installed, so a
     hijacked release is usually withdrawn before it reaches the lockfile;
   - `allowBuilds`: dependency install scripts run only for packages listed there;
   - `trustPolicy: no-downgrade`: a version with weaker provenance than earlier ones
     is refused.

### Tests

8. **Vitest** tests view logic modules without rendering (`docs/conventions.md`).
9. **MSW handlers** are typed from the generated schema (ADR-0036) and serve stories
   and component tests.
10. **Playwright** runs end-to-end tests against the built backend serving the built
    frontend, a migrated database, and `access-dev` for the keys. The token is sent as
    the Access header. Tests run in Chromium and Firefox, the two engines behind most
    browsers, each at desktop width and at 360 px.
11. **The story tool stays open** until the first reusable component (Storybook or
    Histoire, `docs/conventions.md`).

### Serving

12. **`iasion serve` serves the built frontend** at `/`, and `index.html` for any other
    path outside `/api`, so client-side routes load.
13. **Static files need no Access token.** They are the published code and hold no
    household data; Access is in front of them anyway. Everything under `/api` is
    verified as before (ADR-0037).
14. **Every response carries security headers:** a `Content-Security-Policy` that
    allows content from the app's own origin only and forbids framing
    (`frame-ancestors 'none'`), `X-Content-Type-Options: nosniff`, and
    `Referrer-Policy: no-referrer`.
15. **In development, the Vite dev server** forwards `/api` to the backend and adds a
    token from `cargo xtask dev-token`, issued when the dev server starts (ADR-0038,
    decision 9).

### CI

16. **`web`:** install with the frozen lockfile, lint, format check, type check, Vitest,
    and the client drift check through `cargo xtask` (ADR-0036, ADR-0039).
17. **`e2e`:** PostgreSQL, the migrated database, `access-dev` as the same container
    image as in development, the built app, and Playwright. Traces are kept as an
    artifact on failure (ADR-0039, decision 6).

### Deferred

18. **Not decided here:** installability as a PWA (ADR-0008), the CSS approach beyond
    scoped styles in components, and the charting library. Each is decided with the
    first change that needs it.

## Alternatives considered

- **TypeScript 7.0:** the native compiler, but typescript-eslint and openapi-typescript
  do not support it yet.
- **Biome for lint and format:** one fast tool, but its Vue support is partial.
- **oxlint:** fast, but it covers a subset of the rules and no Vue accessibility rules.
- **Pinia from the start:** a store with nothing shared to keep in it.
- **A pnpm workspace with several packages:** one app does not need it.
- **Corepack for pnpm:** no longer shipped with Node 26.
- **Serving static files only with a valid token:** protects nothing that is not
  already public, and a page that cannot load cannot show why it was refused.
- **Serving the frontend from its own server or a CDN:** a second origin, with CORS and
  a harder Origin check (ADR-0040).
- **Chromium only:** the fastest run, but no coverage of Gecko, the engine of Firefox.
- **WebKit as well:** Safari's engine, which matters on iPhones. It is added when a
  phone-specific flow needs it.

## Consequences

- `docs/conventions.md` no longer lists formatting and lint configuration, nor frontend
  structure and state management, as deferred for the frontend; it links here.
  `architecture.md` lists the frontend tooling.
- The workspace image gains Node and pnpm. Playwright's browsers are installed on
  demand, not baked into the image.
- New dependency versions reach the lockfile three days after release at the earliest,
  security fixes included. `minimumReleaseAgeExclude` lets a fix through when it is
  urgent.
- Dependabot's proposals for web dependencies must respect the release age, or their
  install fails (ADR-0039, decision 16).
