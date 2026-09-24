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
