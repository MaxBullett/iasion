# 0001: Spec-driven, test-first development

- Status: accepted
- Date: 2026-09-25

## Context

The project is built largely by AI coding agents and is meant to be long-lived.
Agents lose context between sessions and fill gaps with plausible guesses. We also
want to be able to switch agent tooling without losing project intent.

## Decision

- Specs in the repo are the source of truth. Code is checked against them; specs are
  never generated from code.
- Work flows story → acceptance criteria → tests (human-reviewed) → implementation.
- Agents log unknowns in `docs/open-questions.md` instead of deciding them.
- Domain rules require a cited primary source.
- Stories, specs and tests are linked by ID.
- `AGENTS.md` holds agent rules; `CLAUDE.md` is a symlink to it.

## Alternatives considered

- Code-first with docs afterwards: fast initially, but agents drift and intent is lost.
- Tool-specific planning formats: lock the project to one agent tool.

## Consequences

- Slower start; more writing before code.
- Reviewing tests becomes the main quality gate, which is cheaper than reviewing
  implementations.
- Specs are plain markdown, portable across tools.
