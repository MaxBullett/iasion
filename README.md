# Iasion

Self-hosted household finance modeling: a stock-flow ledger, contracts that generate
cash flows, and stochastic projections for deciding where the next unit of money
should go.

> In Greek myth, Iasion and Demeter are the parents of Plutus, god of wealth.
> Wealth as the product of work and time, not luck.

## Status

Pre-alpha. Nothing runs yet. The specification of the first release is done; the walking
skeleton is being built.

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

## Development

Needs Docker with Compose; everything else is pinned in the repository
([ADR-0038](docs/adr/0038-dev-environment.md)).

1. `cp .env.example .env`. The placeholder values work for development.
2. Either open the repository in its devcontainer, or run `docker compose up -d db` and
   use a toolchain of your own (Rust from `rust-toolchain.toml`). direnv users get `.env`
   loaded through `.envrc`, and a personal, gitignored `.envrc.local`.
3. Install the git hooks once per clone: `git config core.hooksPath .githooks` (the
   devcontainer does it). Put private terms that must never be committed into
   `.private-terms`, one per line ([ADR-0041](docs/adr/0041-git-hooks-in-shell.md)).
4. `cargo run -p iasion-server -- migrate` creates the schemas. `cargo xtask check` and
   `cargo xtask test` run what CI runs.

## License

[AGPL-3.0](LICENSE)
