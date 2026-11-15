# 0011: Opening balance as a transaction against an opening party

- Status: accepted
- Date: 2026-11-15
- Resolves: OQ-009 (representation; backfill follows from ADR-0010)
- Refined by: ADR-0014

## Context

Present-first onboarding (ADR-0007, proposed) gives each account an opening balance:
its positions at a stated date, taken as given. OQ-009 asked whether that is a set of
flows (which per ADR-0006 need a counterparty) or a distinct kind of fact that states
positions directly, and what happens when history before it is backfilled.

## Decision

1. **An opening balance is a checkpoint taken as given, plus a transaction.** The user
   states the observed balance of an account at a date. The system records it as a
   checkpoint and books one transaction at that date between the account and the
   opening party, so that the derived position equals the checkpoint.
2. **Opening party.** Each household has exactly one opening party, created with the
   household. It is outside the boundary and has party kind `system`, a new kind for
   parties that exist for bookkeeping rather than in the world. Its balance per
   account is that account's starting equity.
3. **Positions are always the sum of flows.** There is no second derivation rule.
4. **Views exclude opening flows from in and out.** Flows against the opening party
   affect positions and the balance sheet, but never appear as income or spending.
5. **One opening balance per account.** Changing it is a correction: a new version of
   the opening transaction and its checkpoint (ADR-0010).
6. **Backfill is a correction** (later release). When history before the opening date
   is recorded, the opening transaction gets a new version at the earlier date and
   amount. Flows dated before the opening balance are not accepted until then
   (OQ-019).

## Alternatives considered

- **Distinct fact stating positions directly:** no extra party and no view exclusion,
  but positions then have two derivation rules (stated vs. summed), and value appears
  without a counterparty, against ADR-0006.

## Consequences

- ADR-0006 holds without exceptions: every position change has a counterparty.
- One property test covers all positions: derived position = sum of flows.
- Opening balance and (in R2) suspense booking share one mechanism: a checkpoint plus a
  transaction against a `system` party. The suspense party is likely the second
  `system` party (input to OQ-011).
- Every in/out view must exclude flows against `system` parties. This rule belongs in
  the Ledger context, not in individual views.
- Party kinds gain `system`, which must never be used for real-world counterparties.
