# 0021: Figures carry flags, and a total takes the union

- Status: accepted
- Date: 2026-11-15
- Refines: ADR-0007, ADR-0009
- Resolves: OQ-033 (how figure states combine, and which figures are marked)

## Context

ADR-0007 forbids reading missing data as zero. ADR-0009 added that views need an
approximate and a partial state besides exact and unknown. How these combine in a
total was not defined (`docs/event-storming/r1.md`, H-21): S-012 only said that
approximate propagates, the glossary defined partial for period figures only, and it
was unclear what net worth is when one account is unknown, or which lines of a view
grouped by purpose are marked when an account's data is missing.

## Decision

1. **A figure is a value plus a set of flags.** The flags are: approximate; and
   partial, with a list of named exclusions, each with its reason.
2. **A total takes the union.** It is approximate if any of its parts is, and it is
   partial with the exclusions of all its parts.
3. **An unknown part is left out and named.** A single figure that cannot be stated is
   unknown and is never shown as zero. A total leaves it out and is flagged partial,
   naming it and why. Net worth with an account that is not tracked yet is the total of
   the other accounts, flagged partial, naming that account and the reason.
4. **In and out by purpose:**
   - If it is known which line the missing data belongs to, that line is flagged. An
     excluded aggregate flow (ADR-0009) flags the line of its purpose, or the
     Unclassified line if it has none.
   - If it is not known, as for an account that is not tracked yet, the view is flagged
     once with a note. The lines are not flagged one by one.
5. **Totals of a view follow rule 2.**

## Alternatives considered

- **One state per figure, with an order (exact, approximate, partial, unknown):** the
  worst state would win and the reasons would be lost.
- **A total with an unknown part is unknown:** one wallet that is not tracked yet would
  blank the household's net worth.
- **Flag every purpose line when an account is not tracked:** every line would carry
  the same note, and the flags that mean something would drown.

## Consequences

- "Exact" means no flags. The four states of ADR-0009 remain as a reading of a single
  figure: no flags, approximate, partial, or no value at all.
- A figure can be approximate and partial at once.
- The API returns each figure with its flags and exclusions. The frontend shows them in
  text, not only through colour or icons (`docs/conventions.md`).
- Flags survive attribution: a member figure inherits the flags of the figures it is
  allocated from.
- A checkpoint difference that is approximate (ADR-0014) is a figure with that flag.
- The glossary entry for partial is widened from period figures to all figures, and
  unknown is added.
- S-012 and S-013 state the cases: net worth with an account that is not tracked, an
  excluded aggregate, and a period that starts before an account is tracked.
