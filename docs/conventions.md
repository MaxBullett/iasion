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
- Arithmetic across currencies is an error, never an implicit conversion.
- Floating point is allowed only inside the projection layer. Conversion between
  exact and floating values happens at an explicit, named boundary.
- Rates carry their compounding and day-count convention. No bare percentages.

### Time

- Valid time uses civil dates (no timezone).
- Recorded time uses UTC timestamps.
- Domain code never reads the system clock. The as-of date and "now" are inputs.

### Aggregates and invariants

- Invariants are enforced by the aggregate that owns them:
  - Transaction: flows net to zero per unit.
  - Account: ownership shares sum to 100%.
  - Contract: state is derived from terms plus lifecycle events.
- Facts are append-only. Corrections are new facts with the correct valid time.

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

## Code conventions (deferred)

To be written once the first crates exist:

- Workspace and crate layout
- Error handling libraries
- Test layout and naming, fixture loading
- API design and versioning
- Frontend structure and state management
- Formatting and lint configuration
