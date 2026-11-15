# 0015: An opening balance is one fact, comes first, and corrections never invalidate independent facts

- Status: accepted
- Date: 2026-11-15
- Refines: ADR-0010, ADR-0011
- Refined by: ADR-0017
- Resolves: OQ-027, except accounts registered for parties outside the boundary

## Context

ADR-0011 records an opening balance as a checkpoint plus a transaction, allows one per
account and treats a change as a correction. ADR-0010 lists the opening balance as a
kind of fact with versions. Event storming for R1 (`docs/event-storming/r1.md`, H-09,
H-10) left open:

- whether a household account without an opening balance can take flows or
  checkpoints (OQ-019 only settled flows dated before an existing one);
- what a correction of an opening balance may change once the account has flows or
  checkpoints, and whether it can be voided and recorded again;
- whether the opening balance is one fact, or a checkpoint and a transaction that
  happen to be linked, and which aggregate enforces "at most one".

## Decision

1. **The opening balance comes first.** A flow or a checkpoint on a household account
   without an opening balance is rejected. That includes a flow that makes the account
   the destination of a transfer. Implicit accounts need none (ADR-0013).
2. **A new account is not a special case.** "New account, opened on D" is an option of
   the entry form. It records an opening balance of 0.00 for the end of the day before
   D, which is a true statement: the account held nothing. "Existing account" asks for
   a balance and a date.
3. **One fact.** An opening balance has its own identity and versions (ADR-0010). Its
   checkpoint and its transaction are parts of it. They are written together and are
   never corrected or voided on their own. Corrections target the opening balance.
4. **At most one non-void opening balance per account,** enforced by the Account
   aggregate.
5. **A correction never invalidates independent recorded facts.** Independent facts
   stand on their own, such as other transactions and checkpoints. A correction or
   void that would leave one of them breaking a rule is rejected with a typed error
   naming the conflicting facts. Dependent facts, such as dimension assignments on the
   flows of the corrected transaction, follow their parent instead and never block it
   (ADR-0016).
6. **Applied to the opening balance:**
   - The amount can always be corrected.
   - The date can be corrected to an earlier date (the direction of backfill,
     ADR-0011).
   - A correction of the date to on or after the date of any flow or checkpoint on the
     account is rejected.
   - It can be voided only while the account has no other facts. After a void, a new
     opening balance can be recorded.

"Flow or checkpoint on the account" and "other facts" mean current, non-void ones: a
voided fact did not happen (ADR-0010).

## Alternatives considered

- **Accept flows on an account without an opening balance:** its positions would be
  sums over an unknown start, which is what "missing data is never zero" (ADR-0007)
  forbids.
- **A kernel notion of a newly opened account:** a second way for an account to start.
  An opening balance of 0.00 says the same with the existing mechanism.
- **Two linked facts (checkpoint and transaction):** each could be corrected alone,
  and the two could drift apart.
- **Let a correction cascade** (e.g. void the flows that a later opening date leaves
  behind): one correction would silently rewrite other facts.
- **Supersede ADR-0011 with a consolidated ADR:** nothing in ADR-0011 is reversed, so
  superseding would misstate it. The consolidated rules live in `docs/primitives.md`.

## Consequences

- Recording a transaction or a checkpoint needs to know each household account's
  opening balance: whether it exists, and its date.
- Correcting or voiding an opening balance needs to know the other facts on the
  account. The typed error names them, so the member can correct or void them first.
- Moving an opening date earlier and then recording the flows in between is a backfill
  done by hand. R1 has no feature for it, but does not prevent it.
- Decision 5 is general. Purpose assignments are dependent facts: they follow the
  transaction they sit on (ADR-0016).
- Moving an opening balance onto the date of an existing checkpoint is rejected.
  Whether a checkpoint may be recorded on the opening date is still open (OQ-028).
- The entry form offers "new account" and "existing account". The domain sees an
  opening balance either way.
- Whether these rules apply to accounts registered for a party outside the boundary
  (e.g. a loan account) stays open for a later release (OQ-027).
- Stories gain criteria: S-004 (new account), S-005, S-007 and S-011 (no opening
  balance), S-008 (correcting and voiding an opening balance).
