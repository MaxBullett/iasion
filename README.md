# Iasion

Self-hosted household finance modeling: a stock-flow ledger, contracts that generate
cash flows, and stochastic projections for deciding where the next unit of money
should go.

> In Greek myth, Iasion and Demeter are the parents of Plutus, god of wealth.
> Wealth as the product of work and time, not luck.

## Status

Pre-alpha. Nothing runs yet. The project is in its specification phase: story mapping
and domain modeling come before code.

## What it is

- A single, consistent model of a household's finances: income, costs, accounts,
  assets, liabilities, investments, taxes and benefits.
- A small kernel of stock-flow primitives, with everything domain-specific
  (instruments, importers, jurisdictions, strategies) as plugins.
- Projections as distributions from a shared scenario engine, not single numbers.
- Self-hosted. Your financial data stays with you.

## Documentation

| Document | Content |
|---|---|
| [goal-and-requirements.md](goal-and-requirements.md) | Vision, principles, scope |
| [architecture.md](architecture.md) | Contexts, stack, decisions |
| [docs/primitives.md](docs/primitives.md) | Kernel primitives and invariants |
| [docs/glossary.md](docs/glossary.md) | Ubiquitous language |
| [docs/conventions.md](docs/conventions.md) | Domain and code patterns |
| [docs/story-map.md](docs/story-map.md) | User activities, tasks, stories |
| [docs/releases/](docs/releases/) | Release definitions |
| [docs/adr/](docs/adr/) | Architecture decision records |
| [docs/open-questions.md](docs/open-questions.md) | Unresolved decisions |
| [AGENTS.md](AGENTS.md) | Rules for AI coding agents (also `CLAUDE.md`) |

## Development approach

Spec-driven and test-first. Specs are the source of truth, tests are derived from
acceptance criteria, code follows. See [ADR-0001](docs/adr/0001-spec-driven-test-first.md).

## License

[AGPL-3.0](LICENSE)
