# Architecture

Status: stub. The context map will be derived from event storming against the story
map. Everything below marked _provisional_ may change.

## Overview

A Rust backend exposing an HTTP API, and a responsive Vue 3 + TypeScript web app
(installable as a PWA, no native apps) that consumes a client generated from the
backend's API description. Plugins are Rust crates compiled into the backend and do
not ship UI.

## Bounded contexts (provisional)

| Context | Responsibility |
|---|---|
| Household | Members, parties, boundary, ownership shares |
| Ledger | Accounts, positions, transactions, valuations. Facts only. |
| Contracts | Typed terms and lifecycles; contractual generators |
| Ingestion | Import batches and proposed facts awaiting confirmation |
| Projection | Assumption sets, scenario engine, projections, strategies |
| Jurisdiction | Port implemented by jurisdiction plugins |

Context relationships: _TBD after event storming._

## Kernel

See [docs/primitives.md](docs/primitives.md).

## Dependency rules

- Plugins depend on the kernel. The kernel never depends on plugins.
- The frontend depends only on the API, never on plugin internals.
- Jurisdiction-specific logic never lives in the kernel.

## Stack

| Concern | Choice | Decision |
|---|---|---|
| Backend language | Rust | [ADR-0002](docs/adr/0002-rust-backend.md) |
| Frontend | Vue 3, TypeScript | [ADR-0003](docs/adr/0003-vue-frontend.md) |
| Frontend delivery | Responsive web app, PWA, no native or WASM | [ADR-0008](docs/adr/0008-responsive-web-no-native.md) |
| Plugin mechanism | Compiled-in crates | [ADR-0005](docs/adr/0005-plugins-as-compiled-crates.md) |
| Persistence | _open_ | OQ-006 |
| Event sourcing scope | _proposed: contract lifecycles only_ | OQ-003 |

## Decisions

See the [ADR index](docs/adr/README.md).
