# 0033: Persistence: PostgreSQL only, a schema per context, versioned facts in append-only tables

- Status: accepted
- Date: 2026-11-18
- Refines: ADR-0031, ADR-0032
- Refined by: ADR-0038, ADR-0040
- Resolves: OQ-006; OQ-003 for R1

## Context

`docs/releases/r1.md` names PostgreSQL for the persistence enabler while OQ-006 was still
open, and OQ-003 asked whether anything is event-sourced. Accepted ADRs already ask
things of persistence:

- Facts and household data have versions with recorded time and the recording member,
  are append-only, and are queried "as known on" a recorded time (ADR-0010, ADR-0025,
  ADR-0028).
- A correction names the version it replaces, and invariants that span facts (one
  opening balance per account, one checkpoint per date, at least one login identity)
  hold under concurrent writes (ADR-0015, ADR-0026, ADR-0030, ADR-0031).
- Five contexts own their data, and some commands span contexts as one unit of work
  (ADR-0032).
- Amounts are exact; ownership shares are exact fractions (ADR-0019).

The data is small: a household writes a few thousand facts a year.

## Decision

### Store and layout

1. **PostgreSQL is the only store,** version 18, the newest supported major. No second
   store is added until a release shows a need for one. Documents (later) are expected
   to be files outside the database with metadata inside; that is decided when they
   arrive.
2. **One database, one schema per context:** `household`, `access`, `ledger`,
   `classification`. Views stores nothing in R1.
3. **A context's schema is read and written only by that context's crate.** Foreign keys
   exist only within a schema. Across contexts, rows refer to each other by ID only
   (ADR-0032). This is enforced by crate boundaries and a CI check that the SQL in a
   context's crate names no other context's schema.
4. **A command is one database transaction.** A command that spans contexts (ADR-0032,
   decision 6) writes all its contexts in that one transaction.

### Versioned facts

5. **Versioned facts, not events.** Each versioned thing has an identity table and an
   append-only versions table. A version carries its version number, recorded time,
   recording member, whether it voids the fact, and its content. The current version
   is the latest; a fact whose latest version voids it is void; "as known on T" uses
   the latest version with recorded time at or before T. Event sourcing is not used in
   R1. Whether contract lifecycles need it stays open in OQ-003.
6. **What is versioned:** facts and household data (ADR-0010, ADR-0025), login
   identities (ADR-0026) and the household's time zone (ADR-0030). **What is not:**
   user preferences, caches and stored read models. They are ordinary tables that may
   be updated and deleted.
7. **Append-only is enforced by the database.** There are two roles. The owner role
   owns the schemas and runs migrations. The application role has only SELECT and
   INSERT on identity and version tables, and the rights it needs on the others.

### Concurrency

8. **A write locks its aggregate root first.** Within the transaction, a command that
   corrects, voids or records something under an invariant first locks the row of its
   aggregate root (`SELECT ... FOR UPDATE`): the Account, the Household, the household's
   login identities. Under that lock it checks the version the command names
   (ADR-0031) and the invariants, then inserts. The lock lasts only for the write.
9. **A unique constraint on (identity, version number)** backs this up: the database
   never stores two versions with the same number, even if a lock is forgotten.

### Time and amounts

10. **Recorded time is the transaction timestamp,** stored as `timestamptz`. All
    versions written by one command share it. Sessions use `TimeZone=UTC`. Valid time is
    `date`; occurred-at is `timestamptz`. `timestamp` without time zone is not used.
11. **Amounts are `NUMERIC`,** never floating point. In Rust they are
    `rust_decimal::Decimal`; its range is smaller than NUMERIC's, so amount columns are
    declared with a precision it can decode. Whole minor units for EUR are a domain
    check, not a storage format (ADR-0019).
12. **Ownership shares are two integers,** numerator and denominator, stored reduced and
    with a positive denominator (ADR-0019).

### Access from Rust

13. **sqlx, no ORM.** Queries are plain SQL, checked at compile time against the schema.
    Each aggregate has one repository in its context's crate, which maps rows to domain
    types; domain types know nothing of persistence.
14. **Offline builds.** The query metadata (`.sqlx/`) is committed and builds use
    `SQLX_OFFLINE=true`. CI runs `cargo sqlx prepare --check`, which fails when the
    metadata is out of date.

### Migrations

15. **sqlx migrations, plain SQL, in one sequence.** They live in one `migrations`
    crate, named `<timestamp>_<context>_<name>.sql`, and are embedded in the binary.
    Each migration touches one context's schema; CI checks it.
16. **Forward only.** No down migrations. A failed release is undone by restoring the
    backup taken just before migrating, then fixed forward.
17. **An applied migration is never edited.** sqlx validates applied migrations against
    their files and refuses to run when one has changed. `.gitattributes` fixes line
    endings for migration files so their checksums are the same on every platform.
18. **Migrations never change what a stored version says.** A change of representation
    must be lossless. An attribute added later is NULL on older versions, meaning "not
    recorded", never an invented value.
19. **Migrations run as a separate step, not at startup.** The binary has a `migrate`
    subcommand that runs with the owner role before the new version starts. The
    application role cannot migrate (decision 7). Each migration runs in its own
    transaction, which is sqlx's default on PostgreSQL; a migration opts out only where
    PostgreSQL requires it.

### Tests

20. **Every database test gets a fresh database** with all migrations applied
    (`#[sqlx::test]`). CI also applies all migrations to an empty database on every
    run.

## Alternatives considered

- **Several stores** (e.g. a cache, a document store, a time-series store): a second
  backup and a second service on a self-hosted instance, and no transaction across
  stores, which the cross-context commands of ADR-0032 need. At this data size nothing
  needs one.
- **A database per context:** no transaction spans databases.
- **One schema with table prefixes per context:** ownership would be a naming habit, not
  a boundary a check can rely on.
- **A database role per context:** one transaction runs on one connection with one role,
  so a cross-context command could not run as one unit of work.
- **Event sourcing now:** no R1 story needs replay. Versions already keep the full
  history, append-only, and leave that door open.
- **Versioning everything:** preferences and caches gain nothing from history and would
  only grow.
- **The unique constraint alone:** it protects a single fact, not invariants that span
  facts. "Non-void" depends on a fact's latest version, which no unique index expresses.
- **`SERIALIZABLE` isolation:** any transaction can fail and needs a retry everywhere,
  and the failure says less about what conflicted.
- **Locking a fact while a member edits it:** rejected in ADR-0031.
- **The application clock for recorded time:** versions written by one command could get
  different times.
- **Integer cents:** exact and simple, but tied to two decimals. Security quantities,
  prices and currencies with other minor units would need every amount column
  migrated.
- **An ORM** (Diesel, SeaORM): the model is not CRUD. Versions are inserted, never
  updated; "current" and "as known on" are queries over versions; the views are
  aggregations. With DDD the domain types stay separate from table types anyway, so an
  ORM adds a third layer rather than saving one.
- **Migrations at application startup:** the application role would need DDL rights.
- **Down migrations:** a down migration that drops a column destroys history.
- **A migration folder per context:** no defined order across contexts, for no gain.

## Consequences

- OQ-006 is resolved. OQ-003 is resolved for R1 and stays open for contract
  lifecycles.
- `docs/releases/r1.md` links the persistence enabler to this ADR. `architecture.md`
  lists PostgreSQL and versioned facts in its stack table.
- The dev environment, CI and deployment ADRs must provide PostgreSQL 18, the two roles,
  and a backup right before every migration.
- `#[sqlx::test]` needs a superuser connection, so it runs in dev and CI only.
- Recorded time is the start of the transaction, not its commit. A version can become
  visible slightly after one with a later recorded time, so an "as known on" query in
  that moment can miss it. For a household this is negligible.
- Cargo may not rebuild when only migration files change; the `migrations` crate needs
  a `build.rs` that reruns on changes to its folder.
- The choice of date-time crate and the exact column precisions belong to the code
  conventions, written with the first crates.
