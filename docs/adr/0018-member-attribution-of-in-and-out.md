# 0018: In and out are attributed to whose money paid and whose account received

- Status: accepted
- Date: 2026-11-15
- Refines: ADR-0012
- Refined by: ADR-0019
- Resolves: OQ-035

## Context

ADR-0012 gives member views three kinds of movement (in, out, between members) and
two invariants. Its rule 2 computes between members for a transfer with one source, one
destination and one amount.

Since ADR-0013 a transaction can have any number of flows, and household in and out
are read from the flows on counterparties' accounts. Which member each of those flows
is attributed to was not stated (`docs/event-storming/r1.md`, H-25). Where the flows
on household accounts go in both directions and the accounts have different owners,
several answers reconcile.

## Decision

1. **One principle.** Money out is attributed to the members whose money paid it.
   Money in is attributed to the members whose account received it. Both run through
   ownership shares, per unit.
2. **Opposite sign.** A flow that counts as in or out is attributed in proportion to
   the flows on household accounts of the opposite sign in the same transaction, each
   split by the ownership shares of its account. Out (a positive flow on a
   counterparty's account) follows the negative household flows. In follows the
   positive ones.
3. **Fallback.** If the transaction has no household flow of the opposite sign, the
   flow is attributed in proportion to the household flows of the same sign.
4. **Between members is the remainder.** For each member: change in attributed
   positions, minus in, plus out. This generalises rule 2 of ADR-0012 and gives the
   same result for the transfer it describes.
5. **The kernel never infers intent.** A fee meant to be Alex's alone is recorded from
   Alex's own account.

Examples, reference household, joint current account owned half and half:

| Transaction | In | Out | Between members |
|---|---|---|---|
| ACME GmbH -3,800.00, Alex's current account +3,000.00, joint +800.00 | Alex 3,400.00, Sam 400.00 | none | 0 |
| Joint -202.00, Alex's cash wallet +200.00, Example Bank +2.00 | none | Alex 1.00, Sam 1.00 | Alex +100.00, Sam -100.00 |
| ACME GmbH -3,800.00, Example Bank +5.00, Alex's current account +3,795.00 (fallback: a fee withheld from an incoming payment) | Alex 3,800.00 | Alex 5.00 | 0 |

## Alternatives considered

- **Let the member state who bears a flow:** flexible, but the kernel would store
  intent, and the same statement line could give different figures depending on who
  entered it.
- **Attribute by the member who benefits** (in the second example, the fee to Alex,
  who got the cash): needs a notion of benefit that the flows do not contain.
- **A successor to ADR-0012:** its decisions 1, 3, 4 and 5 stand unchanged. Only the
  formula in decision 2 is generalised.

## Consequences

- The invariants of ADR-0012 keep holding: each member reconciles, between-members
  lines sum to zero across the household, and members' in and out sum to the
  household's. They are property tests, together with a test for the fallback.
- A transaction with a flow on a system party's account, i.e. an opening balance,
  changes attributed positions without in or out. That change is not between members
  either, since it would not sum to zero across the household. Such a transaction lies
  inside a period only if the period starts on or before the opening balance, which is
  the case left open in OQ-033.
- "Per unit" is trivial in R1, which is EUR only. Later, an exchange with an outside
  party (e.g. buying a security) is out in one unit and in in another under this rule.
  Whether in and out should show exchanges that way is open (OQ-038).
- Accounts owned partly by a party outside the boundary are not covered (OQ-022).
- Amounts are attributed in whole minor units (ADR-0019).
- S-013 gains criteria for the three examples.
