# Goal and requirements

## Vision

Iasion is a self-hosted, modular platform for understanding a household's complete
financial situation and planning its future. It models money as stocks and flows between
parties, lets contracts and rules generate expected future flows, and projects outcomes
as distributions rather than single numbers.

The questions it answers change over time. The platform must not be built around any
one of them.

## Principles

- **Holistic.** Income, costs, assets, liabilities, investments, taxes and benefits live
  in one consistent model.
- **Modular.** A small, stable kernel of primitives. Everything domain-specific
  (instrument types, importers, jurisdictions, strategies) is a plugin.
- **Jurisdiction-agnostic core.** Tax and benefit rules are plugins. Moving country means
  swapping a plugin, not rewriting the core.
- **Rigorous.** Double-entry and two-sided at every level. Exact decimals for facts.
  Assumptions are explicit, versioned and testable. No false precision.
- **Stochastic by default.** Projections share one scenario engine so every module sees
  the same simulated world.
- **Honest about time.** Facts are bitemporal (valid time and recorded time). The as-of
  date separates reconciled actuals from projections.
- **Household, not individual.** Multiple members, individual and shared ownership,
  purpose-bound pools of money.

## First guiding question (MVP driver)

> Where should the next unit of surplus money go?

For example: prepay a mortgage, invest, or build a liquidity buffer, compared across
scenarios on terminal net worth and liquidity risk. This question drives the MVP scope
but must not shape the kernel.

## Scope

### MVP (provisional, to be refined via the story map)

- Kernel primitives (see `docs/primitives.md`)
- Household with members, accounts and ownership shares
- Manual capture of accounts, positions, flows and contracts
- Loan contracts with amortization, rate structure and prepayment rights
- Statistical generators fitted from historical flows
- Scenario engine with shared stochastic drivers
- Strategy comparison for surplus allocation
- Web UI

### Later iterations (unordered)

- Universal (LLM-assisted) importer; deterministic importers derived from it
- Portfolio depth: allocation, exposure, risk contribution
- Jurisdiction plugins (tax, benefits), including deduction tagging for tax preparation
- Property and other asset valuation models
- Goals and life-event planning
- Monitoring: plan vs. actual, forecast error, drift alerts, attribution
- Consolidation of related legal entities

### Out of scope for now

- Live bank connections
- Automated actions (payments, trades)
- Multi-household hosting, fine-grained permissions

## Users

- **Primary:** household members who actively capture data and explore scenarios.
  Members may have separate income streams and separately managed outflows.
- No privacy separation between members is required initially.

## Decisions

Stack, license and structural decisions are recorded as ADRs. See `docs/adr/README.md`.
