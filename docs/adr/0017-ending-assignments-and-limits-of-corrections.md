# 0017: Purpose assignments end without blocking, and a correction may change anything but identity

- Status: accepted
- Date: 2026-11-15
- Refines: ADR-0010, ADR-0015, ADR-0016
- Resolves: OQ-029 (the points ADR-0016 left open)

## Context

ADR-0016 settled what a purpose attaches to and how assignments follow their flow. It
left four points in OQ-029:

- a correction that keeps a flow but makes it unable to carry a purpose, e.g. by
  moving it to a household account;
- removing a purpose without assigning another;
- the confidence of a manual assignment (provenance includes a confidence);
- whether there is any limit to what a correction may change.

## Decision

1. **A flow that can no longer carry a purpose loses it.** If a correction keeps a flow
   but makes it unable to carry a purpose, its assignment stops being current, exactly
   as for a removed flow (ADR-0016). Dependent facts never block (ADR-0015). The UI
   warns before saving.
2. **A purpose can be removed without replacement.** The flow is then Unclassified.
   The previous assignment is kept in history.
3. **A manual assignment is certain.** Its provenance is: assigned by a user,
   confirmed, confidence "certain". Confidence only matters for suggestions, which
   come later.
4. **A correction may change anything but identity.** There is no limit beyond this:
   the new version satisfies all rules and breaks no independent fact (ADR-0015).

## Alternatives considered

- **Reject a correction that makes a classified flow unable to carry a purpose:** a
  dependent fact would block its parent, against ADR-0015 and ADR-0016.
- **Move the purpose to another flow automatically:** the system would guess what the
  member meant.
- **No confidence on manual assignments:** provenance would have two shapes, one for
  users and one for suggestions.
- **Limit corrections, e.g. keep accounts fixed:** a wrong account is a common manual
  mistake. It would have to be fixed by voiding and recording again, which loses the
  link between the two in history.

## Consequences

- OQ-029 is closed.
- To warn before saving, the UI must learn from the API which assignments a correction
  would end. The rule itself stays in the backend (`docs/conventions.md`).
- Removing a purpose is a command of its own. S-010 gains criteria for it and for the
  warning.
- "Certain" is the top value of confidence. The scale is defined when suggestions
  arrive.
- Since a correction can change everything else, a fact's ID is what links its
  versions. Flow IDs continue only where the correction says so (ADR-0016).
