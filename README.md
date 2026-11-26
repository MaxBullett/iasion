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
2. `mkdir -p .dev/access/public`, then either open the repository in its devcontainer,
   or run `docker compose up -d` and use a toolchain of your own (Rust from
   `rust-toolchain.toml`). direnv users get `.env` loaded through `.envrc`, and a
   personal, gitignored `.envrc.local`.
3. Install the git hooks once per clone: `git config core.hooksPath .githooks` (the
   devcontainer does it). Put private terms that must never be committed into
   `.private-terms`, one per line ([ADR-0041](docs/adr/0041-git-hooks-in-shell.md)).
4. `cargo run -p iasion-server -- migrate` creates the schemas. `cargo xtask check` and
   `cargo xtask test` run what CI runs.
5. `cargo run -p iasion-server -- serve` starts the app on port 8080. Cloudflare Access
   is stood in for by `access-dev`: `cargo xtask dev-token` prints a token for the
   setup email, signed with a dev key pair it creates in `.dev/` on first use
   ([ADR-0038](docs/adr/0038-dev-environment.md)). Send it in the
   `Cf-Access-Jwt-Assertion` header, e.g. to `GET /api/session`. Right after the key
   pair is created, the app may take a minute to fetch it.

## License

AGPL-3.0-or-later ([ADR-0042](docs/adr/0042-license-agpl-3-or-later.md)).

Iasion is free software: you can redistribute it and/or modify it under the terms of the
GNU Affero General Public License as published by the Free Software Foundation, either
version 3 of the License, or (at your option) any later version.

Iasion is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
PURPOSE. See the GNU Affero General Public License in [LICENSE](LICENSE) for more
details.
