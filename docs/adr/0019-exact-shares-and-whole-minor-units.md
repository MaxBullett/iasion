# 0019: Ownership shares are exact fractions, and attribution allocates whole minor units

- Status: accepted
- Date: 2026-11-15
- Refines: ADR-0012, ADR-0018
- Resolves: OQ-024

## Context

Ownership shares sum to 100%, facts use exact decimals, and member figures must sum
to the household figures (ADR-0012). Event storming for R1
(`docs/event-storming/r1.md`, H-06) showed two gaps: three equal shares cannot be
written as decimals that sum to exactly 100%, and half of EUR 54.21 is not a whole
cent.

## Decision

1. **Shares are exact fractions.** An ownership share is stored as an integer
   numerator and denominator. The shares of an account sum to exactly 1. The UI may
   offer percentages or "split equally".
2. **Whole minor units.** Attribution allocates each flow to members in whole minor
   units of its unit (cents for EUR).
3. **Largest remainder.** Each member first gets their part rounded down to a whole
   minor unit. The minor units left over go, one each, to the members with the largest
   remainders. The sign of the flow is applied afterwards.
4. **Ties are broken deterministically, but not by a fixed order of members.** The
   tie-break is derived from the flow's ID, so that stray minor units do not always
   land on the same member.
5. **What is allocated.** A flow on a household account is allocated by the ownership
   shares of that account. A flow that counts as in or out is allocated with the same
   method; its weights are the amounts already allocated to each member from the
   household flows it is attributed against (ADR-0018).
6. **Member figures are sums of allocations.** A member's attributed position, in and
   out are sums of allocated whole minor units, and between members is the remainder
   (ADR-0018). The reconciliation invariants therefore hold exactly.

Example: EUR 54.21 spent at Corner Grocer from the joint current account, owned half
and half. The flow of -54.21 on the joint account is allocated as -27.11 and -27.10;
which member gets the extra cent follows from the flow's ID. The flow of +54.21 on
Corner Grocer's account is out. Its weights are 27.11 and 27.10, so out is 27.11 and
27.10 for the same members, and between members is zero.

## Alternatives considered

- **Percentages with a fixed number of decimals:** thirds never sum to 100%.
- **Sub-cent precision in member figures:** exact until displayed, but rounding for
  display breaks the reconciliation on screen.
- **Round each member's part on its own:** the parts do not sum to the flow.
- **Give the remainder to the first owner, or in a fixed order:** a systematic bias
  towards one member.
- **Attribute a position as share times balance:** a member's position would no longer
  be the sum of that member's flows, and a balance of an odd number of cents would
  need its own rounding rule.
- **Weights for in and out from ownership shares directly:** with a tie, the two sides
  of a plain purchase could round in opposite directions and leave one cent between
  members on an ordinary grocery bill.

## Consequences

- Property tests, all in exact minor units: every allocation sums to its flow; each
  member reconciles; between-members lines sum to zero; members' in and out sum to
  the household's.
- A member's attributed position is the sum of that member's allocations. It can
  differ from share times balance by a few minor units.
- A transaction with one flow on a household account and one on a counterparty's
  account always has between members of exactly zero. A split can still leave a stray
  minor unit between members.
- The tie-break must be stable: the same flow ID gives the same result in every view
  and every recomputation. The exact function is specified with the module that
  implements it.
- Allocations depend on ownership shares. What a corrected share does to them is part
  of OQ-023.
- Each currency needs a known minor unit. Units that are not currencies get theirs
  when they are introduced.
- S-003 accepts fractions; the typed error for a wrong sum names the actual sum.
  S-013 gains a criterion for the stray cent.
