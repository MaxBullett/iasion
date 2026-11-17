# 0007: Onboarding present first, with explicit coverage and gaps

- Status: accepted (2026-11-15)
- Date: 2026-09-27
- Refined by: ADR-0009, ADR-0010, ADR-0011, ADR-0021. Open details: OQ-010 to OQ-016.

## Context

A household never starts with a complete history. Statements go back a limited time,
some accounts are only partly recorded, and classification is uneven. The MVP question
(where should the next unit of surplus go) needs current balances, open contracts and
incomes far more than it needs history. History matters later, mainly to fit
statistical generators.

If onboarding requires full history first, users stall. If missing history is silently
treated as zero or as "nothing happened", balances drift, statistical generators are
fitted on biased data, and projections look more certain than they are.

## Decision

1. **Present first, history second.** Onboarding starts with current balances, open
   contracts and incomes. History is backfilled later to improve statistical
   generators.
2. **Coverage per account.** Each account has its own opening balance and coverage
   intervals, each marked complete, partial or unknown. Missing data is never read
   as zero.
3. **Gaps are explicit.** Statement balances are checkpoints. Unexplained differences
   between a checkpoint and the position derived from the opening balance and recorded
   flows are booked as flows against a suspense party. The suspense balance is a data
   quality metric.
4. **Data quality flows into uncertainty.** Statistical generators carry parameter
   uncertainty into projections, are fitted only on periods with sufficient
   classification coverage, and must not assume missingness is random.

## Alternatives considered

- **History first:** import all available history, then derive current balances.
  Accurate where data exists, but slow to first value and still incomplete.
- **Single household-wide cutoff date:** simpler, but accounts are opened, closed and
  documented at different times. A global date either discards data or pretends to
  have it.
- **Force balances to match silently:** adjust flows or positions so derived balances
  equal statements. Hides data quality problems and breaks two-sidedness.
- **Treat missing data as zero:** simplest, but biases every statistic and projection.

## Consequences

- Users reach a useful balance sheet quickly.
- The kernel needs opening balances, coverage intervals, checkpoints and a suspense
  party (see `docs/primitives.md`).
- Every aggregate over time must respect coverage: a sum over an interval that is not
  completely covered is partial or unknown, never a complete total.
- The suspense balance becomes a visible metric that users can drive down.
- Statistical generators need a fitting procedure aware of coverage and of
  non-random missingness, and parameter uncertainty must reach the scenario engine.
- Open contracts are registered mid-life, usually without their past lifecycle
  events. Their state at registration needs a representation (OQ-016).
- Several details are open: OQ-009 to OQ-016.
