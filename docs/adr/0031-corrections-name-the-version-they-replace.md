# 0031: A correction names the version it replaces

- Status: accepted
- Date: 2026-11-17
- Refines: ADR-0010
- Refined by: ADR-0033
- Resolves: OQ-030

## Context

Two members may correct or void the same fact at the same time, or record an opening
balance for the same account. Without a rule the last write silently wins, and the
first member's correction disappears from current views without anybody noticing
(`docs/event-storming/r1.md`, H-15).

## Decision

1. **Every correction or void names the version it replaces.** If that version is no
   longer the current one, the command is rejected with a typed error naming the
   current version. This holds for ledger facts (transactions, opening balances,
   checkpoints, dimension assignments) and for household data (parties, accounts,
   purposes, login identities).
2. **Assigning a purpose names the assignment it replaces,** or states that the flow
   has none. Two members classifying the same unclassified flow at the same time: the
   second is rejected.
3. **Recording something new names no version.** Invariants are checked against the
   state that is current when the command is written, as one step: of two opening
   balances recorded for one account at the same time, the second is rejected
   (ADR-0015); of two removals of a household's last two login identities, one is
   rejected (ADR-0026).
4. **No merging, no last write wins.** The member sees the current version and decides
   again.

## Alternatives considered

- **Last write wins:** a correction can vanish from current views without a trace in
  any view. Versions keep it stored, but nobody looks there.
- **Locking a fact while a member edits it:** locks need expiry and a way to break
  them, for a household of a few people.
- **Merging concurrent corrections field by field:** a transaction must net to zero as
  a whole (ADR-0013); two corrections that each net to zero can merge into one that
  does not.

## Consequences

- Commands that correct or void carry the version they replace, and the API returns
  the current version with every fact.
- Persistence must check and write in one step per consistency boundary. The
  persistence ADR has to provide that.
- S-008 and S-015 gain criteria. Conventions say so.
