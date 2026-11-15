# Glossary

Ubiquitous language for Iasion. Use these terms exactly in specs, code and UI.
Kernel primitives are defined in detail in `docs/primitives.md`.

## Kernel

- **Party:** anything that can hold value or claims. Includes external counterparties.
- **Aggregate party:** a party standing in for many small counterparties.
- **Account:** a container holding positions, owned by one or more parties. Optionally
  held at an institution: a bank account is, a physical cash wallet or a self-custodied
  crypto wallet is not.
- **Ownership share:** a party's fractional ownership of an account, stored as an exact
  fraction. The shares of an account sum to exactly 1 (ADR-0019).
- **Implicit account:** the one account created automatically for each party outside
  the boundary, system parties included, and owned by it. When only a counterparty is
  named, the flow is on its implicit account (ADR-0013).
- **Unit:** what is counted (currency, security units, property).
- **Minor unit:** the smallest amount of a unit that is attributed to a member, e.g. the
  cent for EUR (ADR-0019).
- **Position:** a quantity of a unit held in an account at a point in time.
- **Claim:** a position representing an obligation between two parties.
- **Nominal value:** the contractual or face value of a position.
- **Valuation:** an estimate of what a position is worth, at a date, from a source.
- **Flow:** a signed change to the position of one account: a quantity of a unit at a
  valid time. A positive amount increases the position (ADR-0013).
- **Transaction:** a group of flows that nets to zero per unit.
- **Aggregate flow:** a flow that stands in for many flows over a period, e.g. a month
  of grocery spending entered as one amount. Its valid time is an interval, and it is
  never prorated (ADR-0009).
- **Movement:** units going from one account to another. Derived from the flows of a
  transaction, e.g. for a flow diagram; never stored (ADR-0013).
- **Transfer:** flows on household accounts that offset each other within a
  transaction: units moving between household accounts. Not in or out for the
  household. A transaction may combine a transfer with other flows, e.g. a cash
  withdrawal with a fee: current account -202.00, cash wallet +200.00, bank +2.00.
- **In, out:** the direction of a flow that counts for the household: a flow on the
  account of a party outside the boundary that is not a system party, in a transaction
  that has a flow on a household account. Positive there is out, negative is in.
  Gross, never netted (ADR-0013, ADR-0022).
- **Generator:** a process producing expected flows over time.
- **Series:** a time-indexed value, observed or simulated.
- **Dimension:** a tag orthogonal to structure (purpose, pool, tax category).
- **Provenance:** who or what assigned a dimension, with confidence and confirmation
  status. A manual assignment is confirmed, with confidence certain (ADR-0017).
- **Constraint:** a predicate over positions and flows.
- **Document:** evidence that facts are derived from.

## Generators

- **Contractual generator:** derives flows from typed contract terms.
- **Scheduled generator:** derives flows from a fixed recurrence.
- **Statistical generator:** derives flows from a model fitted to historical flows.
- **Policy generator:** derives flows from a decision rule.
- **Committed / discretionary / exogenous:** who controls a generator.

## Household

- **Household:** the set of member parties inside the reporting boundary.
- **Member:** a natural person inside the household.
- **Boundary:** the line separating household parties from counterparties.
- **Pool:** a dimension grouping positions and flows for a purpose, with an owner.
- **Budget:** a constraint on a pool's flows.
- **Between members:** in a member view, what remains of the change in a member's
  attributed positions after in and out: change minus in plus out. It arises when units
  move between household accounts with different ownership. Never in or out. Sums to
  zero across the household (ADR-0012, ADR-0018).

## Views

- **Income, spending:** view terms. A view by purpose nets in and out per purpose over
  its period. A purpose with net in is income, one with net out is spending.
  Unclassified is shown as in and out separately (ADR-0022).

## Time

- **Valid time:** when something happened in the world: a civil date, or an interval of
  civil dates for aggregate flows. Never a time of day (ADR-0014).
- **Occurred-at:** an optional UTC instant on a transaction, given only when its source
  provides one. Orders transactions within one valid date; never changes the valid
  date (ADR-0014).
- **Recorded time:** when the system learned about it.
- **As-of date:** the date separating reconciled facts from projections in a computation.
- **Version:** one recorded state of a fact. A fact has a stable ID and one or more
  versions, each with its recorded time (ADR-0010).
- **Correction:** a new version of a fact that replaces the previous one in current
  views. Not a reversal entry. Rejected if it would invalidate an independent recorded
  fact, such as another transaction or a checkpoint (ADR-0015).
- **Void:** a final version marking a fact as not having happened. Voided facts affect
  no current view.
- **Reconciliation:** linking realized flows to the generator flows that predicted them.

## Data quality

From [ADR-0007](adr/0007-onboarding-present-first.md).

- **Opening balance:** the positions of an account at the end of a stated date, taken
  as given.
  Recorded as a checkpoint plus a transaction against the opening party, so positions
  remain the sum of flows (ADR-0011). One fact with its own versions; an account has
  at most one non-void opening balance (ADR-0015).
- **Opening party:** the household's `system` party that is the counterparty of opening
  balances. Outside the boundary; flows on its account never count as in or out.
- **System party:** a party that exists for bookkeeping, not in the world (opening
  party, later the suspense party).
- **Tracked:** an account is tracked for every date on or after the date of its opening
  balance, which is an end-of-day balance. Before that its positions are unknown and
  its flows are not recorded. Views say "not tracked", never zero (ADR-0020).
- **Coverage:** for one account, the intervals of valid time and whether recorded flows
  in each are complete, partial or unknown. Missing data is never read as zero.
- **Checkpoint:** a position in an account observed at the end of a date, e.g. a
  statement balance. Derived positions are compared against it; it does not replace them.
- **Suspense party:** the counterparty for flows that book unexplained differences
  between a checkpoint and the derived position. Its balance, the suspense balance,
  is a data quality metric.
- **Classification coverage:** for a period, the extent to which its recorded flows
  carry dimension assignments. Distinct from coverage, which is about whether flows
  are recorded at all. How it is measured is open (OQ-013).
- **Approximate (position):** a derived position whose as-of date is on or after the
  first day and before the last day of the interval of an aggregate flow on that
  account (ADR-0009, ADR-0014). A total is approximate if any of its parts is
  (ADR-0021).
- **Partial (figure):** a figure known to exclude something it should contain. It names
  each exclusion and its reason, e.g. an aggregate flow whose interval only partly
  overlaps the period, or an account that is not tracked yet. A total is partial if
  any of its parts is (ADR-0021). Not to be confused with partial coverage.
- **Unknown (figure):** a single figure that cannot be stated, e.g. the position of an
  account before it is tracked. Never shown as zero. A total leaves it out and is
  flagged partial (ADR-0021).

## Projection

- **Assumption set:** versioned parameters for generators and series.
- **Scenario engine:** the single source of simulated series for a projection run.
- **Scenario path:** one simulated realization of all series over the horizon.
- **Projection:** derived future positions and flows for an as-of date, assumption set
  and seed.
- **Strategy:** a policy generator whose outcomes can be compared against others.
- **Attribution:** explaining a change in outcome by changes in facts vs. changes in
  assumptions.

## Plugins

- **Plugin:** a crate implementing kernel traits, compiled into the application.
- **Jurisdiction plugin:** rules for tax, benefits and contributions for one jurisdiction.
- **Importer:** turns a source document into proposed facts for confirmation.
