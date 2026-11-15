# AGENTS.md

This project is **spec-driven**. Specs live in the repo next to the code and are the
source of truth. Code may be generated or hand-written, but it must match the specs.
Specs are never generated from code.

## Where things are

| File | Purpose |
|---|---|
| `goal-and-requirements.md` | Vision, scope, MVP boundary |
| `architecture.md` | Contexts, dependency rules, stack |
| `docs/primitives.md` | Core kernel primitives and their invariants |
| `docs/glossary.md` | Ubiquitous language. Use these terms exactly. |
| `docs/conventions.md` | Domain patterns (binding) and code conventions |
| `docs/story-map.md` | Activities, tasks, stories, with IDs |
| `docs/releases/` | Release definitions: scope, done criteria, enablers |
| `docs/open-questions.md` | Unresolved decisions and unknowns |
| `docs/adr/` | Architecture decision records (index in `docs/adr/README.md`) |
| `fixtures/` | Test fixtures. `fixtures/private/` is gitignored. |
| `<module>/SPEC.md` | Per-module spec (once modules exist) |

Read `goal-and-requirements.md`, `docs/primitives.md`, `docs/glossary.md` and
`docs/conventions.md` before any task.

`CLAUDE.md` is a symlink to this file. Edit `AGENTS.md` only.

## Rules

1. **No silent assumptions.** If a spec does not answer a question you need answered,
   add an entry to `docs/open-questions.md` and stop or ask. Do not pick an answer yourself.
2. **Domain rules need a source.** Any rule about tax, law, banking conventions, day-count
   conventions, benefits or similar must cite a primary source in the spec (statute,
   official publication, contract document). Uncited domain rules are open questions,
   not facts.
3. **Use the glossary.** Do not invent new domain terms. If a new term is needed, propose
   a glossary entry in the same change.
4. **Specs change with code.** When you change a boundary, contract, invariant or decision,
   update the corresponding spec in the same change.
5. **Decisions go in ADRs.** Any decision that constrains future work gets an ADR.
   Accepted ADRs are never edited; they are superseded by new ones.
6. **Never commit personal data or secrets.** See "Personal data and secrets" below. This
   rule has no exceptions and overrides convenience.
7. **Traceability.** Stories have IDs (`S-###`), activities (`A#`) and tasks (`A#.#`) too. Specs reference
   the stories they cover. Tests reference the spec section or story they verify.
   A story without a spec, or a spec without tests, is incomplete.

## Personal data and secrets

This repository is public (or will be). Everything committed, including commit messages
and history, must be safe to publish.

### Never in the repo

- **Personal data of real people**, the maintainers and their families included: names,
  addresses, contact details, dates of birth, employers, health or family details.
- **Real financial data:** account numbers, IBANs, card numbers, balances, amounts,
  salaries, merchants or counterparties taken from someone's actual records, and any
  real document (statements, payslips, contracts, tax assessments).
- **Secrets:** API keys, tokens, passwords, private keys, certificates, database
  connection strings, session cookies, Cloudflare Access credentials or service tokens.
- **Details of a specific deployment:** hostnames, domains, IP addresses, internal
  network layout, hypervisor or VM names. Deployment docs describe generic setups.

### Instead

- Examples, specs, tests and fixtures use the **reference household** defined in
  `fixtures/README.md`. Do not invent other people when the reference household fits.
- Amounts in examples are synthetic. Placeholder companies use obviously fictional
  names (e.g. "ACME GmbH", "Corner Grocer").
- Real documents for local golden tests live in `fixtures/private/` (gitignored).
  Tests that need them skip cleanly when they are absent.
- Configuration that differs per instance or is secret comes from environment
  variables. `.env` is gitignored; `.env.example` lists every variable with a
  placeholder value and is the only env file that may be committed.
- Commits use the author's GitHub no-reply email address.

### When something slips

- If you notice personal data or a secret in a change, **stop**: do not commit, remove
  it, and tell the human.
- If it is already committed, a follow-up commit does not fix it. Tell the human: the
  history must be rewritten before pushing (or, if pushed, rewritten and force-pushed),
  and any leaked secret must be revoked and rotated regardless.
- Never paste personal data or secrets into issues, pull requests, commit messages,
  logs, test output or error messages either.

### Automated checks

Planned for R1 (see `docs/releases/r1.md`, enablers): secret scanning in a pre-commit
hook and in CI, plus a local, gitignored denylist of private terms (`.private-terms`)
checked before each commit. Until these exist, the rules above are enforced by review
only. Do not claim a change passed these checks unless they ran.

## Git and commits

### Branches and merging

- Never commit directly to `main`. Every change happens on a branch and lands through a
  pull request.
- One branch per story, decision or chore. Name: `<type>/<id>-<slug>`, e.g.
  `feat/s-006-record-spending`, `docs/adr-0013-persistence`, `chore/secret-scanning`.
  Use the most specific ID available (story, ADR, OQ); omit it only for chores without one.
- Pull requests are squash-merged. The squash commit on `main` follows the message
  format below and is the record that history readers rely on.

### Commits on a branch

- One logical change per commit. Commit after each green step (tests and checks pass).
  Exception: the reviewed-tests step of the story workflow may be committed while
  failing; squash-merging keeps `main` green.
- Do not mix unrelated changes (e.g. a refactor and a feature) in one commit; split
  them.
- Never rewrite history that has been pushed and shared, except to remove personal data
  or secrets (see above), and only with the human's go-ahead.

### Message format

[Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <summary>

<body: what changed and why, wrapped at 72 characters>

Refs: <IDs>
```

- **type:** `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`, `chore`.
- **scope:** a bounded context (`household`, `ledger`), a crate or frontend area, or a
  doc area (`adr`, `story-map`, `release`, `agents`). Optional for repo-wide changes.
- **summary:** imperative, lower case, no trailing period, at most 72 characters.
  "record aggregate flows over an interval", not "Added stuff".
- **body:** required unless the summary says everything. Explain why, not how.
- **Refs trailer:** required whenever the change relates to a story, ADR or open
  question, e.g. `Refs: S-006, ADR-0009`. This keeps `git log --grep S-006` useful.
- Breaking changes to an API or persisted format: `!` after the type/scope and a
  `BREAKING CHANGE:` trailer.
- Commit messages follow the personal data rules: no real names, amounts or
  instance details.

## Workflow per story

1. Confirm the story and its acceptance criteria (Given/When/Then) exist in the spec.
   Create the story branch.
2. Write tests from the acceptance criteria first. Commit them. Stop for human review of
   the tests.
3. Implement until tests pass, committing each green step. Do not change tests to make
   them pass without review.
4. Update specs and glossary if anything changed, in the same branch.
5. Open a pull request titled like the final squash commit, listing the story ID and the
   acceptance criteria covered.

## Testing expectations

- Property-based tests for invariants (e.g. transactions net to zero).
- Golden tests against real-world artifacts where they exist (e.g. a bank's amortization
  schedule). Fixtures containing personal data must not be committed; use anonymized
  or synthetic equivalents.

## Status

Pre-code. R1 is defined (`docs/releases/r1.md`) and its open questions are resolved.
Next: event storming for R1, then infrastructure ADRs and the walking skeleton.
