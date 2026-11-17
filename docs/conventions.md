# Conventions

Status: domain patterns are binding. Code conventions are deferred until code exists,
to avoid guessing.

## Domain patterns (binding)

### Naming

- Use glossary terms in code: types, modules, functions, API fields.
- No new domain terms without a glossary entry in the same change.

### Values and identity

- Value objects are immutable and validated at construction. Invalid values must be
  unrepresentable, not checked later.
- Every entity has its own newtype ID (`PartyId`, `AccountId`, ...). Raw UUIDs or
  strings never cross module boundaries.

### Money and numbers

- Facts use exact decimals. `Money` is `decimal + currency`.
- Attribution to members allocates each flow in whole minor units by largest
  remainder, so member figures sum exactly to household figures (ADR-0019).
- A figure is a value plus flags: approximate, and partial with named exclusions. A
  total takes the union of the flags of its parts. Unknown is never shown as zero
  (ADR-0021).
- Arithmetic across currencies is an error, never an implicit conversion.
- Floating point is allowed only inside the projection layer. Conversion between
  exact and floating values happens at an explicit, named boundary.
- Rates carry their compounding and day-count convention. No bare percentages.

### Time

- Valid time uses civil dates (no timezone): a date, or an interval of dates. A time of
  day that the source does not provide is never invented (ADR-0014).
- A position, opening balance or checkpoint at a date is the end-of-day state of that
  date: it includes every flow with a valid date on or before it (ADR-0014).
- A transaction may carry an optional occurred-at, a UTC timestamp, only when its
  source provides one. It orders transactions within one valid date and never changes
  the valid date (ADR-0014).
- Recorded time uses UTC timestamps.
- Domain code never reads the system clock. The as-of date and "now" are inputs.

### Aggregates and invariants

- Invariants are enforced by the aggregate that owns them:
  - Transaction: flows net to zero per unit.
  - Account: ownership shares are exact fractions that sum to exactly 1 (ADR-0019);
    at most one non-void opening balance (ADR-0015).
  - Contract: state is derived from terms plus lifecycle events.
- Facts are append-only. Each fact has a stable ID and versions over recorded time; a
  correction is a new version, voiding is a final version marking it void. The
  transaction, not the single flow, is the versioned unit (ADR-0010). A flow keeps a
  stable ID within its transaction across versions (ADR-0016).
- A correction or void never invalidates independent recorded facts (other
  transactions, checkpoints). One that would is rejected with a typed error naming the
  conflicting facts. Dependent facts, such as dimension assignments on a transaction's
  flows, follow their parent (ADR-0015, ADR-0016).
- A correction may change anything but the fact's identity, as long as the new version
  satisfies all rules and breaks no independent fact (ADR-0017).
- Parties, accounts, ownership shares and purposes are versioned the same way. A
  corrected ownership share applies to all dates; nothing has an effective date in R1
  (ADR-0025).
- Every version records the member who recorded it, next to its recorded time.
  Sign-ins and denied requests are not facts: they are not stored, and appear in
  operational logs only (ADR-0028).
- Queries use current knowledge (latest non-void version) by default. "As known on"
  queries select versions by recorded time.

### Generators and projections

- Generators are pure: inputs are state and a scenario path, output is flows.
  No I/O, no clock, no randomness.
- All randomness comes from the scenario engine with an explicit seed.
- Projections are reproducible from (as-of date, assumption set version, seed).

### Errors

- Domain errors are typed values, not strings.
- No panics in domain code. `unwrap`/`expect` only in tests.

### Boundaries

- The kernel knows nothing about plugins or jurisdictions.
- Plugins depend on the kernel, never on each other unless explicitly declared.

## Frontend (binding)

Follows from [ADR-0008](adr/0008-responsive-web-no-native.md). The goal is a frontend
that agents can render, inspect and test without a human clicking through it.
Tool choices marked _open_ are settled when the frontend is scaffolded.

### Where logic lives

- Domain rules live in the backend. The frontend never re-implements a domain rule
  (e.g. ownership attribution, netting); it displays what the API returns.
- View logic (formatting, grouping, filtering, chart data shaping) lives in plain
  TypeScript modules outside components, unit-tested with Vitest without rendering.
- Components stay thin: props in, events out, minimal internal state.

### Seeing components in isolation

- Every reusable component and every screen has stories covering its relevant
  states: empty, loading, error, typical data, edge cases (long names, negative
  amounts, unknown or incomplete figures). Story tool: _open_ (Storybook or Histoire).
- Stories and frontend tests run against a mocked API built from the same generated
  API types as the real client, so they cannot drift. Mocking tool: _open_.
- Fixture data for stories is synthetic. No personal data, as for all fixtures.

### Testing interaction

- End-to-end and interaction tests use Playwright.
- Tests select elements by role and accessible name, not by CSS class or test IDs,
  unless no accessible handle exists.
- Agents inspect running pages through accessibility snapshots (e.g. Playwright MCP)
  first, screenshots second.

### Accessibility is part of the agent interface

- Use semantic elements and ARIA only where semantics are missing. Every interactive
  element has an accessible name.
- Figures that are unknown or incomplete (ADR-0007) are exposed in text, not only
  through colour or icons.

### Charts

- Chart data is shaped in tested TypeScript modules (see above); the chart component
  only renders.
- Charts expose their underlying values in an accessible form (e.g. a data table or
  labelled summary), so agents and screen readers can verify them.
- Visual regression screenshots are used for charts, where the accessibility tree
  cannot show correctness. Elsewhere, prefer accessibility assertions.

### Layout

- Every screen works from phone width (360 px) up. Phone flows are designed, not
  shrunk.

## Code conventions (deferred)

To be written once the first crates exist:

- Workspace and crate layout
- Error handling libraries
- Test layout and naming, fixture loading
- API design and versioning
- Frontend structure and state management
- Formatting and lint configuration
