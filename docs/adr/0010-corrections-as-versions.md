# 0010: Corrections as versions of a fact

- Status: accepted
- Date: 2026-11-15
- Resolves: OQ-020
- Refined by: ADR-0015, ADR-0016, ADR-0017, ADR-0025, ADR-0028

## Context

R1 is manual entry, so wrongly entered facts are routine (S-008). Facts are
append-only and corrections are new facts (`docs/conventions.md`), but the mechanism
was open. Facts are also bitemporal: every fact carries valid time and recorded time.

Classic bookkeeping corrects by reversal: a mirror-image entry cancels the original,
then a correct entry is added. That exists so closed periods are never rewritten.
Iasion has no closed periods, and recorded time already provides the audit trail.

## Decision

1. **Facts have identity and versions.** Each fact (transaction, opening balance,
   checkpoint, dimension assignment) has a stable ID and one or more versions. Each
   version carries its recorded time.
2. **A correction is a new version.** It replaces the previous version in all current
   views. Previous versions stay stored and are never modified.
3. **Voiding is a final version** marking the fact void. A voided fact affects no
   current view.
4. **The transaction is the versioned unit.** Flows are never corrected on their own;
   correcting any flow creates a new version of the whole transaction, which must
   satisfy the netting invariant again.
5. **Corrections apply retroactively in valid time.** A corrected October flow changes
   October's figures. "As known on date X" queries use the version that was current at
   recorded time X.
6. **No reversal entries** for corrections.

## Alternatives considered

- **Reversal plus new entry:** keeps every effect visible as flows, but duplicates the
  audit trail that recorded time already gives, and clutters period views with
  offsetting entries unless the UI hides them.
- **Versioning individual flows:** finer-grained, but a single changed flow can break
  the netting invariant of its transaction.

## Consequences

- Persistence stores facts as (ID, version, recorded time, payload), with the current
  view derived as the latest non-void version per ID.
- Opening balances and checkpoints are corrected the same way as flows (S-004, S-008).
  Dimension assignments already worked this way (S-010).
- Every query must state which knowledge it uses: current (default) or as known at a
  recorded time.
- This is close to event sourcing for ledger facts without committing to it. OQ-003 is
  narrowed accordingly.
- A real-world reversal (e.g. a refund from a merchant) is not a correction. It is a
  new transaction with its own valid time.
