# Open questions

Unresolved decisions and unknowns. Agents add entries here instead of assuming.
Format: ID, question, context, status. Resolved entries link to the ADR or spec that
settled them.

## OQ-001: Conditional flows

Do insurance contracts (flows conditional on events) fit the Generator primitive, or do
they need an extension (e.g. event-triggered generators)?

Status: open

## OQ-002: Derivatives

Do option-like instruments (value depends on a series and a payoff function) fit
Position + Generator + Series, or do they need a dedicated valuation abstraction?

Status: open

## OQ-003: Scope of event sourcing

Proposed: event-source contract lifecycles only; the ledger is append-only but not
event-sourced; projections are derived and cached. Needs an ADR.

Update: [ADR-0010](adr/0010-corrections-as-versions.md) stores ledger facts as
versioned facts with recorded time, which covers corrections and "as known on" queries
without full event sourcing. What remains open is whether contract lifecycles go
further.

Status: proposed

## OQ-004: Plugin mechanism

Plugins are compiled-in crates for now (ADR-0005). Is a runtime boundary (e.g. WASM components)
needed later, and what would trigger that decision?

Status: open

## OQ-005: Calibration tooling

Estimating models for the scenario engine (e.g. VAR, GARCH) has weak Rust library
support. Proposed: separate calibration (offline, any tool) from simulation (core),
with calibration producing versioned parameter files. Needs an ADR.

Status: proposed

## OQ-006: Persistence

Database choice not decided.

Status: open

## OQ-007: Contributor licensing

Decide on a CLA or DCO before accepting the first external contribution, to preserve
relicensing options under AGPL-3.0.

Status: open

## OQ-008: Task IDs in the story map

Activities use `A#` and stories use `S-###`. Tasks have no ID scheme yet. Do tasks need
IDs (e.g. for traceability from stories), and in what format?

Status: resolved. Format `A<activity>.<n>`, never reused or renumbered; defined in
`docs/story-map.md`.

## OQ-009: Recording an opening balance, and backfilling before it

Is an opening balance recorded as flows (which, per ADR-0006, need a counterparty: the
suspense party, a dedicated opening party, something else?) or as a distinct kind of
fact that states positions directly? When history before the opening balance is
backfilled later (ADR-0007, principle 1), does the opening balance move to the new
start, become a checkpoint, or get corrected by a new fact?

Status: resolved by [ADR-0011](adr/0011-opening-balance-as-transaction.md) (a
checkpoint taken as given plus a transaction against the household's opening party;
backfill is a correction of that transaction per ADR-0010).

## OQ-010: Booking differences at checkpoints

ADR-0007 books unexplained differences at checkpoints against the suspense party.
Open details:

- Is the suspense flow booked automatically, or proposed and confirmed by the user?
- Which valid time does it get: the checkpoint date, or spread over the interval since
  the previous checkpoint?
- When the difference is explained later (the missing flows are recorded), how is the
  suspense flow reversed, given facts are append-only?
- For accounts holding non-currency units (e.g. securities), is a checkpoint on
  quantities, on value, or both? How does it relate to a valuation?
- Must a checkpoint reference a document, or can the user enter one directly (e.g. a
  balance read off a banking app)?
- What does a difference inside an interval marked complete mean: an error to
  resolve, or a signal that downgrades the interval to partial?

Status: open

## OQ-011: Scope and kind of the suspense party

One suspense party per household, per account, or per member? Is it inside or outside
the boundary? Which party kind is it (aggregate, or a new kind)? Are flows against it
excluded from income and expense views, or shown as a separate line? Are they excluded
when fitting statistical generators?

Input: [ADR-0011](adr/0011-opening-balance-as-transaction.md) introduces the opening
party (one per household, outside the boundary, kind `system`, excluded from in and
out). The suspense party is likely to follow the same pattern.

Status: open

## OQ-012: How coverage is established

Is coverage declared by the user, derived from attached documents (e.g. statement
periods), or both? Does a coverage assignment carry provenance like a dimension? What
distinguishes partial from unknown? Is coverage tracked per account or per unit within
an account?

Status: open

## OQ-013: Classification coverage

ADR-0007 fits statistical generators only on periods with "sufficient classification
coverage" (glossary: classification coverage). This is a different thing from account
coverage (glossary: coverage). Open:

- How it is measured: share of flows by count or by amount? Per dimension?
- Do unconfirmed suggestions count, or only confirmed assignments?
- What threshold counts as sufficient, and where does it live (e.g. in the
  assumption set)?
- Does fitting also require complete coverage of the period? A fully classified
  period with partial coverage would pass a classification test but still be biased.

Status: open

## OQ-014: Representing parameter uncertainty

Statistical generators carry parameter uncertainty into projections, and all
randomness comes from the scenario engine. How are parameter draws represented (e.g.
one draw per scenario path), and are fitted parameter distributions part of the
versioned assumption set?

Status: open

## OQ-015: Non-random missingness

ADR-0007 forbids assuming missingness is random when fitting statistical generators.
Which approach is used instead (e.g. modeling the missingness mechanism, excluding
affected periods, sensitivity bounds, widening uncertainty)? Needs a spec, ideally
with a cited statistical reference.

Status: open

## OQ-016: State of contracts registered mid-life

Contract state is derived from terms plus lifecycle events (`docs/conventions.md`).
Present-first onboarding (ADR-0007) registers open contracts mid-life, usually without
their past lifecycle events (e.g. earlier prepayments or rate resets on a loan). How is
a contract's state at registration captured: an opening state at a date (analogous to
an opening balance), mandatory backfill of past events, or something else? A gap here
would otherwise show up as a checkpoint difference (e.g. against the outstanding
principal on a loan statement) and end up with the suspense party. Related: OQ-003.

Status: open

## OQ-017: Valid time of aggregate flows

R1 allows manual aggregate flows (e.g. "groceries, October 2026, EUR 640.00", S-006).
A flow currently happens at a point in time. Does an aggregate flow get a single valid
date (e.g. the last day of the period), or a valid-time interval? If an interval, how
do derived positions inside the interval and period views that cut through it treat
it (e.g. the as-of balance on the 15th)? Related: R2, where imported detail for a
period must replace an aggregate for the same account and period without double
counting.

Status: resolved by [ADR-0009](adr/0009-valid-time-intervals-for-aggregate-flows.md)
(valid-time interval, no proration).

## OQ-018: Member view of transfers between members

In the household view, transfers between household accounts are excluded from in and
out (S-007, S-013). In a member view, a transfer from Alex's account to Sam's account
is an outflow for one member and an inflow for the other. Is it shown, excluded, or
shown as a separate "between members" line? The same applies to transfers into or out
of joint accounts, where ownership shares differ between the two accounts.

Status: resolved by [ADR-0012](adr/0012-between-members-line.md) (separate "between
members" line, never in or out).

## OQ-019: Flows before the opening balance

R1 has no backfill (ADR-0007, principle 1, is later). Is a flow dated before an
account's opening balance rejected in R1, or accepted and ignored by derived
positions? Proposed for R1: rejected, with a typed error.

Status: resolved. Rejected with a typed error in R1 (S-005); backfill later is a
correction of the opening balance ([ADR-0011](adr/0011-opening-balance-as-transaction.md)).

## OQ-020: Correcting a flow

Facts are append-only and corrections are new facts (`docs/conventions.md`). For a
wrongly entered flow (S-008), is the correction a superseding fact that replaces the
original in all views, or a reversing flow plus a new flow? The first keeps views
clean and maps naturally onto bitemporal queries; the second is the classic
bookkeeping approach and keeps every effect visible as a flow. Also: how is voiding
represented, and can an opening balance be corrected the same way?

Status: resolved by [ADR-0010](adr/0010-corrections-as-versions.md) (new version of
the transaction; voiding is a final version; opening balances version the same way).
