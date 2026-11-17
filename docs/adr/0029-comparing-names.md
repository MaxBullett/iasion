# 0029: Names are stored trimmed and compared ignoring case

- Status: accepted
- Date: 2026-11-16
- Refines: ADR-0025
- Resolves: OQ-023 (name comparison)

## Context

Purpose names are unique (S-009), and registering a party under a name that already
exists warns (S-002). Nothing said when two names are the same, or whether renaming a
purpose to an existing name fails like creating it does
(`docs/event-storming/r1.md`, H-04).

## Decision

1. **Names are stored trimmed:** without whitespace at the start or the end.
2. **Two names are the same if they are equal ignoring case,** after trimming.
3. **Purposes:** creating a purpose under the name of another purpose is rejected, and
   so is renaming one to the name of another. That would be a merge, which is not R1.
4. **Parties:** the warning for a name that already exists (S-002) uses the same
   comparison. The member can still proceed.
5. **Ignoring case means Unicode default case folding,** independent of the locale of
   the server or the browser.

## Alternatives considered

- **Exact comparison:** "Groceries" and "groceries " would be two purposes.
- **Store names as typed and trim only when comparing:** stored names would differ in
  whitespace nobody can see.
- **Rename onto an existing name merges the two:** a merge moves assignments between
  purposes and cannot be undone by renaming back. Not R1.
- **Case rules of the member's language:** the same two names would be equal on one
  instance and different on another.

## Consequences

- ADR-0025 lets names be corrected freely. For purposes that now ends where the new
  name is the name of another purpose.
- Changing only the case of a purpose's own name is a correction like any other.
- A name that is empty after trimming is rejected.
- Voided purposes do not count: the name of a voided purpose can be used again.
- Names are stored with the case the member typed. Only the comparison ignores it.
- S-002 and S-009 gain or sharpen criteria.
