# 0045: In development the API rebuilds on `./dev restart`, not on every change

- Status: accepted
- Date: 2026-11-28
- Refines: ADR-0044

## Context

ADR-0044 had `./dev up` rebuild, migrate and restart the API on every change to the
Rust code, Cargo's files or a migration, with watchexec. Built and tried, it costs more
than it gives:

- Every save of a migration still being written applies it to the dev database. The
  next save changes its checksum, and the API refuses to start until the dev database
  is wiped, which is the data `./dev down` keeps.
- The API is down for every build, since the old server stops before the new one is
  built, and stays down after a build that fails. Keeping the last good build running
  needs a second watcher for the binary.
- A first build that fails looks like a slow one to the API's health check, so
  `./dev up` waits for its whole start period.

Each has a fix, and together they are a lot of machinery for saving one command. The
frontend's hot reload is Vite's own and has none of these problems.

## Decision

1. **The API does not rebuild on change.** `./dev restart` rebuilds it, and restarts it
   only once the build succeeds: a build that fails leaves the running API as it was,
   and prints the error. Restarting migrates first, as `./dev up` does, so a migration
   is applied only when you ask for it.
2. **The frontend keeps Vite's hot reload** (ADR-0044, decision 5). `./dev restart web`
   installs changed dependencies and starts it again. `IASION_DEV_POLL` makes Vite poll,
   for mounts that pass no file events.
3. **The `prepare` one shot also builds the API.** A build that fails makes `./dev up`
   fail at once with the error, instead of waiting.
4. **watchexec leaves the workspace image.**

ADR-0044's other decisions stand.

## Alternatives considered

- **Keep rebuilding on change and fix each problem** (no migration in the reload loop,
  a second watcher that restarts on a new binary, an initial build before the health
  check): the machinery of the Context, for saving `./dev restart`.
- **Rebuild on change, but never migrate in the loop:** fixes the worst problem, keeps
  the downtime during builds and after failed ones.

## Consequences

- After changing Rust code, Cargo's files or a migration: `./dev restart`.
- The workspace image has one tool less to bump by hand.
- ADR-0044 gets a `Refined by` line.
