# Glossary

Ubiquitous language for Iasion. Use these terms exactly in specs, code and UI.
Kernel primitives are defined in detail in `docs/primitives.md`.

## Kernel

- **Party:** anything that can hold value or claims. Includes external counterparties.
- **Aggregate party:** a party standing in for many small counterparties.
- **Account:** a container at an institution holding positions, owned by one or more parties.
- **Ownership share:** a party's fractional ownership of an account. Shares sum to 100%.
- **Unit:** what is counted (currency, security units, property).
- **Position:** a quantity of a unit held in an account at a point in time.
- **Claim:** a position representing an obligation between two parties.
- **Nominal value:** the contractual or face value of a position.
- **Valuation:** an estimate of what a position is worth, at a date, from a source.
- **Flow:** a movement of units between two positions at a valid time.
- **Transaction:** a group of flows that nets to zero per unit.
- **Generator:** a process producing expected flows over time.
- **Series:** a time-indexed value, observed or simulated.
- **Dimension:** a tag orthogonal to structure (purpose, pool, tax category).
- **Provenance:** who or what assigned a dimension, with confidence and confirmation status.
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

## Time

- **Valid time:** when something happened in the world.
- **Recorded time:** when the system learned about it.
- **As-of date:** the date separating reconciled facts from projections in a computation.
- **Reconciliation:** linking realized flows to the generator flows that predicted them.

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
