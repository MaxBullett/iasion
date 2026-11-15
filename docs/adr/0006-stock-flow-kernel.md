# 0006: Stock-flow kernel with two-sided flows

- Status: accepted
- Date: 2026-09-25
- Refined by: ADR-0013

## Context

Modules (importers, instruments, jurisdictions, strategies) must compose into one
consistent picture of a household's finances and projections.

## Decision

The kernel consists of a small set of stock-flow primitives (see
`docs/primitives.md`). Every flow is two-sided: external counterparties are modeled
as parties. Domain concepts are typed compositions of primitives, not primitives
themselves.

## Alternatives considered

- Feature-specific models per module: faster per module, but modules cannot compose.
- One-sided flows (household perspective only): less modeling overhead, but loses
  accounting consistency and makes claims between parties awkward.

## Consequences

- Accounting consistency holds everywhere; nothing appears without a counterparty.
- More modeling overhead for trivial flows, handled via aggregate parties and the UI.
- Views like flow diagrams come from aggregating the same data at different levels.
