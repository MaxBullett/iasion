# 0026: The last login identity cannot be removed, and recovery is an operator task

- Status: accepted
- Date: 2026-11-16
- Refines: ADR-0024
- Resolves: OQ-040

## Context

Login identities are managed in the app (ADR-0024). If the last one is removed, or the
email behind it is lost, nobody can reach the household's data. It was open whether
that removal is allowed and whether the email named for first setup keeps a role
afterwards (`docs/event-storming/r1.md`, H-30).

## Decision

1. **Removing the last login identity is rejected** with a typed error. A household
   always has at least one login identity.
2. **The setup variable has no role after setup.** `IASION_SETUP_EMAIL` is read only
   while no household exists. It is not a recovery path.
3. **Recovery from a lost login identity is an operator task:** someone with access to
   the server adds a mapping. The deployment docs describe how.
4. **An admin command for recovery comes later.** It is not R1.
5. **Login identities are versioned like other household data** (ADR-0025). A removed
   login identity is a final version and stays in history.

## Alternatives considered

- **The setup email as a standing recovery identity:** a second way in that bypasses
  the mapping and stays valid forever, for an email that may no longer be in use.
- **Allow removing the last login identity:** one click locks everybody out.
- **A recovery flow in the app:** it would need a second proof of identity, which the
  app does not have (ADR-0024: no credentials in the app).

## Consequences

- S-014 gains criteria for removing a login identity.
- A member cannot be voided while a login identity maps to them: the login identity
  refers to the member (ADR-0025, decision 6).
- The check is on the number of login identities, not on whether somebody can still
  sign in with them. An email that Cloudflare Access no longer lets through still
  counts. That case is the operator task of decision 3.
- The deployment docs, still to write, describe the recovery in generic terms.
- Mapping a login identity to another member is still rejected (ADR-0024); removing
  it and mapping the email anew is the way to move it.
