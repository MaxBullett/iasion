# 0039: CI and secret scanning: GitHub Actions, Betterleaks, local private-terms checks

- Status: accepted
- Date: 2026-11-18
- Refines: ADR-0038

## Context

`docs/releases/r1.md` lists two enablers without an ADR: CI (build, test, lint, image
build on GitHub Actions) and secret and PII scanning (a scanner in a versioned
pre-commit hook and in CI, plus a local, gitignored `.private-terms` denylist). The
repository is public. Earlier ADRs added checks that CI must run (ADR-0033, ADR-0034,
ADR-0036) and made `cargo xtask` the one entry point (ADR-0038). The project is AGPL
(ADR-0004), so dependencies need compatible licenses.

As of this ADR, gitleaks is feature-complete and receives security patches only; its
authors maintain Betterleaks as its successor. TruffleHog can verify found secrets
against the providers' live APIs.

## Decision

### Where CI runs

1. **GitHub Actions on GitHub-hosted runners only.** No self-hosted runner: in a public
   repository it would run code from pull requests of forks.
2. **The toolchain is pinned, not containerised.** CI installs the versions from
   `rust-toolchain.toml` and `.node-version`, runs PostgreSQL (`postgres:18.7`) as a
   service container, and runs the same `cargo xtask` commands as developers
   (ADR-0038).
3. **One workflow, `ci`, runs on every pull request and on every push to `main`,** with
   these jobs. Their names are the required checks (decision 17):

   | Job | What it runs |
   |---|---|
   | `secrets` | Betterleaks over every commit of the pull request, or of the push |
   | `rust` | format, clippy with warnings as errors, the xtask checks (dependency edges, schema names, every context crate built and tested with the `postgres` feature off, `cargo sqlx prepare --check`, OpenAPI drift, migrations on an empty database), and all Rust tests |
   | `web` | install with the frozen lockfile, lint, type check, Vitest, client drift |
   | `e2e` | Playwright against the whole application |
   | `supply-chain` | `cargo-deny` and the license check of the web dependencies |
   | `image` | builds the Docker image; pushes it only from `main` |

### Tests in CI

4. **Rust tests** run with `cargo test` over the workspace with all features: unit and
   property tests next to the code, repository tests with `#[sqlx::test]` against the
   PostgreSQL service, the story acceptance tests in `crates/app/tests/`, and the API
   tests in `crates/api/tests/` (ADR-0034, ADR-0036).
5. **Web unit tests** are Vitest tests of the view logic modules
   (`docs/conventions.md`).
6. **End-to-end tests** start the built backend with a fresh, migrated database seeded
   with the reference household, serve the built frontend, and sign in with a token
   from the dev key set (ADR-0038). Playwright selects elements by role and accessible
   name and runs at phone width too (`docs/conventions.md`). On failure, CI keeps the
   Playwright traces as an artifact. Visual comparisons of charts run in Playwright's
   container image, so that rendering does not differ between machines.
7. **Golden tests that need private fixtures skip in CI,** because `fixtures/private/`
   is absent there (AGENTS.md).

### Secret scanning

8. **Betterleaks is the scanner,** pinned to an exact version, in the hooks and in CI.
9. **Hooks are versioned in `.githooks/`** and installed by `cargo xtask install-hooks`,
   which sets `core.hooksPath`. They use a local Betterleaks if one is installed,
   otherwise its container image pinned by digest, so they work wherever Docker does.
   - **pre-commit:** scans the staged changes.
   - **commit-msg:** scans the commit message.
   - **pre-push:** scans every commit about to be pushed, which also catches commits
     made with `--no-verify`.
10. **GitHub's own secret scanning and push protection** are switched on as well
    (decision 17).

### Private terms

11. **`.private-terms` is a local, gitignored file,** one term per line, compared
    ignoring case. `cargo xtask` checks it in all three hooks against the changes, the
    file names and the commit messages.
12. **It cannot run in CI,** because the list itself is private. For real names and
    other private terms, the local hooks are the only automated check; review remains
    the second.

### Supply chain

13. **`cargo-deny`** checks licenses against an allow list compatible with AGPL-3.0,
    security advisories, and that crates come from crates.io only.
14. **Web dependencies** are checked against the same allow list: `cargo xtask` reads
    `pnpm licenses list --json --prod` and fails on a license outside it.
15. **Third-party actions are pinned by full commit SHA,** and workflows get read-only
    permissions unless a job needs more.
16. **Dependabot** proposes updates for cargo, the web dependencies, Docker images and
    GitHub Actions.

### GitHub settings

17. **The maintainer applies these settings by hand.** They are not repository content,
    so this list is the source to check them against:

    **Settings → General → Pull Requests**
    - Allow squash merging: on. Allow rebase merging: on. Allow merge commits: off.
    - Automatically delete head branches: on.

    **Settings → Rules → Rulesets:** one branch ruleset, active, targeting the default
    branch, with an empty bypass list:
    - Restrict deletions: on.
    - Block force pushes: on.
    - Require linear history: on.
    - Require a pull request before merging: on; required approvals: 0; all comments
      must be resolved; allowed merge methods: squash and rebase.
    - Require status checks to pass before merging: `secrets`, `rust`, `web`, `e2e`,
      `supply-chain`, `image`; "Require branches to be up to date before merging": on.

    **Settings → Actions → General**
    - Require actions to be pinned to a full-length commit SHA: on.
    - Workflow permissions: read access for the `contents` and `packages` permissions.
    - Allow GitHub Actions to create and approve pull requests: off.
    - Approval for running fork pull request workflows: require approval for all
      external contributors.

    **Settings → Security**
    - Secret scanning runs automatically on public repositories. Push protection: on.
    - Dependabot alerts: on. Dependabot security updates: on.

## Alternatives considered

- **A self-hosted runner on the maintainer's network:** pull requests from forks would
  run code there.
- **CI inside the workspace image:** exact parity with development, but the image must
  be built or published first and every run gets slower. Pinned versions and the same
  `cargo xtask` commands give near parity.
- **gitleaks:** feature-complete, security patches only.
- **TruffleHog:** its live verification sends found secrets to the providers' APIs;
  heavier than needed here.
- **lefthook or the pre-commit framework:** one more tool to install, for three hooks.
- **Private terms in CI through an encrypted secret:** the list would leave the
  maintainer's machine, and a check's log can echo what it matched.
- **Renovate:** more capable, but one more service to configure.
- **Required approvals of 1:** GitHub does not let authors approve their own pull
  requests, so a single maintainer could never merge.

## Consequences

- AGENTS.md's "Automated checks" points to this ADR; until the walking skeleton builds
  them, the rules stay enforced by review only.
- `docs/releases/r1.md` links both enablers to this ADR, and `architecture.md` lists it.
- Required approvals of 0 means no second pair of eyes is enforced. With a second
  maintainer, it becomes 1.
- A job added later must be added to the required checks by hand.
- Where the image is pushed belongs to the deployment ADR.
