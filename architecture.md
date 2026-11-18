# Architecture

Status: the contexts touched by R1 are derived from the
[R1 event storming](docs/event-storming/r1.md) and recorded in
[ADR-0032](docs/adr/0032-r1-bounded-contexts.md). Everything marked _provisional_ may
change.

## Overview

A Rust backend exposing an HTTP API, and a responsive Vue 3 + TypeScript web app
(installable as a PWA, no native apps) that consumes a client generated from the
backend's API description. Plugins are Rust crates compiled into the backend and do
not ship UI.

## Bounded contexts

### Touched by R1

| Context | Responsibility | Aggregates |
|---|---|---|
| Household | The household, its members, boundary and time zone; all parties with their kinds, including the opening party | Household; Party |
| Access | Login identities, who a signed-in request is, the first setup gate | The household's login identities |
| Ledger | Accounts with owners, ownership shares, institution and implicit accounts; opening balances; transactions and flows; checkpoints. Facts, and the figures derived from them alone: derived positions, which flows count in in and out, the allocation of flows to members | Account; Opening balance; Transaction; Checkpoint |
| Classification | The purpose vocabulary, and purpose assignments on flows with their provenance | Purpose; Dimension assignment |
| Views | Balance sheet, in and out, an account's transactions. Reads only. | none |

What enforces what:

- **Household:** one household per instance and one opening party (ADR-0024,
  ADR-0011); members are parties of kind natural person inside the boundary; a party
  never crosses the boundary and never changes kind to or from system (ADR-0025).
- **Access:** a login identity maps to one member; a household keeps at least one
  login identity (ADR-0024, ADR-0026).
- **Ledger:** the Account keeps its ownership shares summing to 1, its owners on one
  side of the boundary, its institution of kind institution, at most one non-void
  opening balance and one checkpoint per date (ADR-0015, ADR-0019, ADR-0023,
  ADR-0027, ADR-0030). The Transaction nets to zero per unit (ADR-0013). Flow dates
  respect the opening balance and today (ADR-0014, ADR-0015, ADR-0030).
- **Classification:** purpose names are unique (ADR-0029); a purpose attaches only to
  a flow that can carry one, as Ledger says (ADR-0016).

### Later (provisional)

| Context | Responsibility |
|---|---|
| Contracts | Typed terms and lifecycles; contractual generators |
| Ingestion | Import batches and proposed facts awaiting confirmation |
| Projection | Assumption sets, scenario engine, projections, strategies |
| Jurisdiction | Port implemented by jurisdiction plugins |

Valuations join Ledger when a release needs them.

## Context map

```
Household ──> Access
    │
    ├──────> Ledger ──> Classification
    │          │              │
    └──────────┴──> Views <───┘
```

Arrows point downstream: the downstream context depends on the upstream one, never the
reverse.

| Upstream | Downstream | Downstream uses | Port owned upstream, implemented downstream |
|---|---|---|---|
| Household | Access | Member IDs | Does a login identity map to this member? Asked before voiding a member (ADR-0026). |
| Household | Ledger | Parties, their kinds, whether a party is a member | Does the Ledger refer to this party? Asked before voiding a party or correcting its kind away from institution (ADR-0025, ADR-0027). |
| Ledger | Classification | Flows by transaction and flow ID; whether a flow can carry a purpose; whether it is current | none. An assignment whose flow is no longer current, or can no longer carry a purpose, is not current (ADR-0016, ADR-0017). |
| Household, Ledger, Classification | Views | Names; positions, flows and member allocations; purposes and assignments | none |

Rules:

- No context depends on a context downstream of it. There are no cycles.
- Contexts refer to each other's things by ID only.
- Every context keeps its facts and household data as versions with recorded time and
  the recording member (ADR-0010, ADR-0025, ADR-0028), rejects a correction whose
  version is no longer current (ADR-0031), and answers "as known on" a recorded time.
- Shared kernel, used by all contexts: IDs, exact amounts and minor units, civil dates
  and intervals, UTC instants, versions, figure flags (ADR-0021), name comparison
  (ADR-0029). Nothing else is shared.

### Application layer

Outside the contexts, and the only place that calls more than one of them in a command:

- It verifies each request through Access and passes the acting member into every
  command (ADR-0024, ADR-0028).
- It computes today from the clock and the household's time zone and passes it in
  (ADR-0030).
- It runs commands that span contexts as one unit of work: first setup (household,
  first member and opening party in Household; the opening party's implicit account in
  Ledger; the first login identity in Access), registering a party (Household) with
  its implicit account (Ledger), and voiding a party together with its implicit
  account.

### Where read models live

- Each context answers queries on its own data: Household the parties by name (for the
  duplicate-name warning), Ledger derived positions and an account's checkpoints with
  their differences, Classification the unclassified flows.
- Views composes the balance sheet, in and out (netting by purpose, reconciliation,
  figure flags) and an account's transactions from those queries. It never reads
  another context's storage. These rules live in the backend, not in the frontend
  (`docs/conventions.md`).

### From the event storming

| Event storming section | Context |
|---|---|
| 1. Household | Household; first setup through the application layer |
| 2. Counterparties | Household, with the implicit account in Ledger |
| 3. Accounts, 4. Opening balance, 5. Recording flows, 8. Checkpoints | Ledger |
| 6. Corrections | Each context for its own data |
| 7. Classification | Classification |
| 9. Views | Views |
| 10. Access | Access |

## Kernel

See [docs/primitives.md](docs/primitives.md). Its primitives live in Household
(parties, boundary) and Ledger (accounts, positions, flows, transactions); dimensions
live in Classification.

## Dependency rules

- Plugins depend on the kernel. The kernel never depends on plugins.
- The frontend depends only on the API, never on plugin internals.
- Jurisdiction-specific logic never lives in the kernel.
- Contexts depend on each other only along the context map.

## Stack

| Concern | Choice | Decision |
|---|---|---|
| Backend language | Rust | [ADR-0002](docs/adr/0002-rust-backend.md) |
| HTTP framework | axum, only in `api` and `server` | [ADR-0035](docs/adr/0035-axum-web-framework.md) |
| API contract | OpenAPI 3.1 from the code (utoipa), committed; typed client (openapi-typescript, openapi-fetch) | [ADR-0036](docs/adr/0036-api-contract.md) |
| Authentication | Cloudflare Access JWT verified in `api`, email mapped to a member | [ADR-0037](docs/adr/0037-authentication.md) |
| Dev environment | Docker Compose for services, a devcontainer for the toolchain, `cargo xtask`, pnpm | [ADR-0038](docs/adr/0038-dev-environment.md) |
| Frontend | Vue 3, TypeScript | [ADR-0003](docs/adr/0003-vue-frontend.md) |
| Frontend delivery | Responsive web app, PWA, no native or WASM | [ADR-0008](docs/adr/0008-responsive-web-no-native.md) |
| Plugin mechanism | Compiled-in crates | [ADR-0005](docs/adr/0005-plugins-as-compiled-crates.md) |
| Workspace layout | One crate per context, persistence behind a feature, edges checked against the context map | [ADR-0034](docs/adr/0034-workspace-layout.md) |
| Persistence | PostgreSQL 18, one schema per context, sqlx | [ADR-0033](docs/adr/0033-persistence-postgresql-versioned-facts.md) |
| Event sourcing scope | None in R1: versioned facts in append-only tables. Contract lifecycles open. | [ADR-0033](docs/adr/0033-persistence-postgresql-versioned-facts.md), OQ-003 |

## Decisions

See the [ADR index](docs/adr/README.md).
