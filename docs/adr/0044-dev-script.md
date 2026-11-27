# 0044: `./dev up` runs the whole app in development and reloads it on change; `cargo xtask` stays the entry point for checks

- Status: accepted
- Date: 2026-11-28
- Refines: ADR-0038, ADR-0041

## Context

ADR-0038 made `cargo xtask` the one entry point for everything developers and CI run,
and rejected `just`, `make` and package scripts as a second one. `cargo xtask` runs where
the toolchain is: in the workspace container of the devcontainer, or on a machine with
Rust installed.

Running the app in development takes more than that. The database and `access-dev`
start with Compose. Then, in the workspace: create the directory `access-dev` serves
before Docker creates it as root, migrate, create the dev key pair, `pnpm install`,
`iasion serve` in one terminal and `pnpm dev` in another. Every pull request's How to
test section has repeated these steps as shell helpers, and a machine with Docker but no
Rust toolchain, which ADR-0038 supports, has no short way in.

Iasion is one web app. Working on it should start with one command that brings up all of
it, frontend included, keep all of it current while the code changes, and stop with one
command that keeps the dev database.

ADR-0041 already wrote the git hooks as POSIX shell scripts that need only git and
Docker, for the same reason: they run on the host, before any toolchain exists.

## Decision

1. **`./dev` at the repository root** is a POSIX shell script that needs only Docker
   with Compose. It runs on the host from any shell, and drives the same Compose
   project as the devcontainer.
2. **The app runs as Compose services**, in a profile of their own so the devcontainer
   does not start them, all from the workspace image with the repository mounted:

   | Service | What it does |
   |---|---|
   | `prepare` | One shot: creates the dev key pair if missing |
   | `api` | Migrates, then `iasion serve` on port 8080, after `prepare`; builds into its own target directory, so checks in the workspace do not wait on it or rebuild it |
   | `web` | `pnpm install`, then the Vite dev server on port 5173, forwarding `/api` to `api` with a dev token |

   Ports 8080 and 5173 move from the workspace to these services. Both stay bound to
   `127.0.0.1`.
3. **`./dev up` brings up everything** in the background: creates `.env` from
   `.env.example` if missing, creates `.dev/access/public`, starts the database,
   `access-dev`, `prepare`, `api` and `web`, waits until the app answers, and prints
   `http://localhost:5173`. It is safe to run again.
4. **`./dev down` stops everything and keeps the dev database.** Only `./dev down --wipe`
   removes it, and says so.
5. **Everything reloads on change:**

   | Change | Reload |
   |---|---|
   | Rust code, `Cargo.toml`, `Cargo.lock`, a migration | `api` rebuilds, migrates and restarts |
   | Frontend code | Vite's hot reload in the browser |
   | `web/vite.config.ts` | Vite restarts itself |
   | `web/package.json`, `web/pnpm-lock.yaml` | `web` runs `pnpm install` and restarts |
   | The API's contract | `cargo xtask openapi` stays a step of its own; the TypeScript client it writes reloads through Vite |

   [watchexec](https://github.com/watchexec/watchexec) watches for `api` and `web`, and
   restarts the whole command, so a build that fails leaves `api` down with its error
   in `./dev logs api` until the next save fixes it. The workspace image installs it from
   its release with a checksum, as it does cargo-deny. Where the mount passes no file
   events, as on some Docker Desktop setups, `IASION_DEV_POLL=1` in `.env` makes it poll;
   `.env.example` lists it.

6. **The other commands:**

   | Command | What it does |
   |---|---|
   | `restart [service]` | Restarts a service, `api` by default, e.g. after changing `.env` |
   | `status`, `logs [service]` | What runs, and its logs, followed |
   | `psql` | `psql` on the dev database as the application role |
   | `token [--email <email>]` | A dev Access token (`cargo xtask dev-token`) |
   | `check`, `test`, `e2e` | `cargo xtask check` and `test`, and the Playwright tests |
   | `x <args>` | Any `cargo xtask` command in the workspace |
   | `sh` | A shell in the workspace |

7. **`./dev` holds no logic of its own.** It runs Compose and forwards to the
   workspace; whatever CI runs stays a `cargo xtask` command (ADR-0038, decision 6),
   and CI never calls `./dev`. A step that needs more than a command line becomes an
   `xtask` command.
8. **Inside the devcontainer, `cargo xtask` is used directly.** `./dev` refuses to run
   there, with that hint, because the container has no Docker.
9. **shellcheck checks every shell script in CI:** `./dev`, the git hooks, and the
   scripts in `deploy/`. A `cargo xtask shellcheck` command runs it, in the `rust` job
   and in the workspace image, which gains the package.

## Alternatives considered

- **`./dev up` running `serve` and Vite in the foreground:** one terminal held per
  session, and Ctrl-C stops half of it.
- **Rebuilding the API only on `./dev restart`:** one more step after every change,
  and easy to forget while the frontend is already current.
- **`cargo-watch`:** its maintainers point to watchexec and bacon instead.
- **`bacon`:** made for running checks in a terminal, not for keeping a server
  running in the background.
- **`just` or `make`:** one more tool to install on the host, which ADR-0038 avoided.
- **A nushell, fish or zsh helper:** ties the repository to one person's shell.
- **`cargo xtask` on the host:** needs Rust on the host, which ADR-0038 does not
  require; and `xtask` would grow container management that CI does not need.
- **The devcontainer CLI:** a Node tool on the host, and it starts the workspace, not
  the app.

## Consequences

- ADR-0038 keeps one entry point for checks and tasks; `./dev` adds one for the host
  around it. ADR-0038 and ADR-0041 get a `Refined by` line.
- The README's Development section is written around `./dev`, and How to test
  sections use it instead of their own helpers.
- The `api` service's target directory costs a second build's worth of disk.
- The workspace image gains watchexec, which Dependabot cannot see; its version is
  bumped by hand.
- A migration already applied to the dev database cannot be edited in place: `api`
  refuses to start on the changed checksum. `./dev down --wipe` starts over.
- `./dev` itself is checked by shellcheck only; its commands are exercised by every
  How to test that uses them.
- Windows needs WSL or Git Bash for `./dev`, as it already does for the git hooks.
