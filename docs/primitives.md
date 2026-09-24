# Kernel primitives

Status: draft v0.2

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
| Flow / Transaction | Units moving between positions at a point in time. |
| Generator | A process that produces expected flows over time. |
| Series | A time-indexed value, observed or simulated. |
| Dimension | A tag orthogonal to structure, with provenance. |
| Constraint | A predicate over positions and flows. |
| Document | Evidence that facts are derived from. |

## Party

Anything that can hold value or claims: household members, the household itself,
employers, banks, the state, legal entities, other private persons.

- External parties are modeled explicitly so every flow is two-sided.
- Parties have a kind (natural person, legal entity, institution, public body, aggregate).
- **Aggregate parties** (e.g. "various grocery merchants") keep capture overhead low
  without breaking two-sidedness.
- A **boundary** defines which parties are inside the household for reporting. Parties
  outside it (e.g. a company owned by a member) appear as counterparties; consolidation
  is a later concern.

## Account

A container at an institution that holds positions. Owned by one or more parties with
ownership shares.

- Shares per account sum to 100%.
- One party may own many accounts; one account may have many owners.
- Attribution to members (e.g. for tax) runs through ownership shares, never through
  the account itself.

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

## Flow and Transaction

A flow moves a quantity of a unit from one position to another at a valid time.
A transaction groups flows.

- **Invariant:** the flows of a transaction net to zero per unit.
- Facts use exact decimals and are append-only. Corrections are new facts.
- A realized flow may be linked to the generator that predicted it (reconciliation).

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

See `docs/open-questions.md` (OQ-001, OQ-002).
