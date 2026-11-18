# 0038: Dev environment: Docker Compose and a devcontainer, cargo xtask as the entry point

- Status: accepted
- Date: 2026-11-18
- Refines: ADR-0033, ADR-0037
- Refined by: ADR-0039

## Context

`docs/releases/r1.md` fixed the frame for the dev environment: a devcontainer for the
toolchain, Docker Compose for services, no Nix dependency. Earlier ADRs need a
PostgreSQL 18 with an owner and an application role (ADR-0033), several checks that CI
and developers run alike (ADR-0034, ADR-0036), and a local stand-in for Cloudflare
Access that does not disable verification (ADR-0037). Versions as of this ADR: Node 26
LTS, pnpm 12, the `postgres:18.7` image.

## Decision

### Where it runs

1. **Anywhere Docker and Docker Compose run.** Docker is the reference. The Compose
   files use only standard Compose features. The repository contains no Nix files;
   a developer's own environment stays outside it.
2. **Services in `compose.yaml`** at the repository root: `db` and `access-dev`.
3. **The devcontainer** (`.devcontainer/`) adds a `workspace` service with the toolchain
   through a second Compose file listed after `compose.yaml`. Without the devcontainer,
   a developer runs the same `compose.yaml` and installs the toolchain locally.

### Versions

4. **Pinned in the repository:** Rust in `rust-toolchain.toml` (pinned in the walking
   skeleton), Node 26 LTS in `.node-version`, pnpm in the `packageManager` field of
   `web/package.json`, and PostgreSQL as the Debian-based `postgres:18.7` image.
5. **pnpm** is the frontend's package manager.

### One entry point

6. **`cargo xtask`** runs everything developers and CI run: the checks (dependency edges
   and schema names from ADR-0034, the OpenAPI and client drift from ADR-0036), tests,
   regenerating the API description and client, issuing dev tokens and seeding. An
   alias in `.cargo/config.toml` provides `cargo xtask`. CI calls the same commands.

### Access in development

7. **`access-dev` serves a local key set,** and `cargo xtask dev-token --email <email>`
   issues a token signed with the matching key. The dev key pair is generated locally
   on first use in `.dev/`, which is gitignored; it is never committed.
8. **The certs URL is configurable** through `IASION_ACCESS_CERTS_URL`. By default it is
   the team domain's `/cdn-cgi/access/certs` endpoint. Verification is the same in
   every environment (ADR-0037).
9. **The Vite dev server** forwards `/api` to the backend and adds the dev token header.

### Database in development

10. **One PostgreSQL container** serves the dev database and the test databases of
    `#[sqlx::test]`. Its credentials come from `.env`; `.env.example` lists them with
    placeholders, and the Compose files contain none.
11. **An init script creates the owner and the application role with the same grants as
    in production** (ADR-0033), so a missing grant fails in development first.
12. **`cargo xtask seed` loads the reference household** (`fixtures/README.md`) through
    the application layer, as a member, so seed data passes every rule a member's input
    passes.

### Frontend

13. **The Vite dev server runs in the workspace container,** with its port forwarded.
    The story tool stays open until the frontend is scaffolded (`docs/conventions.md`).

## Alternatives considered

- **A Nix flake in the repository:** it would make Nix a dependency for anyone who uses
  it, against the frame in `docs/releases/r1.md`.
- **Podman as the reference:** daemonless and rootless by default, but Compose and dev
  container tooling are built around Docker first, and rootless volume mounts bring
  UID-mapping surprises. Its strengths matter more on the production host; the
  deployment ADR considers it.
- **`just`, `make` or package scripts as the entry point:** an extra tool, or a second
  language, while the checks need Rust code anyway.
- **npm:** no protection against using undeclared dependencies.
- **Yarn Classic:** frozen, maintenance only.
- **Yarn Berry:** Plug'n'Play is as strict as pnpm but breaks tools that expect
  `node_modules` and needs editor setup; its zero-installs would put a package cache
  into git.
- **The Alpine PostgreSQL image:** a different C library than production, with
  different collation behaviour.
- **Seeding with raw SQL:** seed data could break rules that every real input obeys.
- **A committed dev key pair:** a private key in the repository, however harmless,
  trains everyone to ignore the scanner.

## Consequences

- `.env.example` lists the database variables and `IASION_ACCESS_CERTS_URL`.
  `.gitignore` covers the generated dev key pair.
- `docs/releases/r1.md` links the dev environment enabler to this ADR, and
  `architecture.md` lists it.
- The CI ADR runs the `cargo xtask` commands, ideally in the workspace image, so CI and
  development cannot drift.
- `DATABASE_URL` holds the superuser connection that `#[sqlx::test]` and
  `cargo sqlx prepare` need. It exists only in development and CI.
