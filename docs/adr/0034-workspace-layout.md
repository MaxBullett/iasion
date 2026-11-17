# 0034: Workspace layout: one crate per context, persistence behind a feature

- Status: accepted
- Date: 2026-11-18
- Refines: ADR-0032

## Context

ADR-0032 defines five contexts and an application layer, with dependencies pointing one
way. ADR-0033 puts each context's SQL in that context's crate and all migrations in one
sequence. `docs/conventions.md` says domain code has no I/O, no clock and no randomness,
and that `unwrap` and `expect` appear only in tests. AGENTS.md expects a `SPEC.md` per
module and tests that name the story or spec section they verify. None of this said how
the repository is laid out or how the rules are enforced.

## Decision

### Layout

1. **One repository** holds the backend, the frontend, the docs and the fixtures:

   ```
   Cargo.toml          workspace
   clippy.toml
   crates/
     shared-kernel/    IDs, amounts, dates and intervals, versions, figure flags, names
     household/
     access/
     ledger/
     classification/
     views/            composes the contexts' queries; no storage
     app/              application layer: commands across contexts, today, acting member
     api/              HTTP and the API description
     server/           the binary: configuration, `serve` and `migrate` subcommands
     migrations/       the one migration sequence (ADR-0033)
   web/                the Vue app
   docs/
   fixtures/
   ```

2. **One crate per context.** Its domain code and its persistence code live in the same
   crate; the persistence code is in a `pg` module.
3. **Package names carry the prefix `iasion-`,** e.g. `iasion-ledger`. Directory names do
   not.
4. **The shared kernel is `iasion-shared-kernel`.** "Kernel" alone already means the
   primitives (glossary, `docs/primitives.md`, ADR-0005), which live in Household and
   Ledger.

### Enforcement

5. **Persistence behind a feature.** In each context crate, sqlx is an optional
   dependency enabled by a feature `postgres`, and the `pg` module is compiled only with
   it. CI also checks every context crate with that feature off, so any use of sqlx
   outside `pg`, and any domain code that reaches into `pg`, fails to compile. `server`
   enables the feature.
6. **No clock and no randomness outside `app` and `server`.** A workspace `clippy.toml`
   lists the functions that read the clock or produce randomness (from `std` and from
   each such crate the workspace uses) under `disallowed-methods`, with a reason. Only
   `app` and `server` allow that lint, at crate level.
7. **Dependencies follow the context map.** CI compares the workspace's internal
   dependency edges, read from `cargo metadata`, against this allowed list:

   | Crate | May depend on |
   |---|---|
   | shared-kernel | nothing internal |
   | household | shared-kernel |
   | access | household, shared-kernel |
   | ledger | household, shared-kernel |
   | classification | ledger, shared-kernel |
   | views | household, ledger, classification, shared-kernel |
   | app | the contexts, views, shared-kernel |
   | api | app, views, shared-kernel |
   | server | everything above, migrations |
   | migrations | nothing internal |

   A port owned upstream (ADR-0032, decision 4) is a trait in the upstream crate,
   implemented in the downstream one, so it needs no edge against the map.
8. **Schema ownership** (ADR-0033, decision 3) is checked on each context crate's `pg`
   module: its SQL names only that context's schema.

### Workspace settings

9. **One version per dependency,** declared in `[workspace.dependencies]`.
10. **Workspace lints,** inherited by every crate: `unsafe_code` is forbidden; clippy's
    `unwrap_used` and `expect_used` are denied, with `allow-unwrap-in-tests` and
    `allow-expect-in-tests` set in `clippy.toml`.
11. **Edition 2024.** The toolchain is pinned to a stable release in the walking
    skeleton.

### Specs and tests

12. **Each context crate has a `SPEC.md`:** its aggregates and invariants, the stories
    and ADRs it implements, and its place in the context map. It is short and points
    to the ADRs rather than repeating them.
13. **Where tests live:**
    - Unit and property tests next to the code they test.
    - Repository tests in the context crate's `tests/`, with `#[sqlx::test]`, behind
      the `postgres` feature.
    - Story acceptance tests in `crates/app/tests/`, one file per story
      (`s005_record_income.rs`), through the application layer. Each test names the
      acceptance criterion it covers.
    - End-to-end tests in `web/e2e/` (Playwright, `docs/conventions.md`).

## Alternatives considered

- **Two crates per context, domain and persistence:** the compiler separates them
  without a feature flag, but the workspace doubles to eight context crates. The
  feature check gives the same guarantee for sqlx.
- **Module rules through clippy alone:** clippy's disallowed lists name single
  methods, types or macros, not a whole crate or module, so it cannot ban sqlx from a
  module.
- **A grep for `sqlx` outside `pg`:** a text match, not a compiler check, and easy to
  bypass through a re-export.
- **`iasion-kernel` or `iasion-core`:** "kernel" already means the primitives; "core"
  says nothing.
- **Acceptance tests through HTTP:** slower, and they would test the API's shape
  together with the domain rules. The API gets its own tests.
- **Separate Git repositories for the frontend or per context:** specs, backend and
  frontend change together, and the generated client must match the backend in one
  commit.

## Consequences

- Cargo prevents cycles; the edge check of decision 7 also prevents a dependency in the
  wrong direction.
- The list of disallowed clock and random functions grows with the dependencies. A new
  crate that reads the clock is added to it in the same change.
- Editors need the `postgres` feature enabled to analyse `pg` modules.
- `docs/releases/r1.md` links the layout enabler to this ADR. `docs/conventions.md`
  no longer lists workspace and crate layout as deferred. AGENTS.md points to
  `crates/<context>/SPEC.md`.
- The web side is only the `web/` folder here. Its package manager belongs to the dev
  environment ADR, and the location of the generated API client to the API contract
  ADR.
- Plugins and the later contexts get their place in this layout when they arrive.
