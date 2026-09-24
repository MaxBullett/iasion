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
6. **Never commit personal data.** Real documents live in `fixtures/private/`.
7. **Traceability.** Stories have IDs (`S-###`), activities have IDs (`A#`). Specs reference
   the stories they cover. Tests reference the spec section or story they verify.
   A story without a spec, or a spec without tests, is incomplete.

## Workflow per story

1. Confirm the story and its acceptance criteria (Given/When/Then) exist in the spec.
2. Write tests from the acceptance criteria first. Stop for human review of the tests.
3. Implement until tests pass. Do not change tests to make them pass without review.
4. Update specs and glossary if anything changed.

## Testing expectations

- Property-based tests for invariants (e.g. transactions net to zero).
- Golden tests against real-world artifacts where they exist (e.g. a bank's amortization
  schedule). Fixtures containing personal data must not be committed; use anonymized
  or synthetic equivalents.

## Status

Pre-code. Current phase: story mapping and domain modeling.
