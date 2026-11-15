# 0022: In and out are kernel terms; income and spending are view terms, netted per purpose

- Status: accepted
- Date: 2026-11-15
- Refines: ADR-0013
- Resolves: OQ-033 (terms, and refunds)

## Context

The specs use "in" and "out" (S-013, ADR-0012, ADR-0013) next to "income" and
"spending" (S-005, S-006, ADR-0011) without saying whether they are the same
(`docs/event-storming/r1.md`, H-22). A refund is a new transaction (ADR-0010), and it
was open whether it shows as an inflow under its purpose or reduces the outflow.

## Decision

1. **In and out are kernel terms and say direction only.** A flow that counts is out
   if it is positive on the counterparty's account and in if it is negative
   (ADR-0013). They are gross and are never netted.
2. **A view by purpose nets in and out per purpose** over its period. A refund, which
   is in with the purpose Groceries, reduces Groceries.
3. **Income and spending are view terms.** They are the purposes grouped by their net
   direction in the period: a purpose with net in is income, a purpose with net out is
   spending.
4. **Unclassified is not netted.** It is shown as in and out separately.
5. **The reconciliation keeps in and out** (ADR-0020). Total income minus total
   spending, plus Unclassified in minus Unclassified out, equals in minus out.

## Alternatives considered

- **Gross per purpose:** a refund would raise both sides, and a purpose with returns
  would look bigger than it is.
- **A fixed direction per purpose** (an income purpose or a spending purpose): refunds
  and purposes that go both ways would not fit, and the member would have to decide the
  direction up front.
- **Net Unclassified as well:** it would hide how much is still to be classified.

## Consequences

- The same purpose can be income in one period and spending in another, and income in
  one member's view and spending in the household's.
- Where accepted ADRs and the glossary say "income or spending" for what a transfer or
  an opening balance is not (ADR-0011), read "in or out".
- Member attribution, the invariants of ADR-0012 and the allocation of ADR-0019 work on
  in and out, before netting.
- The story titles of S-005 and S-006 keep "income" and "spending" as plain words for
  what the member records.
- The glossary defines both pairs with this distinction. S-013 is reworded and gains a
  criterion for a refund.
- Not stated: where a purpose whose in and out cancel to exactly zero is listed.
