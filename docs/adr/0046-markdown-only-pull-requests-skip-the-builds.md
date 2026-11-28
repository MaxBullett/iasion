# 0046: A pull request that changes only Markdown skips the build jobs in CI

- Status: accepted
- Date: 2026-11-28
- Refines: ADR-0039

## Context

ADR-0039 runs every job of the `ci` workflow on every pull request (decision 3). The
workflow has no path filters: a workflow skipped by a path filter never reports its
required checks, so the pull request waits for good.

This project is spec-driven, so much of its work is Markdown. 12 of the 30 pull requests
merged before this ADR changed nothing else. Each of them still built and tested the
Rust workspace and the frontend, ran Playwright, checked the supply chain and built the
image: about six minutes of runner time, and two to three minutes before the checks
went green, to find that nothing that runs had changed.

As of this ADR, GitHub's documentation on required status checks states that a job
skipped by its `if:` condition reports success, that a workflow skipped by a path filter
leaves its checks pending, and that a job skipped because a job it needs failed may not
block merging either.

Nothing in the repository compiles, embeds or checks Markdown, except Prettier, which
checks every file under `web/`, Markdown included.

## Decision

1. **A `changes` job decides, on every pull request, whether it changes only
   Markdown:** every path it adds, changes or deletes, both names of a renamed file
   included, ends in `.md` and lies outside `web/`. A pull request that changes nothing
   does not count.
2. **For such a pull request, `rust`, `web`, `e2e`, `supply-chain` and `image` are
   skipped by their `if:` condition.** They stay the required checks of ADR-0039 and
   report success.
3. **`secrets` always runs.** Markdown is where a name or a secret is most likely to slip
   in.
4. **When unsure, everything runs.** The build jobs are skipped only when `changes`
   reports that the pull request changes only Markdown. If `changes` fails or reports
   nothing, they run, so `changes` need not be a required check.
5. **Pushes to `main` run every job,** whatever they change. `main` stays fully checked,
   its caches stay warm, and `publish` keeps every job it needs.
6. **A check added later that reads Markdown,** such as a link checker or a check of the
   ADR index, runs in a job that is never skipped, or narrows decision 1 in the same
   change.

## Alternatives considered

- **Path filters on the workflow:** the required checks would never report, and every
  Markdown-only pull request would wait for good.
- **A path filter action from the marketplace:** one more third-party action pinned by
  commit SHA (ADR-0039, decision 15), for what one `git diff` and two `grep`s do.
- **Skipping the steps inside each job:** each job would still pay for its setup:
  service containers, the toolchain, caches. That setup is most of a short job.
- **Skipping on `main` too:** a docs-only merge would then neither build nor publish an
  image. It would save runner time, but `main` would no longer be checked in full, and
  whether a docs-only merge should redeploy is a separate question (see Consequences).

## Consequences

- A pull request that changes only Markdown gets its checks in the time `secrets` takes,
  about 20 seconds.
- Every other pull request starts its build jobs only after `changes`, a few seconds
  later than before.
- A docs-only merge to `main` still builds and publishes a new image, and the host
  takes it (ADR-0040), as before this ADR.
- The comment at the top of `.github/workflows/ci.yml` names this ADR. The required
  checks in ADR-0039, decision 17, are unchanged; `changes` is not among them.
