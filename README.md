# Iasion

Self-hosted household finance modeling: a stock-flow ledger, contracts that generate
cash flows, and stochastic projections for deciding where the next unit of money
should go.

> In Greek myth, Iasion and Demeter are the parents of Plutus, god of wealth.
> Wealth as the product of work and time, not luck.

## Status

Pre-alpha. The walking skeleton runs: a page that shows who is signed in, from the
browser through the API to the database, deployable behind Cloudflare Access. The
first release's stories come next.

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
| [docs/deployment.md](docs/deployment.md) | Running an instance: setup, updates, backups |
| [AGENTS.md](AGENTS.md) | Rules for AI coding agents (also `CLAUDE.md`) |

## Development approach

Spec-driven and test-first. Specs are the source of truth, tests are derived from
acceptance criteria, code follows. See [ADR-0001](docs/adr/0001-spec-driven-test-first.md).

## Development

Needs Docker with Compose; everything else is pinned in the repository
([ADR-0038](docs/adr/0038-dev-environment.md)). `./dev` drives it from the host, in any
shell ([ADR-0044](docs/adr/0044-dev-script.md)); on Windows, from WSL or Git Bash.

```sh
./dev up      # the app on http://localhost:5173; the first build takes a few minutes
./dev down    # stops everything; the dev database stays
```

`./dev up` creates `.env` from `.env.example` if there is none (its placeholder values
work for development), and starts the database, `access-dev`, the API on port 8080 and
the Vite dev server on port 5173. The frontend reloads in the browser on change. After
changing Rust code, Cargo's files, a migration or `.env`, `./dev restart` builds the API
and, if the build succeeds, migrates and restarts it; if not, it prints the error and
the running API stays as it was ([ADR-0045](docs/adr/0045-api-restarts-on-demand.md)).

| Command | What it does |
|---|---|
| `./dev up`, `./dev down` | Start and stop everything. `./dev down --wipe` also removes the dev database. |
| `./dev status`, `./dev logs [service]` | What runs, and its logs (`api`, `web`, `db`, `access-dev`) |
| `./dev restart [service]` | Rebuild and restart the API, or restart another service: `./dev restart web` installs changed frontend dependencies |
| `./dev psql` | psql on the dev database, as the application role |
| `./dev token [--email <email>]` | A dev Access token |
| `./dev check`, `./dev test` | What CI's `rust` job runs, and every test |
| `./dev e2e` | The end-to-end tests in Chromium and Firefox |
| `./dev x <command>` | Any `cargo xtask` command, e.g. `./dev x openapi` |
| `./dev sh` | A shell in the workspace container |

Cloudflare Access is stood in for by `access-dev`, which serves the public half of a dev
key pair that `cargo xtask dev-token` creates in `.dev/`. The Vite dev server adds a dev
token for the setup email to every `/api` request; to call the API on port 8080
directly, send one in the `Cf-Access-Jwt-Assertion` header.

Install the git hooks once per clone: `git config core.hooksPath .githooks` (the
devcontainer does it). Put private terms that must never be committed into
`.private-terms`, one per line ([ADR-0041](docs/adr/0041-git-hooks-in-shell.md)).

### In the devcontainer, or with a toolchain of your own

The devcontainer is the same Compose project: open the repository in it and use
`cargo xtask` directly (`cargo xtask` with no command lists them). `./dev` does not run
inside it, but `./dev up` on the host next to it works; `./dev down` stops the
devcontainer's workspace too. With a toolchain of your own
(Rust from `rust-toolchain.toml`, Node and pnpm at their pinned versions),
`docker compose up -d` starts the database and `access-dev`, after
`mkdir -p .dev/access/public`; direnv users get `.env` loaded through `.envrc`, and a
personal, gitignored `.envrc.local`. Then:

- `cargo run -p iasion-server -- migrate`, then `cargo run -p iasion-server -- serve`
  for the app on port 8080.
- In `web/`: `pnpm install`, then `pnpm dev` for the frontend on port 5173
  ([ADR-0043](docs/adr/0043-frontend-tooling.md)).
- `cargo xtask check`, `cargo xtask test` and `cargo xtask web-check` run what CI runs;
  `cargo xtask supply-chain` needs cargo-deny, and `cargo xtask shellcheck` shellcheck.
- End-to-end tests, in `web/`: `pnpm exec playwright install chromium firefox` once,
  then `pnpm run build` and `pnpm exec playwright test`, against a migrated database.

## License

AGPL-3.0-or-later ([ADR-0042](docs/adr/0042-license-agpl-3-or-later.md)).

Iasion is free software: you can redistribute it and/or modify it under the terms of the
GNU Affero General Public License as published by the Free Software Foundation, either
version 3 of the License, or (at your option) any later version.

Iasion is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
PURPOSE. See the GNU Affero General Public License in [LICENSE](LICENSE) for more
details.
