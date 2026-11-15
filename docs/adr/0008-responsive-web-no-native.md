# 0008: Responsive web app, installable as a PWA, no native or WASM frontend

- Status: accepted
- Date: 2026-11-15

## Context

Iasion is used mostly on desktop, with some use on phones (e.g. quick manual entry
of cash spending or aggregate flows). Offline use is not required. The instance is
self-hosted behind Cloudflare Access.

Most frontend work is done by AI agents. They need to see and test the UI cheaply:
render a component in a given state, interact with it, and check the result without
a human in the loop. That works well when the UI renders to the DOM (browser
automation, accessibility tree) and badly with native renderers.

ADR-0003 chose Vue 3 and TypeScript for a web UI but did not settle how the UI reaches
desktop and mobile.

## Decision

- The frontend is a single responsive web app (Vue 3, TypeScript, per ADR-0003),
  usable from desktop and phone browsers.
- It is installable as a progressive web app on phones. Offline support is not a
  goal; the PWA requires a connection.
- No native apps and no native shell (Tauri, Capacitor, Electron) for now.
- No Rust/WASM frontend framework.
- Agent-facing testability is a design constraint on the frontend. The binding
  practices are in `docs/conventions.md` (Frontend).

## Alternatives considered

- **Rust/WASM frontend (Leptos, Dioxus):** would share domain types with the backend
  and let the compiler catch more agent errors; Dioxus also targets desktop and
  mobile. Rejected because both are pre-1.0 with APIs that change between minor
  versions, so agents work from outdated training data and are more likely to invent
  APIs. The charting ecosystem is thin, and charts are central to Iasion. Testing
  would still run through the DOM, so the agent loop would not improve.
- **React with React Native or Expo:** the mainstream route to native mobile, with the
  largest body of agent training data. Rejected because native mobile is not needed,
  which removes its main advantage. ADR-0003's preference for Vue stands.
- **Flutter:** cross-platform from one codebase, but renders outside the DOM, which
  makes agent testing slower and harder to inspect, and means leaving TypeScript.
- **Tauri 2 or Capacitor wrapper:** keeps the web UI and adds desktop and mobile
  shells. Not needed for a browser-first, online-only app, and adds a second
  authentication path through Cloudflare Access.

## Consequences

- One codebase, one deployment, one test harness.
- Layouts must work from phone width up. Phone flows (e.g. entering a flow) get
  explicit attention instead of being shrunken desktop screens.
- Agents test the frontend with browser tooling: Playwright, accessibility
  snapshots, component stories against a mocked API.
- Semantic, accessible markup is a requirement, because the accessibility tree is
  how agents read the UI.
- If offline entry or native features become necessary, revisit with a new ADR. A
  wrapper around the existing web UI (Tauri, Capacitor) would be the first option to
  evaluate.
