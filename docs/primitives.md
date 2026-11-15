# Kernel primitives

Status: draft v0.5. Sections marked *(ADR-0007)* follow from
[ADR-0007](adr/0007-onboarding-present-first.md); their open details are listed in
`docs/open-questions.md`. Flows follow
[ADR-0013](adr/0013-signed-flows-on-accounts.md).

The kernel models money the way stock-flow consistent models do: everything is a stock
held by a party, or a flow between stocks, and nothing appears or disappears without a
counterparty.

These primitives are the **kernel, not the domain language**. Bounded contexts speak in
domain terms (loan, salary, support payment) that are typed compositions of these
primitives. Users and most code should never see `Generator<Flow<Unit>>`.

## Overview

| Primitive | One-liner |
|---|---|
| Party | Who. Anything that can hold value or claims. |
| Account | A container at an institution, owned by one or more parties. |
| Unit | What is being counted. |
| Position | A quantity of a unit held in an account at a point in time. |
| Flow / Transaction | A signed change to a position; a transaction groups flows that net to zero. |
| Generator | A process that produces expected flows over time. |
| Series | A time-indexed value, observed or simulated. |
| Dimension | A tag orthogonal to structure, with provenance. |
| Constraint | A predicate over positions and flows. |
| Document | Evidence that facts are derived from. |

## Party

Anything that can hold value or claims: household members, the household itself,
employers, banks, the state, legal entities, other private persons.

- External parties are modeled explicitly so every transaction is two-sided.
- Every party can own accounts. Each party outside the boundary, system parties
  included, has one **implicit account**, created with the party. Members have none:
  what a member holds is in an account they own, and a member is never a counterparty
  directly ([ADR-0013](adr/0013-signed-flows-on-accounts.md)).
- Parties have a kind (natural person, legal entity, institution, public body,
  aggregate, system). `system` parties exist for bookkeeping, not in the world, and are
  never used for real counterparties.
- Each household has one **opening party** (kind `system`, outside the boundary). It is
  the counterparty of opening balances; flows on its account never count as income or
  spending ([ADR-0011](adr/0011-opening-balance-as-transaction.md)).
- **Aggregate parties** (e.g. "various grocery merchants") keep capture overhead low
  without breaking two-sidedness.
- A **boundary** defines which parties are inside the household for reporting. Parties
  outside it (e.g. a company owned by a member) appear as counterparties; consolidation
  is a later concern.
- *(ADR-0007)* A **suspense party** is the counterparty for flows that book
  unexplained differences at checkpoints. Its kind and scope are open (OQ-011).

## Account

A container that holds positions. Owned by one or more parties with ownership shares.

- The institution is optional. A bank or brokerage account has one; a physical cash
  wallet or a self-custodied crypto wallet does not.
- Cash is held in accounts like any other unit. Withdrawing cash is a transfer between
  two household accounts, not an outflow.
- Shares per account sum to 100%.
- One party may own many accounts; one account may have many owners.
- An implicit account is owned by its party alone. More specific accounts can be
  registered for a party outside the boundary, e.g. a loan account
  ([ADR-0013](adr/0013-signed-flows-on-accounts.md)).
- Attribution to members (e.g. for tax) runs through ownership shares, never through
  the account itself.
- Each account has at most one **opening balance**: its positions at a stated date,
  taken as given. It is recorded as a checkpoint plus a transaction against the
  opening party, so positions at every date are the sum of flows
  ([ADR-0011](adr/0011-opening-balance-as-transaction.md)).
- *(ADR-0007)* Each account has **coverage** intervals, each marked
  complete, partial or unknown. Accounts have independent opening balances and
  coverage; there is no household-wide cutoff date.

## Unit

What is being counted: a currency, units of a security, a property, an index unit.

- Converting a unit to money requires a price or valuation (a Series).
- Money is `decimal + currency`. The household has one reporting currency.

## Position

A party's holding of a quantity of a unit, inside an account, at a point in time.

- Claims between parties (loans, receivables) are positions: an asset for one party,
  a liability for the other.
- A position has a **nominal value** and may have one or more **valuations**. These can
  differ materially (e.g. a claim against an entity that cannot repay it).
- *(ADR-0007)* A **checkpoint** is an observed position, e.g. a statement
  balance. It does not replace the derived position. A difference between the two is
  booked as a flow against the suspense party (OQ-010).

## Flow and Transaction

A flow is a signed change to the position of one account: an account, a unit and a
signed amount. A positive amount increases the position. A transaction groups flows
([ADR-0013](adr/0013-signed-flows-on-accounts.md)).

- **Invariant:** the flows of a transaction net to zero per unit. The transaction
  enforces it; a set of flows that does not net to zero is rejected. Example, a cash
  withdrawal with a fee: current account -202.00, cash wallet +200.00, bank +2.00.
- Every flow is on an account, never directly against a party.
- A transaction need not touch a household account. Transactions between outside
  parties affect no household view.
- A **movement**, units going from one account to another, is derived from the flows
  of a transaction. It is never stored.
- A transaction's valid time is a civil date or a closed interval of civil dates, and
  all its flows share it. An **aggregate flow** uses an interval and stands in for many
  flows over that period. It is never prorated: derived positions strictly inside the
  interval are approximate, and period views that only partly overlap it are partial
  ([ADR-0009](adr/0009-valid-time-intervals-for-aggregate-flows.md)).
- Facts use exact decimals and are append-only. A correction is a new version of the
  whole transaction; voiding is a final version marking it void
  ([ADR-0010](adr/0010-corrections-as-versions.md)).
- A realized flow may be linked to the generator that predicted it (reconciliation).
- *(ADR-0007)* **Missing data is never zero.** An aggregate of flows over an
  interval that is not completely covered is partial or unknown, and must say so.

## Generator

A process that produces expected flows over time. A generator is to flows what a
data-generating process is to observations.

Generator kinds:

| Kind | Source of flows | Example |
|---|---|---|
| Contractual | Typed contract terms | Loan annuity, employment contract |
| Scheduled | Fixed recurrence | Standing order, recurring fee |
| Statistical | Fitted to historical flows | Groceries, utilities |
| Policy | A decision rule | Surplus allocation strategy |

Control classification (orthogonal to kind):

- **Committed:** the household is bound by it.
- **Discretionary:** the household can change it at will.
- **Exogenous:** driven mainly by the world (rates, prices, benefit rules).

Generators read the world only through Series from the shared scenario engine.
They never generate their own randomness.

*(ADR-0007)* Statistical generators:

- carry parameter uncertainty into projections. Parameter draws come from the
  scenario engine, like all other randomness (OQ-014);
- are fitted only on periods with sufficient classification coverage (OQ-013);
- must not assume missingness is random (OQ-015).

## Series

A time-indexed value: prices, rates, indices, inflation, income shocks.

- Observed history and simulated paths use the same shape.
- All modules in one projection run read the same simulated paths, so correlations
  are preserved.

## Dimension

A tag orthogonal to structure: purpose, pool, tax category, productive/unproductive.

- Every assignment carries **provenance**: who or what assigned it (user, plugin,
  importer), confidence, and whether it has been confirmed.
- Plugins may define dimension vocabularies (e.g. a jurisdiction plugin defines
  deductible categories).

## Constraint

A predicate over positions and flows: goals, budgets, liquidity floors.

## Document

Evidence that facts are derived from: statements, receipts, contracts, assessments.

- Facts, import batches and dimension assignments may reference documents.

## Cross-cutting

### Time

- All facts are **bitemporal**: valid time (when it happened in the world) and recorded
  time (when the system learned it).
- Computations run relative to an **as-of date**. Before it: state folded from facts and
  contract events. After it: projections.
- Projections are derived, never stored as facts. They are keyed by as-of date,
  assumption set version and seed, and are reproducible.

### Assumptions

- Generators and series are parameterized by **assumption sets**, which are versioned.
- Without versioned assumptions, attribution (did reality change, or did we change our
  minds?) is impossible.

## Composition examples

- **Pool:** a dimension over positions and flows, with an owner. A budget is a
  constraint on a pool's flows. Pools are not a primitive.
- **Recurring benefit paid to a member:** a contractual generator (rules from a
  jurisdiction plugin) producing flows from a public-body party into the member's
  account, with an end date that projections must respect.
- **Mortgage:** a claim position held by the bank against the owners, plus a
  contractual generator built from the loan terms.
- **Shareholder loan to an owned company:** a claim position against a party outside
  the household boundary, growing with each payment. Nominal value and valuation may
  diverge.

## Open points

See `docs/open-questions.md` (OQ-001, OQ-002, OQ-009 to OQ-016).
