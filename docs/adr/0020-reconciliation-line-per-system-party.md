# 0020: The reconciliation has one line per system party

- Status: accepted
- Date: 2026-11-15
- Refines: ADR-0011, ADR-0012, ADR-0018
- Resolves: OQ-033 (the reconciliation of a period that contains an opening balance)

## Context

ADR-0012 requires each member view to reconcile: opening + in - out + between members
= closing. Flows on the opening party's account are neither in nor out (ADR-0011), and
the change they cause is not between members either (ADR-0018). So a period that
contains an opening balance could not reconcile, and its opening figure was unknown
for the account concerned (`docs/event-storming/r1.md`, H-21). That is the normal case
in a household's first month, and whenever an account is added later.

## Decision

1. **One line per system party.** The reconciliation of a period shows the flows on
   each system party's account on a line of their own.
2. **R1 has one such line: "Opening balances".** It holds the opening balances
   recorded in the period, i.e. the flows on the opening party's account in the
   period, seen from the household.
3. **Later, the suspense party gets "Unexplained differences"** in the same way.
4. **The identity,** per member and for the household:
   tracked opening + opening balances + in - out + between members = closing.
   For the household, between members is zero.
5. **Tracked.** An account is tracked from the date of its opening balance.
6. **Not tracked is never zero.** An account that is not yet tracked at the start of
   the period is left out of the opening figure, and the opening figure names it as
   not tracked. The view stays closed and honest.

Example: all accounts have an opening balance at 2026-09-30, except Sam's cash wallet,
opened with EUR 80.00 at 2026-10-10. The view for October shows an opening figure
without the wallet, naming it as not tracked, and EUR 80.00 on "Opening balances" for
Sam and for the household. Closing includes the wallet, and the identity holds.

## Alternatives considered

- **No reconciliation for such periods:** the first month, the one the release is
  judged by, would not reconcile.
- **Count an opening balance as in:** against ADR-0011. It would show as income.
- **Show the opening figure as unknown and stop there:** honest, but the view no
  longer closes, and one late wallet blanks the whole reconciliation.
- **Only allow periods that start after every opening balance:** the member would have
  to know those dates before asking.

## Consequences

- The per-member line is the member's allocation of the opening flows on household
  accounts (ADR-0019). The members' lines sum to the household's.
- The property tests of ADR-0012 and ADR-0019 use the identity with this line.
- In and out of an account for the days of the period before it was tracked are
  missing. How the view says so follows ADR-0021.
- The glossary gains "tracked".
- Input to OQ-011: flows on the suspense party's account are meant to appear on a line
  of their own, not in in or out. The rest of OQ-011 stays open.
- S-013 gains a criterion for the example and uses the extended identity.
