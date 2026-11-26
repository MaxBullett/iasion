# 0041: Git hooks are shell scripts that need only git, grep and Betterleaks or Docker

- Status: accepted
- Date: 2026-11-26
- Refines: ADR-0039

## Context

ADR-0039 runs Betterleaks and the `.private-terms` check in three versioned hooks, and
says that `cargo xtask` checks the private terms. A hook runs wherever a commit or push
happens: in a terminal on the host, in the devcontainer, or in an IDE that runs git with
its own environment. Such an environment often has no Rust toolchain on its path, and
the dev environment is meant to work anywhere Docker runs (ADR-0038). A hook that needs
cargo would fail there, and a failing hook gets bypassed.

As of this ADR, the latest stable Betterleaks release is v1.10.1; v2, with a different
command line, is available only as release candidates.

## Decision

1. **The hooks in `.githooks/` are POSIX `sh` scripts.** They need git, grep, and either
   a local Betterleaks or Docker. They need no Rust toolchain.
2. **The hooks check `.private-terms` themselves.** Terms are matched as fixed strings,
   ignoring case; empty lines and lines starting with `#` are skipped.
   - **pre-commit:** the added lines of the staged changes and the staged file paths.
   - **commit-msg:** the commit message.
   - **pre-push:** the added lines, file paths and messages of every commit being
     pushed.
3. **A hook that finds a private term names where it is,** the file and line or the
   commit, **but never prints the term or the matching line.** Hook output can end up in
   an IDE's log.
4. **Without a `.private-terms` file, the hooks warn and continue.** A contributor may
   have none.
5. **Betterleaks is pinned to v1.10.1.** A hook uses a local `betterleaks` only if it
   reports that version; otherwise it runs the Betterleaks container image pinned by
   digest. CI's `secrets` job uses the same version. Moving to v2 is a change of its own,
   once v2 is stable.
6. **Installing:** `cargo xtask install-hooks`, or without Rust,
   `git config core.hooksPath .githooks`. Both are documented.
7. **The hooks are tested:** tests run them against a throwaway repository and check
   that a planted private term and a planted fake secret are refused, and that a clean
   commit passes.

## Alternatives considered

- **Hooks that call `cargo xtask`,** as ADR-0039 wrote: they fail on a host without Rust
  and in IDEs whose environment lacks cargo.
- **Everything through Docker, the private terms too:** a container start for every
  commit, where grep does the job.
- **A hook manager** (lefthook, the pre-commit framework): one more tool, for three
  scripts.
- **Committing only inside the devcontainer:** a rule nobody notices until it is
  broken.
- **Betterleaks v2 now:** a release candidate in the one check that guards against
  leaks.

## Consequences

- In ADR-0039, decision 11, the hooks, not `cargo xtask`, check the private terms.
- AGENTS.md says how to install the hooks.
- When Betterleaks v2 is stable, the hooks and the `secrets` job move to it together.
